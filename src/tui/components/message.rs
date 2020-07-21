// This file is part auf the greetly package.
//
// (c) 2020 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::cmp::{max, min};

use crate::tui::components::{BorderType, Container, Label};

use std::io::Write;

use crate::tui::Draw;

const MIN_WIDTH: u16 = 40;
const MAX_WIDTH: u16 = 80;

fn split_line(line: &str) -> (&str, Option<&str>) {
    if line.chars().count() > MAX_WIDTH as usize {
        let mut last_word = 0;

        for (idx, (byte, chr)) in line.char_indices().enumerate() {
            if chr.is_ascii_whitespace() {
                last_word = byte;
            }
            if idx > MAX_WIDTH as usize {
                let (l, rest) = line.split_at(last_word);
                return (l.trim_end(), Some(rest.trim_start()));
            }
        }
    }

    (line, None)
}

pub struct Message {
    position: (u16, u16),
    size: (u16, u16),
    label: Label,
    lines: Vec<Label>,
    container: Container,
}

impl Message {
    pub fn new(title: &str, message: &str, position: (u16, u16)) -> Self {
        let message_lines: Vec<_> = message.lines().collect();

        let title_len = title.chars().count();
        let message_len = message_lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .expect("Message has no lines");

        let label = Label::new_truncated(title, (2, 1), MAX_WIDTH as usize);

        let mut lines = Vec::with_capacity(message_lines.len());
        for line in message_lines.iter() {
            let mut split = ("", Some(*line));
            while let Some(rest) = split.1 {
                split = split_line(rest);
                let message = Label::new(split.0, (2, lines.len() as u16 + 2));
                lines.push(message);
            }
        }

        let width = min(
            max(MIN_WIDTH, max(title_len, message_len) as u16),
            MAX_WIDTH,
        );
        let height = lines.len() as u16 + 1;

        let size = (width + 4, height + 2);
        let container = Container::new(BorderType::Unicode, (0, 0), size);

        Self {
            position,
            size,
            label,
            lines,
            container,
        }
    }

    pub fn height(&self) -> u16 {
        self.size.1
    }
}

impl<T: Write> Draw<T> for Message {
    fn draw(&self, tty: &mut T, origin: (u16, u16), focused: bool) {
        let new_origin = (origin.0 + self.position.0, origin.1 + self.position.1);

        self.container.draw(tty, new_origin, focused);
        self.label.draw(tty, new_origin, true);
        for message in self.lines.iter() {
            message.draw(tty, new_origin, false);
        }
    }
}
