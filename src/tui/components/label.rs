// This file is part auf the greetly package.
//
// (c) 2020 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use termion::cursor;
use termion::style;

use std::io::Write;

use crate::tui::Draw;

use crate::draw2tty;

fn truncate_text(text: &str, length: Option<usize>) -> String {
    match length {
        None | Some(0) => text.to_owned(),
        Some(len) => {
            let mut truncated = text.chars().take(len).collect::<String>();
            if text.chars().count() > len {
                truncated.pop();
                truncated.push('…');
            }

            truncated
        }
    }
}

pub struct Label {
    text: String,
    position: (u16, u16),
    length: Option<usize>,
}

impl Label {
    pub fn new(text: &str, position: (u16, u16)) -> Self {
        Self::new_truncated(text, position, 0)
    }

    pub fn new_truncated(text: &str, position: (u16, u16), length: usize) -> Self {
        let length = match length {
            0 => None,
            _ => Some(length),
        };

        Self {
            text: truncate_text(text, length),
            position,
            length,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = truncate_text(text, self.length);
    }
}

impl<T: Write> Draw<T> for Label {
    fn draw(&self, tty: &mut T, origin: (u16, u16), focused: bool) {
        let (x, y) = (origin.0 + self.position.0, origin.1 + self.position.1);

        if focused {
            draw2tty!(
                tty,
                "{}{}{}{}",
                cursor::Goto(x, y),
                style::Bold,
                self.text,
                style::Reset
            );
        } else {
            draw2tty!(tty, "{}{}", cursor::Goto(x, y), self.text);
        }
    }
}
