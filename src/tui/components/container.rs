// This file is part auf the greetly package.
//
// (c) 2020 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::iter;
use termion::cursor;

use std::io::Write;

use crate::tui::Draw;

use crate::draw2tty;

macro_rules! draw_horiz_line {
    ($tty:expr, $len:expr, ($x:expr, $y:expr), $start:expr, $center:expr, $end:expr) => {{
        let mut line = String::with_capacity($len as usize);
        line.push($start);
        for c in iter::repeat($center).take($len as usize - 2) {
            line.push(c);
        }
        line.push($end);
        draw2tty!($tty, "{}{}", cursor::Goto($x, $y), line);
    }};
}

macro_rules! draw_vert_line {
    ($tty:expr, $len:expr, ($x:expr, $y:expr), $center:expr) => {{
        draw2tty!($tty, "{}{}", cursor::Goto($x, $y), $center);
        for i in 1..$len {
            draw2tty!($tty, "{}{}", cursor::Goto($x, $y + i), $center);
        }
    }};
}

struct BorderCharacters {
    northwest: char,
    north: char,
    northeast: char,
    east: char,
    southeast: char,
    south: char,
    southwest: char,
    west: char,
}

impl BorderCharacters {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        northwest: char,
        north: char,
        northeast: char,
        east: char,
        southeast: char,
        south: char,
        southwest: char,
        west: char,
    ) -> Self {
        Self {
            northwest,
            north,
            northeast,
            east,
            southeast,
            south,
            southwest,
            west,
        }
    }
}

impl Default for BorderCharacters {
    fn default() -> Self {
        Self {
            northwest: ' ',
            north: ' ',
            northeast: ' ',
            east: ' ',
            southeast: ' ',
            south: ' ',
            southwest: ' ',
            west: ' ',
        }
    }
}

pub enum BorderType {
    //Plain,
    //Ascii,
    //Dos,
    //DosDouble,
    Unicode,
    //UnicodeBold,
    //UnicodeDouble,
}

pub struct Container {
    position: (u16, u16),
    size: (u16, u16),
    characters: BorderCharacters,
}

impl Container {
    pub fn new(border: BorderType, position: (u16, u16), size: (u16, u16)) -> Self {
        let characters = match border {
            //BorderType::Plain => BorderCharacters::default(),
            //BorderType::Ascii => BorderCharacters::new('+', '-', '+', '|', '+', '-', '+', '|'),
            //BorderType::Dos => BorderCharacters::new('┌', '─', '┐', '│', '┘', '─', '└', '│'),
            //BorderType::DosDouble => BorderCharacters::new('╔', '═', '╗', '║', '╝', '═', '╚', '║'),
            BorderType::Unicode => BorderCharacters::new('┌', '─', '┐', '│', '┘', '─', '└', '│'),
            //BorderType::UnicodeBold => BorderCharacters::new('┏', '━', '┓', '┃', '┛', '━', '┗', '┃'),
            //BorderType::UnicodeDouble => BorderCharacters::new('╔', '═', '╗', '║', '╝', '═', '╚', '║'),
        };

        Self {
            position,
            size,
            characters,
        }
    }
}

impl<T: Write> Draw<T> for Container {
    fn draw(&self, tty: &mut T, origin: (u16, u16), _focused: bool) {
        let (x, y) = (origin.0 + self.position.0, origin.1 + self.position.1);
        let (width, height) = self.size;
        let c = &self.characters;

        draw_horiz_line!(tty, width, (x, y), c.northwest, c.north, c.northeast);
        draw_vert_line!(tty, height - 2, (x, y + 1), c.west);
        draw_vert_line!(tty, height - 2, (x + width - 1, y + 1), c.east);
        draw_horiz_line!(
            tty,
            width,
            (x, y + height - 1),
            c.southwest,
            c.south,
            c.southeast
        );
    }
}
