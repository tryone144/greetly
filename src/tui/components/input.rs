// This file is part auf the greetly package.
//
// (c) 2020 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::cmp::{max, min};
use std::iter;
use termion::color;
use termion::cursor;
use termion::style;

use std::fmt;
use std::io::Write;
use std::iter::FromIterator;

use crate::tui::{Draw, KeyboardInput};

use crate::draw2tty;

pub struct TextInput<T> {
    data: Vec<char>,
    vis_len: usize,
    vis_start: usize,
    cursor: usize,
    masked: bool,
    is_secret: bool,
    position: (u16, u16),
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TextInput<T> {
    pub fn new(length: usize, is_secret: bool, position: (u16, u16)) -> Self {
        Self {
            data: Vec::with_capacity(length),
            vis_len: length,
            vis_start: 0,
            cursor: 0,
            masked: is_secret,
            is_secret,
            position,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn set_secret(&mut self, is_secret: bool) {
        self.is_secret = is_secret;
        self.masked = is_secret;
    }

    fn toggle(&mut self) {
        if self.is_secret {
            self.masked = !self.masked;
        }
    }

    fn set_cursor(&mut self, position: usize) {
        self.cursor = min(position, self.data.len());
        self.refresh_visible();
    }

    fn move_cursor(&mut self, positions: i32) {
        self.cursor = min(
            max(0, self.cursor as i32 + positions) as usize,
            self.data.len(),
        );
        self.refresh_visible();
    }

    fn refresh_visible(&mut self) {
        if self.cursor < 2 {
            self.vis_start = 0;
        } else if self.cursor - 2 < self.vis_start {
            self.vis_start = self.cursor - 2;
        } else if self.cursor + 1 > self.vis_start + self.vis_len {
            self.vis_start = self.cursor - self.vis_len + 1;
        }
    }
}

impl<T> fmt::Display for TextInput<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_iter(self.data.iter()))
    }
}

impl<T: Write> KeyboardInput<T> for TextInput<T> {
    fn clear(&mut self) {
        self.is_secret = false;
        self.masked = false;

        self.data.clear();
        self.set_cursor(0);
    }

    fn push(&mut self, c: char) {
        if self.cursor >= self.data.len() {
            self.data.push(c);
        } else {
            self.data.insert(self.cursor, c);
        }
        self.move_cursor(1);
    }

    fn pop(&mut self, right: bool) {
        if right {
            if self.cursor < self.data.len() {
                self.data.remove(self.cursor);
            }
            return;
        }

        if self.cursor >= self.data.len() {
            self.data.pop();
        } else if self.cursor > 0 {
            self.data.remove(self.cursor - 1);
        }
        self.move_cursor(-1);
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn toggle(&mut self) {
        self.toggle();
    }

    fn cursor_left(&mut self) {
        self.move_cursor(-1);
    }

    fn cursor_right(&mut self) {
        self.move_cursor(1);
    }

    fn cursor_start(&mut self) {
        self.set_cursor(0);
    }

    fn cursor_end(&mut self) {
        self.set_cursor(self.data.len());
    }
}

impl<T: Write> Draw<T> for TextInput<T> {
    fn draw(&self, tty: &mut T, origin: (u16, u16), focused: bool) {
        let length = min(self.data.len() - self.vis_start, self.vis_len);
        let (x, y) = (origin.0 + self.position.0, origin.1 + self.position.1);

        let text = if self.masked {
            String::from_iter(iter::repeat('*').take(length))
        } else {
            String::from_iter(self.data.iter().skip(self.vis_start).take(length))
        };

        if focused {
            draw2tty!(tty, "{}", style::Bold);
        }
        draw2tty!(
            tty,
            "{}{}{3: <2$}",
            cursor::Goto(x, y),
            text,
            self.vis_len - length,
            ""
        );

        if self.vis_start > 0 {
            draw2tty!(
                tty,
                "{}{}{}{}",
                cursor::Goto(x, y),
                color::Fg(color::LightBlack),
                "<",
                color::Fg(color::Reset)
            );
        }
        if self.data.len() > self.vis_start + self.vis_len {
            draw2tty!(
                tty,
                "{}{}{}{}",
                cursor::Goto(x + self.vis_len as u16 - 1, y),
                color::Fg(color::LightBlack),
                ">",
                color::Fg(color::Reset)
            );
        }

        if focused {
            let cursor = x as usize + self.cursor - self.vis_start;
            draw2tty!(
                tty,
                "{}{}{}",
                style::Reset,
                cursor::Goto(cursor as u16, y),
                cursor::Save
            );
        }
    }
}
