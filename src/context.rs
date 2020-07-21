// This file is part auf the greetly package.
//
// (c) 2020 Bernd Busse
//
// For the full copyright and license information, please view the README.md file
// that was distributed with this source code.
//

use greetd_ipc as greetd;
use std::env;
use std::io;
use std::os::unix::net::UnixStream;

use greetd::codec::SyncCodec;

use crate::tui::GreetUI;

const ENV_SOCKET: &str = "GREETD_SOCK";

#[derive(Debug, Copy, Clone, PartialEq)]
enum ContextState {
    Connected,
    Created,
    Authenticated,
    Started,
    Failed,
}

#[derive(Debug)]
pub enum ContextError {
    Connection(io::Error),
    Protocol(greetd::codec::Error),
    Internal(String),
    SocketMissing,
}

impl From<io::Error> for ContextError {
    fn from(err: io::Error) -> Self {
        Self::Connection(err)
    }
}

impl From<greetd::codec::Error> for ContextError {
    fn from(err: greetd::codec::Error) -> Self {
        Self::Protocol(err)
    }
}

impl From<greetd::Response> for ContextError {
    fn from(response: greetd::Response) -> Self {
        match response {
            greetd::Response::Error { description, .. } => Self::Internal(description),
            _ => unimplemented!("Cannot convert succesful result into Error type"),
        }
    }
}

pub type ContextResult = Result<greetd::Response, ContextError>;

pub struct GreeterContext {
    socket: UnixStream,
    state: ContextState,
}

impl GreeterContext {
    pub fn connect() -> Result<Self, ContextError> {
        let sock_addr = env::var(ENV_SOCKET).map_err(|_| ContextError::SocketMissing)?;
        let socket = UnixStream::connect(sock_addr)?;

        Ok(Self {
            socket,
            state: ContextState::Connected,
        })
    }

    pub fn is_started(&self) -> bool {
        self.state == ContextState::Started
    }

    pub fn is_failed(&self) -> bool {
        self.state == ContextState::Failed
    }

    pub fn reset(&mut self) -> Result<(), ContextError> {
        match self.state {
            ContextState::Started => panic!("Cannot reset connection for started session"),
            ContextState::Failed => {}
            _ => self.cancel()?,
        }

        let sock_addr = env::var(ENV_SOCKET).map_err(|_| ContextError::SocketMissing)?;
        self.socket = UnixStream::connect(sock_addr)?;
        self.state = ContextState::Connected;

        Ok(())
    }

    pub fn send_request(&mut self, data: String, greeter: &mut impl GreetUI) -> ContextResult {
        let reply = match self.state {
            ContextState::Connected => self.create_session(data),
            ContextState::Created => self.authentication_response(Some(data)),
            ContextState::Failed => panic!("Cannot send data with failed connection"),
            _ => panic!("Cannot send request to session in state {:?}", self.state),
        };

        self.handle_response(reply, greeter)
    }

    pub fn start(&mut self, command: Vec<String>, greeter: &mut impl GreetUI) -> ContextResult {
        let reply = match self.state {
            ContextState::Authenticated => self.start_session(command),
            ContextState::Failed => panic!("Cannot start session with failed connection"),
            _ => panic!("Cannot start session in state {:?}", self.state),
        };

        self.handle_response(reply, greeter)
    }

    pub fn cancel(&mut self) -> Result<(), ContextError> {
        let reply = match self.state {
            ContextState::Started => panic!("Cannot cancel already started session"),
            ContextState::Failed => panic!("Cannot close failed connection"),
            _ => self.cancel_session(),
        };

        self.state = match reply {
            Err(_) => ContextState::Failed,
            Ok(_) => ContextState::Connected,
        };

        reply
    }

    fn create_session(&mut self, username: String) -> ContextResult {
        debug_assert!(
            self.state == ContextState::Connected,
            format!("Cannot create session: Invalid state {:?}", self.state)
        );

        let request = greetd::Request::CreateSession { username };
        request.write_to(&mut self.socket)?;
        let response = greetd_ipc::Response::read_from(&mut self.socket)?;

        Ok(response)
    }

    fn authentication_response(&mut self, response: Option<String>) -> ContextResult {
        debug_assert!(
            self.state == ContextState::Created,
            format!(
                "Cannot reply to authentication message: Invalid state {:?}",
                self.state
            )
        );

        let request = greetd::Request::PostAuthMessageResponse { response };
        request.write_to(&mut self.socket)?;
        let response = greetd_ipc::Response::read_from(&mut self.socket)?;

        Ok(response)
    }

    fn start_session(&mut self, command: Vec<String>) -> ContextResult {
        debug_assert!(
            self.state == ContextState::Authenticated,
            format!("Cannot start session: Invalid state {:?}", self.state)
        );

        let request = greetd::Request::StartSession { cmd: command };
        request.write_to(&mut self.socket)?;
        let response = greetd_ipc::Response::read_from(&mut self.socket)?;

        Ok(response)
    }

    fn cancel_session(&mut self) -> Result<(), ContextError> {
        debug_assert!(
            self.state != ContextState::Failed,
            format!(
                "Cannot cancel current session: Invalid state {:?}",
                self.state
            )
        );

        let request = greetd::Request::CancelSession;
        request.write_to(&mut self.socket)?;
        greetd_ipc::Response::read_from(&mut self.socket)?;

        Ok(())
    }

    fn handle_response(
        &mut self,
        reply: ContextResult,
        greeter: &mut impl GreetUI,
    ) -> ContextResult {
        self.state = match reply {
            Err(_) => ContextState::Failed,
            Ok(ref response) => match *response {
                greetd::Response::Error { .. } => {
                    self.cancel()?;
                    ContextState::Connected
                }
                greetd::Response::AuthMessage { .. } => ContextState::Created,
                greetd::Response::Success => match self.state {
                    ContextState::Authenticated => ContextState::Started,
                    _ => ContextState::Authenticated,
                },
            },
        };

        match reply {
            Err(_) => reply,
            Ok(response) => match response {
                greetd::Response::Success => Ok(response),
                greetd::Response::AuthMessage {
                    ref auth_message_type,
                    ref auth_message,
                } => match *auth_message_type {
                    greetd::AuthMessageType::Visible => {
                        greeter.set_prompt(auth_message);
                        Ok(response)
                    }
                    greetd::AuthMessageType::Secret => {
                        greeter.set_secret_prompt(auth_message);
                        Ok(response)
                    }
                    greetd::AuthMessageType::Info => {
                        greeter.show_info_message(auth_message);
                        let ack = self.authentication_response(None);
                        self.handle_response(ack, greeter)
                    }
                    greetd::AuthMessageType::Error => {
                        greeter.show_error_message(auth_message);
                        let ack = self.authentication_response(None);
                        self.handle_response(ack, greeter)
                    }
                },
                greetd::Response::Error {
                    ref error_type,
                    ref description,
                } => match *error_type {
                    greetd::ErrorType::AuthError => {
                        greeter.show_authentication_failure(description);
                        Ok(response)
                    }
                    greetd::ErrorType::Error => Err(ContextError::from(response)),
                },
            },
        }
    }
}

impl Drop for GreeterContext {
    fn drop(&mut self) {
        match self.state {
            ContextState::Started | ContextState::Failed => {}
            _ => {
                let _ = self.cancel();
            }
        }
    }
}
