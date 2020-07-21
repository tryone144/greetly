// This file is part auf the greetly package.
//
// (c) 2020 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use nix::sys::utsname::uname;
use std::fmt;

use crate::tui::components::{BorderType, Container, Label, TextInput};

use std::io::Write;

use crate::tui::{Draw, KeyboardInput};

const DEFAULT_PROMPT: &str = "Login:";

#[derive(PartialEq)]
pub enum FormElement {
    Prompt,
    //Session,
}

pub struct LoginForm<T> {
    position: (u16, u16),
    _size: (u16, u16),
    host_label: Label,
    session_label: Label,
    input_label: Label,
    prompt_label: Label,
    prompt_input: TextInput<T>,
    container: Container,
    focus: FormElement,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> fmt::Display for LoginForm<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.prompt_input)
    }
}

impl<T: Write> LoginForm<T> {
    pub fn new(position: (u16, u16), size: (u16, u16)) -> Self {
        let uts = uname();
        let host_len = uts.nodename().chars().count() as u16;
        let host_label = Label::new(uts.nodename(), ((size.0 - host_len) / 2, 2));

        let session_label = Label::new("Session:", (3, size.1 - 7));

        let input_label = Label::new(">", (4, size.1 - 3));
        let prompt_label = Label::new(DEFAULT_PROMPT, (3, size.1 - 5));
        let prompt_input = TextInput::new(size.0 as usize - 10, false, (7, size.1 - 3));

        let container = Container::new(BorderType::Unicode, (0, 0), size);

        Self {
            position,
            _size: size,
            host_label,
            session_label,
            input_label,
            prompt_label,
            prompt_input,
            container,
            focus: FormElement::Prompt,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn reset(&mut self) {
        self.prompt_label.set_text(DEFAULT_PROMPT);
        self.prompt_input.clear();
        self.focus = FormElement::Prompt;
    }

    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt_label.set_text(prompt);
    }

    pub fn set_secret_prompt(&mut self, prompt: &str) {
        self.prompt_label.set_text(prompt);
        self.prompt_input.set_secret(true);
    }

    pub fn focus(&self) -> &FormElement {
        &self.focus
    }

    pub fn focus_prev(&mut self) {
        //self.focus = match self.focus {
        //    FormElement::Username => FormElement::Username,
        //    FormElement::Password => FormElement::Username,
        //};
    }

    pub fn focus_next(&mut self) {
        //self.focus = match self.focus {
        //    FormElement::Username => FormElement::Password,
        //    FormElement::Password => FormElement::Password,
        //};
    }
}

impl<T: Write> KeyboardInput<T> for LoginForm<T> {
    fn clear(&mut self) {
        KeyboardInput::<T>::clear(match self.focus {
            FormElement::Prompt => &mut self.prompt_input,
        });
    }

    fn push(&mut self, c: char) {
        KeyboardInput::<T>::push(
            match self.focus {
                FormElement::Prompt => &mut self.prompt_input,
            },
            c,
        );
    }

    fn pop(&mut self, right: bool) {
        KeyboardInput::<T>::pop(
            match self.focus {
                FormElement::Prompt => &mut self.prompt_input,
            },
            right,
        );
    }

    fn is_empty(&self) -> bool {
        self.prompt_input.is_empty()
    }

    fn toggle(&mut self) {
        self.prompt_input.toggle();
    }

    fn cursor_left(&mut self) {
        KeyboardInput::<T>::cursor_left(match self.focus {
            FormElement::Prompt => &mut self.prompt_input,
        });
    }

    fn cursor_right(&mut self) {
        KeyboardInput::<T>::cursor_right(match self.focus {
            FormElement::Prompt => &mut self.prompt_input,
        });
    }

    fn cursor_start(&mut self) {
        KeyboardInput::<T>::cursor_start(match self.focus {
            FormElement::Prompt => &mut self.prompt_input,
        });
    }

    fn cursor_end(&mut self) {
        KeyboardInput::<T>::cursor_end(match self.focus {
            FormElement::Prompt => &mut self.prompt_input,
        });
    }
}

impl<T: Write> Draw<T> for LoginForm<T> {
    fn draw(&self, tty: &mut T, origin: (u16, u16), focused: bool) {
        let new_origin = (origin.0 + self.position.0, origin.1 + self.position.1);

        self.container.draw(tty, new_origin, focused);
        self.host_label.draw(tty, new_origin, false);

        self.session_label.draw(
            tty, new_origin, false, /*self.focus == FormElement::Session*/
        );

        self.input_label
            .draw(tty, new_origin, self.focus == FormElement::Prompt);
        self.prompt_label
            .draw(tty, new_origin, self.focus == FormElement::Prompt);
        self.prompt_input
            .draw(tty, new_origin, self.focus == FormElement::Prompt);
    }
}
