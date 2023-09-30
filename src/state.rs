#![allow(dead_code)]

use fltk::{
    app::{self, WidgetId},
    enums::*,
    group,
    prelude::*,
    text,
    utils::oncelock::Lazy,
};
use std::collections::HashMap;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};

static COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Debug, Default)]
pub struct Commands {
    idx: usize,
    pub cmds: Vec<String>,
}

impl Commands {
    pub fn push(&mut self, s: &str) {
        self.cmds.push(s.to_string());
        self.idx = self.cmds.len();
    }
}

#[derive(Clone, Debug)]
pub struct MyBuffer {
    pub modified: bool,
    pub id: String,
    pub buf: text::TextBuffer,
    pub current_file: Option<PathBuf>,
}

pub struct State {
    pub map: HashMap<usize, MyBuffer>,
    pub current_dir: PathBuf,
    pub cmds: Commands,
}

impl State {
    pub fn new(current_dir: PathBuf) -> Self {
        let map = HashMap::default();
        State { map, current_dir, cmds: Commands::default() }
    }
    pub fn append(&mut self, current_path: Option<PathBuf>) {
        let mut open = false;
        let mut tabs: group::Tabs = app::widget_from_id("tabs").unwrap();
        let mut edid = 0;
        for (k, v) in &self.map {
            if v.current_file == current_path {
                open = true;
                edid = *k;
                break;
            }
        }
        if !open {
            let old_count = COUNT.load(Ordering::Relaxed);
            let id = format!("edrow{}", old_count);
            COUNT.store(old_count + 1, Ordering::Relaxed);
            let mut buf = text::TextBuffer::default();
            buf.set_tab_distance(4);
            tabs.begin();
            let mut edrow = group::Flex::default()
                .row()
                .with_label(if let Some(current_path) = current_path.as_ref() {
                    if current_path.is_dir() {
                        "untitled"
                    } else {
                        current_path.file_name().unwrap().to_str().unwrap()
                    }
                } else {
                    "untitled"
                })
                .with_id(&id);
            edrow.set_trigger(CallbackTrigger::Closed);
            edrow.set_callback(crate::utils::tab_close_cb);
            let mut ed = text::TextEditor::default().with_id("ed");
            ed.set_buffer(buf.clone());
            crate::utils::init_editor(&mut ed);
            edrow.end();
            tabs.end();
            tabs.auto_layout();
            tabs.set_value(&edrow).ok();
            if let Some(p) = current_path.as_ref() {
                buf.load_file(p).ok();
                crate::highlight::highlight(p, &mut ed, &mut buf);
            }
            let mybuf = MyBuffer {
                modified: false,
                id,
                buf,
                current_file: current_path,
            };
            self.map.insert(ed.as_widget_ptr() as usize, mybuf);
        } else {
            unsafe {
                tabs.set_value(
                    &text::TextEditor::from_widget_ptr(edid as *mut _)
                        .parent()
                        .unwrap(),
                )
                .ok();
                tabs.set_damage(true);
            }
        }
    }
    pub fn current_id(&self) -> Option<usize> {
        let tabs: group::Tabs = app::widget_from_id("tabs").unwrap();
        if tabs.children() == 0 {
            return None;
        }
        tabs.value()
            .unwrap()
            .child(0)
            .map(|ed| ed.as_widget_ptr() as usize)
    }
    pub fn was_modified(&mut self, flag: bool) {
        let mut tabs: group::Tabs = app::widget_from_id("tabs").unwrap();
        if tabs.children() == 0 {
            return;
        }
        let mut edrow = tabs.value().unwrap();
        if let Some(c) = edrow.child(0) {
            let id = c.as_widget_ptr() as usize;
            let mybuf = self.map.get_mut(&id).unwrap();
            mybuf.modified = flag;
            if let Some(f) = mybuf.current_file.as_ref() {
                if flag {
                    edrow.set_label(&format!("\t{} *", f.file_name().unwrap().to_str().unwrap()));
                } else {
                    edrow.set_label(&format!("\t{}", f.file_name().unwrap().to_str().unwrap()));
                }
                tabs.redraw();
            }
        }
    }
    pub fn modified(&self) -> bool {
        if let Some(current_id) = self.current_id() {
            let mybuf = self.map.get(&current_id).unwrap();
            mybuf.modified
        } else {
            false
        }
    }
    pub fn buf(&self) -> Option<text::TextBuffer> {
        if let Some(current_id) = self.current_id() {
            let mybuf = self.map.get(&current_id).unwrap();
            Some(mybuf.buf.clone())
        } else {
            None
        }
    }
    pub fn current_file(&self) -> Option<PathBuf> {
        if let Some(current_id) = self.current_id() {
            let mybuf = self.map.get(&current_id).unwrap();
            mybuf.current_file.clone()
        } else {
            None
        }
    }
    pub fn set_current_file(&mut self, path: PathBuf) {
        if let Some(current_id) = self.current_id() {
            let mybuf = self.map.get_mut(&current_id).unwrap();
            mybuf.current_file = Some(path)
        }
    }
    pub fn current_editor(&self) -> Option<text::TextEditor> {
        let tabs: group::Tabs = app::widget_from_id("tabs").unwrap();
        if tabs.children() == 0 {
            return None;
        }
        tabs.value()
            .unwrap()
            .child(0)
            .map(|c| unsafe { c.into_widget() })
    }
}

pub static STATE: Lazy<app::GlobalState<State>> = Lazy::new(app::GlobalState::<State>::get);
