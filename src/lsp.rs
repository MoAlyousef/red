use anyhow::{anyhow, Context, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use lsp_types as lsp;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex, OnceLock,
};
use url::Url;

use crate::diagnostics;
use fltk::app;
// STATE not used directly here; keep LSP isolated

static REQ_ID: AtomicU64 = AtomicU64::new(1);
static AVAILABLE: AtomicBool = AtomicBool::new(false);
static DISABLED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
enum Outgoing {
    Request {
        id: u64,
        method: String,
        params: JsonValue,
    },
    Notification {
        method: String,
        params: JsonValue,
    },
}

// ----- Lightweight logger controlled by env: RED_LSP_LOG ("stderr" or filepath), RED_LSP_LOG_FULL=1 -----

enum LogTarget {
    Stderr,
    File(std::sync::Mutex<std::fs::File>),
    None,
}

static LOG_TARGET: OnceLock<LogTarget> = OnceLock::new();

fn log_target() -> &'static LogTarget {
    LOG_TARGET.get_or_init(|| match std::env::var("RED_LSP_LOG") {
        Ok(val) if val.eq_ignore_ascii_case("stderr") => LogTarget::Stderr,
        Ok(path) if !path.is_empty() => {
            let file = OpenOptions::new().create(true).append(true).open(path).ok();
            if let Some(f) = file {
                LogTarget::File(std::sync::Mutex::new(f))
            } else {
                LogTarget::Stderr
            }
        }
        _ => LogTarget::None,
    })
}

pub(crate) fn lsp_log(msg: &str) {
    match log_target() {
        LogTarget::Stderr => eprintln!("[LSP] {msg}"),
        LogTarget::File(mf) => {
            if let Ok(mut f) = mf.lock() {
                let _ = writeln!(f, "[LSP] {msg}");
            }
        }
        LogTarget::None => {}
    }
}

// Full JSON logging helper intentionally omitted to keep logs concise.

pub struct LspClient {
    child: Child,
    tx: Sender<Outgoing>,
    _writer: std::thread::JoinHandle<()>,
    _reader: std::thread::JoinHandle<()>,
    caps: Arc<Mutex<Option<lsp::ServerCapabilities>>>,
    ready: Arc<std::sync::atomic::AtomicBool>,
    queue: Arc<Mutex<Vec<Outgoing>>>,
}

impl LspClient {
    fn spawn_server(root: &Path) -> Result<Child> {
        let ra = std::env::var("RED_RA_PATH").unwrap_or_else(|_| "rust-analyzer".to_string());
        let mut cmd = Command::new(ra);
        cmd.current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        lsp_log(&format!("starting rust-analyzer in {}", root.display()));
        let mut child = cmd.spawn().context("failed to start rust-analyzer")?;
        // Drain stderr in background to surface errors
        if let Some(es) = child.stderr.take() {
            std::thread::spawn(move || {
                let mut buf = String::new();
                let mut r = BufReader::new(es);
                while let Ok(n) = r.read_line(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let line = buf.trim_end();
                    eprintln!("[LSP-ERR] {}", line);
                    buf.clear();
                }
            });
        }
        Ok(child)
    }

    pub fn start(root: PathBuf) -> Result<Arc<Self>> {
        let mut child = Self::spawn_server(&root)?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow!("no stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("no stdout"))?;

        let (tx, rx) = unbounded::<Outgoing>();
        let caps = Arc::new(Mutex::new(None));

        let writer = Writer::start(stdin, rx.clone());
        let caps2 = caps.clone();
        let ready = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let queue: Arc<Mutex<Vec<Outgoing>>> = Arc::new(Mutex::new(Vec::new()));
        let init_ctx = InitCtx {
            tx: tx.clone(),
            ready: ready.clone(),
            queue: queue.clone(),
        };
        let reader = Reader::start(stdout, move |msg| {
            handle_incoming(msg, &caps2, Some(init_ctx.clone()))
        });

        let client = Arc::new(LspClient {
            child,
            tx,
            _writer: writer,
            _reader: reader,
            caps,
            ready,
            queue,
        });
        client.initialize(&root)?; // sends only initialize; initialized is sent upon response
        Ok(client)
    }

    fn initialize(&self, root: &Path) -> Result<()> {
        let id = REQ_ID.fetch_add(1, Ordering::Relaxed);
        let root_uri = Url::from_directory_path(root).map_err(|_| anyhow!("bad root uri"))?;
        let name = root
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| root.display().to_string());
        let init = lsp::InitializeParams {
            process_id: Some(std::process::id()),
            workspace_folders: Some(vec![lsp::WorkspaceFolder {
                uri: root_uri,
                name,
            }]),
            capabilities: client_caps(),
            ..Default::default()
        };
        lsp_log("sending initialize");
        self.tx
            .send(Outgoing::Request {
                id,
                method: "initialize".into(),
                params: serde_json::to_value(init)?,
            })
            .ok();
        Ok(())
    }

    pub fn did_open(&self, path: &Path, text: &str, version: i32) {
        if let Ok(uri) = Url::from_file_path(path) {
            let item = lsp::TextDocumentItem {
                uri,
                language_id: "rust".into(),
                version,
                text: text.into(),
            };
            let params = lsp::DidOpenTextDocumentParams {
                text_document: item,
            };
            lsp_log(&format!("didOpen {} v{version}", path.display()));
            if let Some(params) = to_json(&params) {
                self.send_or_queue(Outgoing::Notification {
                    method: "textDocument/didOpen".into(),
                    params,
                });
            } else {
                lsp_log("failed to serialize didOpen params");
            }
        }
    }

    pub fn did_change_full(&self, path: &Path, text: &str, version: i32) {
        if let Ok(uri) = Url::from_file_path(path) {
            let id = lsp::TextDocumentIdentifier { uri };
            let content = lsp::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.into(),
            };
            let params = lsp::DidChangeTextDocumentParams {
                text_document: lsp::VersionedTextDocumentIdentifier {
                    uri: id.uri,
                    version,
                },
                content_changes: vec![content],
            };
            lsp_log(&format!(
                "didChange {} v{version} ({} bytes)",
                path.display(),
                text.len()
            ));
            if let Some(params) = to_json(&params) {
                self.send_or_queue(Outgoing::Notification {
                    method: "textDocument/didChange".into(),
                    params,
                });
            } else {
                lsp_log("failed to serialize didChange params");
            }
        }
    }

    pub fn did_save(&self, path: &Path) {
        if let Ok(uri) = Url::from_file_path(path) {
            let id = lsp::TextDocumentIdentifier { uri };
            let params = lsp::DidSaveTextDocumentParams {
                text: None,
                text_document: id,
            };
            lsp_log(&format!("didSave {}", path.display()));
            if let Some(params) = to_json(&params) {
                self.send_or_queue(Outgoing::Notification {
                    method: "textDocument/didSave".into(),
                    params,
                });
            } else {
                lsp_log("failed to serialize didSave params");
            }
        }
    }

    pub fn did_close(&self, path: &Path) {
        if let Ok(uri) = Url::from_file_path(path) {
            let id = lsp::TextDocumentIdentifier { uri };
            let params = lsp::DidCloseTextDocumentParams { text_document: id };
            lsp_log(&format!("didClose {}", path.display()));
            if let Some(params) = to_json(&params) {
                self.send_or_queue(Outgoing::Notification {
                    method: "textDocument/didClose".into(),
                    params,
                });
            } else {
                lsp_log("failed to serialize didClose params");
            }
        }
    }

    pub fn completion<F>(&self, path: &Path, pos: lsp::Position, cb: F)
    where
        F: FnOnce(lsp::CompletionResponse) + Send + 'static,
    {
        if let Ok(uri) = Url::from_file_path(path) {
            let id = REQ_ID.fetch_add(1, Ordering::Relaxed);
            if let Ok(mut p) = pending().lock() {
                p.insert(id, Pending::Completion(Box::new(cb)));
            } else {
                lsp_log("failed to lock pending map for completion");
                return;
            }
            let params = lsp::CompletionParams {
                text_document_position: lsp::TextDocumentPositionParams {
                    text_document: lsp::TextDocumentIdentifier { uri },
                    position: pos,
                },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
                context: None,
            };
            lsp_log(&format!(
                "request completion id={id} at {}:{}",
                pos.line, pos.character
            ));
            if let Some(params) = to_json(&params) {
                self.send_or_queue(Outgoing::Request {
                    id,
                    method: "textDocument/completion".into(),
                    params,
                });
            } else {
                lsp_log("failed to serialize completion params");
            }
        }
    }

    fn send_or_queue(&self, msg: Outgoing) {
        if self.ready.load(Ordering::Relaxed) {
            let _ = self.tx.send(msg);
        } else {
            if let Ok(mut q) = self.queue.lock() {
                q.push(msg);
            } else {
                lsp_log("failed to lock queue; dropping message");
            }
        }
    }
}

fn client_caps() -> lsp::ClientCapabilities {
    lsp::ClientCapabilities {
        text_document: Some(lsp::TextDocumentClientCapabilities {
            synchronization: Some(lsp::TextDocumentSyncClientCapabilities {
                did_save: Some(true),
                will_save: Some(false),
                will_save_wait_until: Some(false),
                dynamic_registration: Some(false),
            }),
            completion: Some(lsp::CompletionClientCapabilities::default()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

struct Writer;
impl Writer {
    fn start(mut stdin: ChildStdin, rx: Receiver<Outgoing>) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                let payload = match msg {
                    Outgoing::Request { id, method, params } => serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "method": method,
                        "params": params,
                    }),
                    Outgoing::Notification { method, params } => serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": method,
                        "params": params,
                    }),
                };
                let body = match serde_json::to_string(&payload) {
                    Ok(s) => s,
                    Err(e) => {
                        lsp_log(&format!("writer serialize error: {e}"));
                        continue;
                    }
                };
                if let Some(m) = payload.get("method").and_then(|m| m.as_str()) {
                    let id_s = payload.get("id").and_then(|v| v.as_u64());
                    lsp_log(&format!(
                        "-> {}{} ({} bytes)",
                        m,
                        id_s.map(|i| format!(" #{i}")).unwrap_or_default(),
                        body.len()
                    ));
                }
                let header = format!("Content-Length: {}\r\n\r\n", body.len());
                if stdin.write_all(header.as_bytes()).is_err() {
                    lsp_log("writer header write error");
                    break;
                }
                if stdin.write_all(body.as_bytes()).is_err() {
                    lsp_log("writer body write error");
                    break;
                }
                let _ = stdin.flush();
            }
        })
    }
}

#[derive(Default)]
struct Reader;

type CompletionCb = Box<dyn FnOnce(lsp::CompletionResponse) + Send + 'static>;

enum Pending {
    Completion(CompletionCb),
}

static PENDING: OnceLock<Arc<Mutex<HashMap<u64, Pending>>>> = OnceLock::new();

fn pending() -> &'static Arc<Mutex<HashMap<u64, Pending>>> {
    PENDING.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

#[derive(Debug, Deserialize)]
struct RawMsg {
    #[serde(default)]
    jsonrpc: String,
    #[serde(default)]
    id: Option<JsonValue>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    params: Option<JsonValue>,
    #[serde(default)]
    result: Option<JsonValue>,
}

impl Reader {
    fn start(
        stdout: ChildStdout,
        mut on_msg: impl FnMut(RawMsg) + Send + 'static,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut content_length: Option<usize> = None;
                let mut line = String::new();
                // read headers
                loop {
                    line.clear();
                    if reader
                        .read_line(&mut line)
                        .ok()
                        .filter(|&n| n > 0)
                        .is_none()
                    {
                        lsp_log("<- eof from server");
                        return;
                    }
                    let l = line.trim_end();
                    if l.is_empty() {
                        break;
                    }
                    if let Some(rest) = l.strip_prefix("Content-Length:") {
                        content_length = rest.trim().parse::<usize>().ok();
                    }
                }
                let len = match content_length {
                    Some(l) => l,
                    None => {
                        lsp_log("<- missing Content-Length");
                        return;
                    }
                };
                let mut body = vec![0u8; len];
                if let Err(e) = reader.read_exact(&mut body) {
                    lsp_log(&format!("<- read body error: {e}"));
                    return;
                }
                lsp_log(&format!("<- {} bytes", len));
                let raw: RawMsg = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(e) => {
                        lsp_log(&format!("<- JSON parse error: {e}"));
                        continue;
                    }
                };
                on_msg(raw);
            }
        })
    }
}

#[derive(Clone)]
struct InitCtx {
    tx: Sender<Outgoing>,
    ready: Arc<std::sync::atomic::AtomicBool>,
    queue: Arc<Mutex<Vec<Outgoing>>>,
}

fn handle_incoming(
    raw: RawMsg,
    caps: &Arc<Mutex<Option<lsp::ServerCapabilities>>>,
    init_ctx: Option<InitCtx>,
) {
    if let Some(method) = raw.method.as_deref() {
        lsp_log(&format!("<- notif {method}"));
        match method {
            "window/logMessage" | "rust-analyzer/status" | "$/progress" => {
                // ignore for now
            }
            "textDocument/publishDiagnostics" => {
                if let Some(params) = raw.params {
                    if let Ok(p) = serde_json::from_value::<lsp::PublishDiagnosticsParams>(params) {
                        if let Ok(path) = p.uri.to_file_path() {
                            lsp_log(&format!(
                                "diagnostics {} items for {}",
                                p.diagnostics.len(),
                                path.display()
                            ));
                            diagnostics::update_diagnostics(&path, p.diagnostics);
                        }
                    }
                }
            }
            _ => {}
        }
        return;
    }
    if let (Some(idv), Some(result)) = (raw.id, raw.result) {
        if let Some(id) = idv.as_u64() {
            // handle known pending requests
            {
                if let Ok(mut pend_map) = pending().lock() {
                    if let Some(pend) = pend_map.remove(&id) {
                        match pend {
                            Pending::Completion(cb) => {
                                let resp: lsp::CompletionResponse =
                                    serde_json::from_value(result.clone())
                                        .unwrap_or(lsp::CompletionResponse::Array(vec![]));
                                let count = match &resp {
                                    lsp::CompletionResponse::Array(a) => a.len(),
                                    lsp::CompletionResponse::List(l) => l.items.len(),
                                };
                                lsp_log(&format!("<- response completion #{id} ({count} items)"));
                                cb(resp);
                            }
                        }
                    } else if let Ok(init) =
                        serde_json::from_value::<lsp::InitializeResult>(result.clone())
                    {
                        lsp_log("<- response initialize: storing capabilities");
                        if let Ok(mut c) = caps.lock() {
                            *c = Some(init.capabilities);
                        } else {
                            lsp_log("failed to lock caps");
                        }
                        if let Some(ctx) = init_ctx {
                            // Send initialized now, mark ready, and flush queue
                            let _ = ctx.tx.send(Outgoing::Notification {
                                method: "initialized".into(),
                                params: serde_json::json!({ "capabilities": {} }),
                            });
                            ctx.ready.store(true, Ordering::Relaxed);
                            if let Ok(mut q) = ctx.queue.lock() {
                                for m in q.drain(..) {
                                    let _ = ctx.tx.send(m);
                                }
                            } else {
                                lsp_log("failed to lock queue for flush");
                            }
                            lsp_log("initialized sent, queued messages flushed");
                            // Wake UI to refresh status line
                            app::awake();
                        }
                    } else {
                        lsp_log(&format!("<- response #{id} with unknown handler"));
                    }
                } else {
                    lsp_log("failed to lock pending map");
                }
            }
        }
    }
}

// ---- Global access ----

static CLIENT: OnceLock<Arc<LspClient>> = OnceLock::new();

pub fn init(root: PathBuf) {
    if disabled_by_env() {
        DISABLED.store(true, Ordering::Relaxed);
        AVAILABLE.store(false, Ordering::Relaxed);
        lsp_log("LSP disabled by environment");
        app::awake();
        return;
    }
    // Only start LSP if a Cargo.toml exists in the working directory
    if !root.join("Cargo.toml").exists() {
        AVAILABLE.store(false, Ordering::Relaxed);
        lsp_log(&format!(
            "LSP not started: missing Cargo.toml in {}",
            root.display()
        ));
        app::awake();
        return;
    }
    let start = || LspClient::start(root);
    match start() {
        Ok(c) => {
            lsp_log("LSP client started");
            AVAILABLE.store(true, Ordering::Relaxed);
            CLIENT.set(c).ok();
        }
        Err(err) => {
            lsp_log(&format!("LSP init failed: {err}"));
            AVAILABLE.store(false, Ordering::Relaxed);
        }
    }
}

pub fn with_client<F: FnOnce(&LspClient)>(f: F) {
    if let Some(c) = CLIENT.get() {
        f(c);
    } else {
        lsp_log("no LSP client available");
    }
}

pub fn is_ready() -> bool {
    if let Some(c) = CLIENT.get() {
        c.ready.load(Ordering::Relaxed)
    } else {
        false
    }
}

pub fn is_available() -> bool {
    AVAILABLE.load(Ordering::Relaxed) && !is_disabled()
}

pub fn status_text() -> String {
    if is_disabled() {
        "disabled".into()
    } else if !AVAILABLE.load(Ordering::Relaxed) {
        "unavailable".into()
    } else if is_ready() {
        "ready".into()
    } else {
        "starting".into()
    }
}

pub fn is_disabled() -> bool {
    DISABLED.load(Ordering::Relaxed)
}

fn disabled_by_env() -> bool {
    match std::env::var("RED_LSP_DISABLE") {
        Ok(v) => match v.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            _ => false,
        },
        Err(_) => false,
    }
}

// ---- Helpers ----

pub fn compute_position_from_offset(text: &str, offset: usize) -> lsp::Position {
    // Compute UTF-16 based line/character
    let mut line = 0u32;
    let mut col16 = 0u32;
    let mut count = 0usize;
    for ch in text.chars() {
        if count >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col16 = 0;
        } else {
            col16 += ch.len_utf16() as u32;
        }
        count += ch.len_utf8();
    }
    lsp::Position {
        line,
        character: col16,
    }
}

pub fn range_to_offsets(text: &str, range: lsp::Range) -> (usize, usize) {
    fn pos_to_off(text: &str, pos: lsp::Position) -> usize {
        let mut line = 0u32;
        let mut off = 0usize;
        for l in text.split_inclusive('\n') {
            if line == pos.line {
                let mut col16 = 0u32;
                let mut byte_off = 0usize;
                for ch in l.chars() {
                    if col16 >= pos.character {
                        break;
                    }
                    col16 += ch.len_utf16() as u32;
                    byte_off += ch.len_utf8();
                }
                off += byte_off;
                return off;
            }
            off += l.len();
            line += 1;
        }
        text.len()
    }
    (pos_to_off(text, range.start), pos_to_off(text, range.end))
}

fn to_json<T: Serialize>(v: &T) -> Option<JsonValue> {
    serde_json::to_value(v).ok()
}
