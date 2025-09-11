use crate::state::STATE;
use fltk::{prelude::*, *};
use lsp_types as lsp;

#[derive(Clone, Debug)]
pub struct CompletionEntry {
    pub label: String,
    pub insert_text: String,
    pub edit_range: Option<lsp::Range>,
    pub kind: Option<lsp::CompletionItemKind>,
    pub detail: Option<String>,
}

pub fn show_popup(ed: &text::TextEditor, entries: Vec<CompletionEntry>) {
    if entries.is_empty() {
        return;
    }
    // Use the pre-spawned dialog created in dialogs::CompletionDialog::new()
    if let (Some(mut win), Some(mut list)) = (
        app::widget_from_id::<window::Window>("completion"),
        app::widget_from_id::<browser::HoldBrowser>("completion_list"),
    ) {
        list.clear();
        for e in &entries {
            list.add(&format_item(e));
        }
        // Selection handler: double-click or Space/Enter (Space only without Ctrl)
        use std::sync::Arc;
        let entries_arc = Arc::new(entries);
        let space_or_enter = || {
            if app::event() != enums::Event::KeyDown {
                return false;
            }
            let k = app::event_key();
            let mods = app::event_state();
            (k == enums::Key::Enter || k == enums::Key::KPEnter)
                || (k == enums::Key::from_char(' ') && !mods.contains(enums::Shortcut::Ctrl))
        };
        let mut win_handle = win.clone();
        list.set_callback(move |b| {
            let released = app::event() == enums::Event::Released && app::event_clicks();
            let enter = space_or_enter();
            if released || enter {
                let idx = b.value();
                if idx > 0 {
                    let ent = entries_arc[idx as usize - 1].clone();
                    insert_entry_owned(ent);
                    win_handle.hide();
                }
            }
        });
        // Handle Space/Enter on the list as well
        list.handle(move |l, ev| match ev {
            enums::Event::KeyDown => {
                if space_or_enter() {
                    l.do_callback();
                    true
                } else {
                    false
                }
            }
            _ => false,
        });
        // No extra key handler; Enter handled via callback above
        // Position near caret and show
        let pos = ed.insert_position();
        let (mut ex, mut ey) = ed.position_to_xy(pos);
        ex += 8;
        ey += ed.text_size() + 42;
        win.set_pos(ex, ey);
        win.show();
        if list.size() > 0 {
            list.select(1);
        }
        let _ = list.take_focus();
    }
}

fn insert_entry_owned(ent: CompletionEntry) {
    // Avoid holding STATE borrow across buffer.replace(), since replace()
    // triggers editor callbacks which also touch STATE, leading to reentrancy.
    let CompletionEntry {
        insert_text,
        edit_range,
        ..
    } = ent;
    let (mut buf_opt, start, end) = STATE.with(move |s| {
        if let Some(ed) = s.current_editor() {
            if let Some(v) = s.map.get(&(ed.as_widget_ptr() as usize)) {
                if v.current_file.is_some() {
                    let b = v.buf.clone();
                    let mut start = ed.insert_position();
                    let mut end = start;
                    if let Some(r) = edit_range {
                        let text = b.text();
                        let (s_off, e_off) = crate::lsp::range_to_offsets(&text, r);
                        start = s_off as i32;
                        end = e_off as i32;
                    }
                    return (Some(b), start, end);
                }
            }
        }
        (None, 0, 0)
    });
    if let Some(ref mut b) = buf_opt {
        // Perform replacement first
        b.replace(start, end, &insert_text);
        // Then move the caret to the end of the inserted text
        let new_pos = start.saturating_add(insert_text.len() as i32);
        STATE.with(move |s| {
            if let Some(mut ed) = s.current_editor() {
                ed.set_insert_position(new_pos);
                let _ = ed.take_focus();
            }
        });
    }
}

fn kind_to_short(k: lsp::CompletionItemKind) -> String {
    format!("{:?}", k).to_lowercase().to_string()
}

fn format_item(e: &CompletionEntry) -> String {
    match (e.kind.map(kind_to_short), e.detail.as_deref()) {
        (Some(kind), Some(detail)) => format!("{:<10} {} — {}", kind, e.label, detail),
        (Some(kind), None) => format!("{:<10} {}", kind, e.label),
        (None, Some(detail)) => format!("{:<10} {} — {}", "", e.label, detail),
        (None, None) => e.label.clone(),
    }
}
