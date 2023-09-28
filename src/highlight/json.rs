use fltk::{enums::*, *};
use json_tools::{Buffer, BufferType, Lexer, Span, TokenType};

pub fn styles() -> Vec<text::StyleTableEntry> {
    vec![
        text::StyleTableEntry {
            color: Color::Red,
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::from_hex(0x5dafef),
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::Black,
            font: Font::Courier,
            size: 14,
        },
        text::StyleTableEntry {
            color: Color::Green.darker(),
            font: Font::Courier,
            size: 14,
        },
    ]
}

pub fn apply(s: &str, sbuf: &mut text::TextBuffer) {
    let mut local_buf = vec![b'A'; s.len()];
    for token in Lexer::new(s.bytes(), BufferType::Span) {
        use TokenType::*;
        let c = match token.kind {
            CurlyOpen | CurlyClose | BracketOpen | BracketClose | Colon | Comma | Invalid => 'A',
            String => 'B',
            BooleanTrue | BooleanFalse | Null => 'C',
            Number => 'D',
        };
        if let Buffer::Span(Span { first, end }) = token.buf {
            let start = first as _;
            let last = end as _;
            local_buf[start..last].copy_from_slice(c.to_string().repeat(last - start).as_bytes());
        }
    }
    sbuf.set_text(&String::from_utf8_lossy(&local_buf));
}
