// This file is part auf the greetly package.
//
// (c) 2020 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use getopts::Options;
use greetd_ipc as greetd;
use std::env;
use std::fmt;
use std::io;

mod context;
mod tui;

use context::{ContextError, GreeterContext};
use tui::TerminalUI;
use tui::{LoginAction, LoginError};

const SESSION_COMMAND: &str = "/bin/bash";

#[derive(Debug)]
enum GreetLyError {
    CLIParsing(getopts::Fail),
    Configuration(String),
    Connection(io::Error),
    Protocol(greetd::codec::Error),
    Session(String),
    UI(String),
    Unauthenticated,
}

impl From<getopts::Fail> for GreetLyError {
    fn from(err: getopts::Fail) -> Self {
        Self::CLIParsing(err)
    }
}

impl From<io::Error> for GreetLyError {
    fn from(err: io::Error) -> Self {
        Self::Connection(err)
    }
}

impl From<greetd::codec::Error> for GreetLyError {
    fn from(err: greetd::codec::Error) -> Self {
        Self::Protocol(err)
    }
}

impl From<LoginError> for GreetLyError {
    fn from(err: LoginError) -> Self {
        Self::UI(err.to_string())
    }
}

impl From<ContextError> for GreetLyError {
    fn from(err: ContextError) -> Self {
        match err {
            ContextError::Connection(err) => err.into(),
            ContextError::Protocol(err) => err.into(),
            ContextError::Internal(msg) => Self::Session(msg),
            ContextError::SocketMissing => {
                Self::Configuration("Environment variable GREETD_SOCK missing".to_owned())
            }
        }
    }
}

impl fmt::Display for GreetLyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CLIParsing(err) => write!(f, "Cannot parse arguments: {}", err),
            Self::Configuration(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::Connection(err) => write!(f, "Conection to greetd failed: {}", err),
            Self::Protocol(err) => write!(f, "Communication with greetd failed: {}", err),
            Self::Session(msg) => write!(f, "Starting the user session failed: {}", msg),
            Self::UI(msg) => write!(f, "UI Error: {}", msg),
            Self::Unauthenticated => write!(f, "Session not authenticated"),
        }
    }
}

fn run_greetly() -> Result<(), GreetLyError> {
    let mut session_ctx = GreeterContext::connect()?;
    let mut ui = TerminalUI::init()?;

    // TODO: Allow for default user

    loop {
        if session_ctx.is_failed() {
            session_ctx.reset()?;
        }

        match ui.handle_input().unwrap() {
            LoginAction::Cancel => {
                session_ctx.cancel()?;
            }
            LoginAction::Submit(data) => {
                if let greetd::Response::Success = session_ctx.send_request(data, &mut ui)? {
                    if let greetd::Response::Success =
                        session_ctx.start(vec![SESSION_COMMAND.to_owned()], &mut ui)?
                    {
                        break;
                    }
                };
            }
            LoginAction::Quit => {
                session_ctx.cancel()?;
                break;
            }
        }
    }

    if session_ctx.is_started() {
        Ok(())
    } else {
        Err(GreetLyError::Unauthenticated)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help message");
    opts.optopt(
        "c",
        "config",
        "config file to use (default: /etc/greetd/greetly.toml)",
        "CONFIG_FILE",
    );
    opts.optopt("e", "cmd", "command to run", "COMMAND");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{}", f.to_string());
            eprintln!("Usage: {} [options]", program);
            eprintln!("See {} --help for a list of supported options", program);
            std::process::exit(1);
        }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", program);
        print!("{}", opts.usage(&brief));
        std::process::exit(0);
    }

    if let Err(err) = run_greetly() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
