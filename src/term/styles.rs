use fltk::{
    enums::{Color, Font},
    text::StyleTableEntry,
};

pub(crate) fn init() -> Vec<StyleTableEntry> {
    vec![
        StyleTableEntry {
            color: Color::White,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Red,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Green,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Yellow,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            color: Color::from_hex(0x61afef),
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Magenta,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Cyan,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            color: Color::White,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Foreground,
            font: Font::Courier,
            size: 14,
        },
        StyleTableEntry {
            color: Color::White,
            font: Font::CourierBold,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Red,
            font: Font::CourierBold,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Green,
            font: Font::CourierBold,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Yellow,
            font: Font::CourierBold,
            size: 14,
        },
        StyleTableEntry {
            color: Color::from_hex(0x61afef),
            font: Font::CourierBold,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Magenta,
            font: Font::CourierBold,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Cyan,
            font: Font::CourierBold,
            size: 14,
        },
        StyleTableEntry {
            color: Color::White,
            font: Font::CourierBold,
            size: 14,
        },
        StyleTableEntry {
            color: Color::Foreground,
            font: Font::CourierBold,
            size: 14,
        },
    ]
}
