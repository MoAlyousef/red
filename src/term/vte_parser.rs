use fltk::prelude::DisplayExt;
use vte::{Params, Perform};

macro_rules! debug {
    ($($e:expr),+) => {
        {
            #[cfg(feature="debug-term")]
            {
                eprintln!($($e),+)
            }
            #[cfg(not(feature="debug-term"))]
            {
                ($($e),+)
            }
        }
    };
}

impl Perform for super::VteParser {
    fn print(&mut self, c: char) {
        self.temp_s.push(c);
        self.temp_b.push(self.ch);
    }

    fn execute(&mut self, byte: u8) {
        debug!("{}", byte);
        match byte {
            8 => {
                // backspace
                let mut buf = self.st.buffer().unwrap();
                if let Some(ch) = buf.text().chars().last() {
                    let mut temp = [0u8; 4];
                    let s = ch.encode_utf8(&mut temp);
                    buf.remove(buf.length() - s.len() as i32, buf.length());
                    self.sbuf.remove(buf.length() - 1, buf.length());
                }
            }
            10 | 13 => {
                // crlf
                self.temp_s.push(byte as char);
                self.temp_b.push(self.ch);
            }
            0 | 7 => (), // tabs?
            _ => (),
        }
    }

    fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        debug!(
            "[hook] params={:?}, intermediates={:?}, ignore={:?}, char={:?}",
            params, intermediates, ignore, c
        );
    }

    fn put(&mut self, byte: u8) {
        debug!("[put] {:02x}", byte);
    }

    fn unhook(&mut self) {
        debug!("[unhook]");
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        debug!(
            "[osc_dispatch] params={:?} bell_terminated={}",
            params, bell_terminated
        );
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        debug!(
            "[csi_dispatch] params={:#?} intermediates={:?}, ignore={:?}, char={}",
            params, intermediates, ignore, c
        );
        match c {
            'm' => {
                let mut bold = false;
                for p in params {
                    let p = p[0];
                    if p == 1 {
                        bold = true;
                        continue;
                    }
                    if p > 30 && p < 40 {
                        if bold {
                            self.ch = (p + 35 + 9) as u8 as char;
                        } else {
                            self.ch = (p + 35) as u8 as char;
                        }
                    } else {
                        self.ch = 'A';
                    }
                }
            }
            'K' => {
                for p in params {
                    match p[0] {
                        0 => {
                            let mut buf = self.st.buffer().unwrap();
                            let len = buf.length();
                            let pos = self.st.insert_position();
                            buf.remove(pos, len);
                            self.sbuf.remove(pos, len);
                        }
                        _ => {}
                    }
                }
            }
            'C' => {
                for p in params {
                    match p[0] {
                        0 => {}
                        _ => {}
                    }
                }
            }
            'H' => {
                if params.len() > 1 {
                    let mut iter = params.iter();
                    let _row = iter.next().unwrap()[0] as i32;
                    let _col = iter.next().unwrap()[0] as i32;
                } else {
                }
            }
            'J' => {
                for p in params {
                    match p[0] {
                        2 => {
                            self.st.buffer().unwrap().set_text("");
                            self.st.style_buffer().unwrap().set_text("");
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        debug!(
            "[esc_dispatch] intermediates={:?}, ignore={:?}, byte={:02x}",
            intermediates, ignore, byte
        );
    }
}
