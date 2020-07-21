// This file is part auf the greetly package.
//
// (c) 2020 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::fmt;
use std::io;

pub mod components;
mod greeter;

pub use greeter::TerminalUI;

#[macro_export]
macro_rules! draw2tty {
    ($tty:expr, $($args:expr),+) => {
        write!($tty, $($args),+).expect("drawing to tty failed")
    };
}

#[derive(Debug)]
pub enum LoginAction {
    Quit,
    Cancel,
    Submit(String),
}

#[derive(Debug)]
pub enum LoginError {
    NotATTY,
    IoError(io::Error),
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotATTY => write!(f, "Stdout is not a TTY"),
            Self::IoError(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for LoginError {
    fn from(err: io::Error) -> Self {
        LoginError::IoError(err)
    }
}

pub trait GreetUI {
    fn set_prompt(&mut self, prompt: &str);
    fn set_secret_prompt(&mut self, prompt: &str);
    fn show_info_message(&mut self, message: &str);
    fn show_error_message(&mut self, message: &str);
    fn show_authentication_failure(&mut self, reason: &str);
}

trait Draw<T: io::Write> {
    fn draw(&self, tty: &mut T, base: (u16, u16), focused: bool);
}

trait KeyboardInput<T: io::Write>: Draw<T> {
    fn clear(&mut self);
    fn push(&mut self, c: char);
    fn pop(&mut self, right: bool);
    fn is_empty(&self) -> bool;
    fn toggle(&mut self);
    fn cursor_left(&mut self);
    fn cursor_right(&mut self);
    fn cursor_start(&mut self);
    fn cursor_end(&mut self);
}
