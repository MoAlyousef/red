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
}

impl State {
    pub fn new(
        ed: &text::TextEditor,
        buf: text::TextBuffer,
        current_dir: PathBuf,
        current_file: Option<PathBuf>,
        id: &'static str,
    ) -> Self {
        let mybuf = MyBuffer {
            modified: false,
            id: id.to_string(),
            buf,
            current_file,
        };
        let mut map = HashMap::default();
        map.insert(ed.as_widget_ptr() as usize, mybuf);
        State { map, current_dir }
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
            if let Some(p) = current_path.as_ref() {
                buf.load_file(p).ok();
            }
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
            ed.set_linenumber_width(40);
            ed.set_text_font(Font::Courier);
            ed.set_buffer(buf.clone());
            ed.set_trigger(CallbackTrigger::Changed);
            ed.set_callback(crate::utils::editor_cb);
            edrow.end();
            tabs.end();
            tabs.auto_layout();
            tabs.set_value(&edrow).ok();
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
                app::redraw();
            }
        }
    }
    pub fn current_id(&self) -> usize {
        let tabs: group::Tabs = app::widget_from_id("tabs").unwrap();
        // TODO: Check if tabs empty
        let ed = tabs.value().unwrap().child(0).unwrap();
        ed.as_widget_ptr() as usize
    }
    pub fn was_modified(&mut self, flag: bool) {
        let current_id = self.current_id();
        let mybuf = self.map.get_mut(&current_id).unwrap();
        mybuf.modified = flag;
        if mybuf.current_file.is_some() {
            if !flag {
                app::first_window().unwrap().set_label(&format!(
                    "{} - RustyEd",
                    mybuf.current_file.as_ref().unwrap().display()
                ));
            } else {
                app::first_window().unwrap().set_label(&format!(
                    "*{} - RustyEd",
                    mybuf.current_file.as_ref().unwrap().display()
                ));
            }
        }
    }
    pub fn modified(&self) -> bool {
        let current_id = self.current_id();
        let mybuf = self.map.get(&current_id).unwrap();
        mybuf.modified
    }
    pub fn buf(&self) -> text::TextBuffer {
        let current_id = self.current_id();
        let mybuf = self.map.get(&current_id).unwrap();
        mybuf.buf.clone()
    }
    pub fn current_file(&self) -> Option<PathBuf> {
        let current_id = self.current_id();
        let mybuf = self.map.get(&current_id).unwrap();
        mybuf.current_file.clone()
    }
    pub fn set_current_file(&mut self, path: PathBuf) {
        let current_id = self.current_id();
        let mybuf = self.map.get_mut(&current_id).unwrap();
        mybuf.current_file = Some(path)
    }
    pub fn current_editor(&self) -> text::TextEditor {
        let tabs: group::Tabs = app::widget_from_id("tabs").unwrap();
        // TODO: Check if tabs empty
        unsafe { tabs.value().unwrap().child(0).unwrap().into_widget() }
    }
}

pub static STATE: Lazy<app::GlobalState<State>> = Lazy::new(app::GlobalState::<State>::get);
