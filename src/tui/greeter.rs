// This file is part auf the greetly package.
//
// (c) 2020 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use std::io;
use termion::clear;
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::raw;

use crate::tui::components::{FormElement, LoginForm};
use crate::tui::{LoginAction, LoginError};

use crate::tui::components::Message;

use std::io::Write;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use crate::tui::{Draw, GreetUI, KeyboardInput};

use crate::draw2tty;

pub struct TerminalUI<T: Write> {
    tty: cursor::HideCursor<T>,
    login_form: LoginForm<T>,
    messages: Vec<Message>,
}

impl TerminalUI<raw::RawTerminal<io::Stdout>> {
    pub fn init() -> Result<Self, LoginError> {
        let stdout = io::stdout();
        if !termion::is_tty(&stdout) {
            return Err(LoginError::NotATTY);
        }

        let tty = stdout.into_raw_mode().map(cursor::HideCursor::from)?;

        let (width, height) = termion::terminal_size().unwrap();
        let login_form = LoginForm::new(((width - 64) / 2, (height - 11) / 2), (64, 11));

        let messages = Vec::with_capacity(3);

        let mut ui = Self {
            tty,
            login_form,
            messages,
        };

        ui.clear();
        ui.redraw();

        Ok(ui)
    }

    pub fn handle_input(&mut self) -> Result<LoginAction, LoginError> {
        let stdin = io::stdin();
        let stdin = stdin.lock();

        self.redraw();

        for evt in stdin.keys() {
            let key = evt?;

            match key {
                Key::Esc => {
                    // DEBUG: Exit
                    break;
                }
                Key::F(1) => {
                    // DEBUG: Clear and Redraw
                    self.reset();
                    return Ok(LoginAction::Cancel);
                }
                Key::F(3) => {
                    self.login_form.toggle();
                }
                Key::Left => {
                    self.login_form.cursor_left();
                }
                Key::Right => {
                    self.login_form.cursor_right();
                }
                Key::Up => {
                    self.login_form.focus_prev();
                }
                Key::Down => {
                    self.login_form.focus_next();
                }
                Key::Home => {
                    self.login_form.cursor_start();
                }
                Key::End => {
                    self.login_form.cursor_end();
                }
                Key::Char(c) => {
                    if !c.is_control() {
                        self.login_form.push(c);
                    }
                    match c {
                        '\t' => {} //self.login_form.focus_next(),
                        '\n' => match self.login_form.focus() {
                            FormElement::Prompt => {
                                return Ok(LoginAction::Submit(self.login_form.to_string()));
                            }
                        },
                        _ => {}
                    };
                }
                Key::Backspace => {
                    self.login_form.pop(false);
                }
                Key::Delete => {
                    self.login_form.pop(true);
                }
                _ => {}
            };

            self.redraw();
        }

        Ok(LoginAction::Quit)
    }
}

impl<T: Write> TerminalUI<T> {
    pub fn reset(&mut self) {
        self.clear();
        self.messages.clear();
        self.login_form.reset();
    }

    fn add_message(&mut self, title: &str, message: &str) {
        let msg = Message::new(title, message, (0, 0));
        self.messages.push(msg);
    }

    fn clear(&mut self) {
        draw2tty!(
            self.tty,
            "{}{}{}{}",
            cursor::Goto(1, 1),
            clear::All,
            cursor::BlinkingBlock,
            cursor::Show
        );
        self.tty.flush().expect("Cannot flush stdout");
    }

    fn redraw(&mut self) {
        draw2tty!(self.tty, "{}", cursor::Goto(1, 1));
        draw2tty!(
            self.tty,
            "{}{}{}",
            color::Bg(color::Red),
            "F1 shutdown",
            color::Bg(color::Reset)
        );
        draw2tty!(self.tty, "{}", cursor::Right(3));
        draw2tty!(
            self.tty,
            "{}{}{}",
            color::Fg(color::Green),
            "F2 reboot",
            color::Fg(color::Reset)
        );

        self.login_form.draw(&mut self.tty, (1, 1), true);

        let message_left = 2;
        let mut message_top = 3;
        for message in self.messages.iter() {
            message.draw(&mut self.tty, (message_left, message_top), false);
            message_top += message.height();
        }

        draw2tty!(self.tty, "{}", cursor::Restore);
        self.tty.flush().expect("Cannot flush stdout");
    }
}

impl<T: Write> GreetUI for TerminalUI<T> {
    fn set_prompt(&mut self, prompt: &str) {
        self.login_form.reset();
        self.login_form.set_prompt(prompt);
    }

    fn set_secret_prompt(&mut self, prompt: &str) {
        self.login_form.reset();
        self.login_form.set_secret_prompt(prompt);
    }

    fn show_info_message(&mut self, message: &str) {
        self.add_message("Info:", message);
    }

    fn show_error_message(&mut self, message: &str) {
        self.add_message("Error:", message);
    }

    fn show_authentication_failure(&mut self, reason: &str) {
        self.reset();
        self.add_message(
            "Authentication failed!",
            format!("Reason: {}", reason).as_str(),
        );
    }
}

impl<T: Write> Drop for TerminalUI<T> {
    fn drop(&mut self) {
        let (_, height) = termion::terminal_size().unwrap();
        draw2tty!(self.tty, "{}", cursor::Goto(1, height));
        //draw2tty!(self.tty, "{}{}", cursor::Goto(1, 1), clear::All);
    }
}
