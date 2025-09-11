use fltk::text::TextBuffer;
use fltk::{app, frame, prelude::WidgetExt};
use lsp_types as lsp;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::state::STATE;
use fltk::enums::Align;

#[derive(Default, Debug)]
struct DiagState {
    // Registered style buffers for open files to allow overlay adjustments
    styles: HashMap<PathBuf, (TextBuffer, usize)>, // (style_sbuf, base_styles_count)
    // Pending diagnostics, stored until applied on main thread
    pending: VecDeque<PathBuf>,
    // Latest diagnostics per file (LSP ranges)
    diags: HashMap<PathBuf, Vec<lsp::Diagnostic>>,
}

static DIAG_STATE: OnceLock<Mutex<DiagState>> = OnceLock::new();

fn with_state<F: FnOnce(&mut DiagState)>(f: F) {
    let m = DIAG_STATE.get_or_init(|| Mutex::new(DiagState::default()));
    if let Ok(mut s) = m.lock() {
        f(&mut s);
    }
}

pub fn register_style_buf(path: &Path, style_buf: &TextBuffer, base_styles: usize) {
    with_state(|s| {
        s.styles
            .insert(path.to_path_buf(), (style_buf.clone(), base_styles));
    });
}

pub fn update_diagnostics(path: &Path, diags: Vec<lsp::Diagnostic>) {
    crate::lsp::lsp_log(&format!(
        "diag update queued {} ({} items)",
        path.display(),
        diags.len()
    ));
    with_state(|s| {
        s.diags.insert(path.to_path_buf(), diags);
        s.pending.push_back(path.to_path_buf());
    });
    // Wake GUI to apply changes on the main thread
    app::awake();
}

// Call once at GUI startup to install a wake handler
pub fn install_awake_handler() {
    static INSTALLED: OnceLock<()> = OnceLock::new();
    if INSTALLED.set(()).is_ok() {
        app::awake_callback(|| {
            update_status_bar();
            apply_pending();
        });
        // Also poll periodically so diagnostics still apply even if another
        // part of the app replaces the global awake callback.
        fn schedule_diag_poll() {
            app::add_timeout3(0.2, |_| {
                apply_pending();
                schedule_diag_poll();
            });
        }
        schedule_diag_poll();
    }
}

fn apply_pending() {
    // Drain queue, dedupe, and apply overlays
    let mut items: Vec<PathBuf> = Vec::new();
    with_state(|s| {
        while let Some(p) = s.pending.pop_front() {
            items.push(p);
        }
    });
    let mut seen = HashSet::new();
    for p in items {
        if seen.insert(p.clone()) {
            apply_for_path(&p);
        }
    }
}

fn apply_for_path(path: &Path) {
    let pbuf = path.to_path_buf();
    // Need both style buffer and source text
    let collected = with_collect(move |s| {
        let (sb, bs) = match s.styles.get(&pbuf) {
            Some(v) => (v.0.clone(), v.1),
            None => return None,
        };
        let diags = s.diags.get(&pbuf).cloned().unwrap_or_default();
        let text = STATE.with(move |st| {
            // Try to get the buffer for this path
            for v in st.map.values() {
                if v.current_file
                    .as_ref()
                    .map(|pp| pp.as_path() == pbuf.as_path())
                    .unwrap_or(false)
                {
                    return v.buf.text();
                }
            }
            String::new()
        });
        Some((sb, bs, diags, text))
    });
    if let Some((mut sb, base_styles, diags, text)) = collected {
        let mut styles = sb.text();
        if styles.is_empty() || text.is_empty() {
            return;
        }
        // Clear existing underlines by normalizing any underlined style back to base (keep color)
        {
            let bytes = unsafe { styles.as_bytes_mut() };
            normalize_styles(bytes, base_styles);
            // Apply error underlines (severity 1)
            for d in diags.iter() {
                if matches!(d.severity, Some(lsp::DiagnosticSeverity::ERROR))
                    || d.severity.is_none()
                {
                    if let (Some(start), Some(end)) = (
                        pos_to_offset(&text, d.range.start),
                        pos_to_offset(&text, d.range.end),
                    ) {
                        let len = bytes.len();
                        let sidx = start.min(len);
                        let eidx = end.min(len);
                        underline_range(bytes, sidx, eidx, base_styles);
                    }
                }
            }
        }
        sb.set_text(&styles);
        // Refresh footer to reflect current LSP status and errors
        update_status_bar();
    }
}

fn with_collect<T, F: FnOnce(&mut DiagState) -> Option<T>>(f: F) -> Option<T> {
    let m = DIAG_STATE.get_or_init(|| Mutex::new(DiagState::default()));
    if let Ok(mut s) = m.lock() {
        f(&mut s)
    } else {
        None
    }
}

fn normalize_styles(bytes: &mut [u8], base_styles: usize) {
    let base = 'A' as u32;
    let total = base_styles * 2;
    for ch in bytes.iter_mut() {
        let c = *ch as char;
        let idx = (c as u32).saturating_sub(base) as usize;
        if idx < total && idx >= base_styles {
            // map underlined variant back to base
            let new_idx = idx - base_styles;
            *ch = (base + new_idx as u32) as u8;
        }
    }
}

fn underline_range(bytes: &mut [u8], start: usize, end: usize, base_styles: usize) {
    let base = 'A' as u32;
    let total = base_styles * 2;
    let end = end.min(bytes.len());
    for b in bytes.iter_mut().take(end).skip(start) {
        let c = *b as char;
        let idx = (c as u32).saturating_sub(base) as usize;
        if idx < base_styles {
            let new_idx = idx + base_styles;
            if new_idx < total {
                *b = (base + new_idx as u32) as u8;
            }
        }
    }
}

fn pos_to_offset(text: &str, pos: lsp::Position) -> Option<usize> {
    // Convert UTF-16 based Position into byte offset in `text`
    let mut line = 0u32;
    let mut off = 0usize;
    for (line_idx, ltxt) in text.split_inclusive('\n').enumerate() {
        if line == pos.line {
            // compute col in this line
            let mut col16 = 0u32;
            let mut byte_off = 0usize;
            for ch in ltxt.chars() {
                if col16 >= pos.character {
                    break;
                }
                col16 += ch.len_utf16() as u32;
                byte_off += ch.len_utf8();
            }
            off += byte_off;
            return Some(off);
        }
        off += ltxt.len();
        line = line_idx as u32 + 1;
    }
    // If target line is after last, clamp to end
    Some(text.len())
}

// Update footer status line based on current directory, LSP status, and errors (if available)
pub fn update_status_bar() {
    let (dir_text, lsp_status, err_opt): (String, String, Option<usize>) = STATE.with(|st| {
        let dir = st.current_dir.display().to_string();
        let lsp_status = crate::lsp::status_text();
        let mut err_count: Option<usize> = None;
        if let Some(path) = st.current_file() {
            with_state(|ds| {
                if let Some(diags) = ds.diags.get(&path) {
                    let errs = diags
                        .iter()
                        .filter(|d| {
                            d.severity
                                .map(|s| s == lsp::DiagnosticSeverity::ERROR)
                                .unwrap_or(true)
                        })
                        .count();
                    err_count = Some(errs);
                }
            });
        }
        (dir, lsp_status, err_count)
    });
    if let Some(mut info) = app::widget_from_id::<frame::Frame>("info") {
        let label = match err_opt {
            Some(n) => format!(
                "Directory: {}   |   LSP: {}   |   Errors: {}",
                dir_text, lsp_status, n
            ),
            None => format!("Directory: {}   |   LSP: {}", dir_text, lsp_status),
        };
        info.set_label(&label);
        info.set_align(Align::Left | Align::Inside);
    }
}
