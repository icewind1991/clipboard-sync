use crate::common::ClipboardCommand;
use mio::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;
use ws::{listen, CloseCode, Error, Handler, Message, Result, Sender};

mod common;

#[derive(Default)]
struct Session {
    clients: HashMap<Token, Sender>,
}

impl Session {
    pub fn join(&mut self, client: Sender) {
        self.clients.insert(client.token(), client);
    }
}

fn handle_command(
    command: ClipboardCommand,
    sessions: &mut HashMap<String, Session>,
    client: &Sender,
) {
    match &command {
        ClipboardCommand::Listen {
            session: session_name,
        } => {
            sessions
                .entry(session_name.clone())
                .or_default()
                .join(client.clone());
        }

        ClipboardCommand::Set {
            value: _,
            session: session_name,
        } => match sessions.get_mut(session_name) {
            Some(session) => {
                send_to_session(session, &command, client.token());
            }
            None => println!("session {} not found", session_name),
        },
    }
}

fn send_to_session(session: &Session, command: &ClipboardCommand, exclude: Token) {
    let command_text = serde_json::to_string(command).unwrap();
    for client in session.clients.values() {
        if client.token() != exclude {
            let _ = client.send(command_text.as_str());
        }
    }
}

struct Server {
    out: Sender,
    sessions: Rc<RefCell<HashMap<String, Session>>>,
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        match ClipboardCommand::try_from(msg) {
            Ok(command) => {
                handle_command(command, &mut self.sessions.borrow_mut(), &self.out);
            }
            Err(err) => {
                println!("{}", err);
            }
        };
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        let mut sessions = self.sessions.borrow_mut();
        let token = self.out.token();

        for session in sessions.values_mut() {
            session.clients.remove(&token);
        }
    }

    fn on_error(&mut self, err: Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

fn main() {
    let port = std::env::var("PORT").unwrap_or("80".to_string());
    let listen_address = format!("0.0.0.0:{}", port);

    println!("listening on: {:?}", listen_address);

    let sessions: Rc<RefCell<HashMap<String, Session>>> = Rc::new(RefCell::new(HashMap::new()));

    let result = listen(listen_address, |out| Server {
        out,
        sessions: sessions.clone(),
    });
    match result {
        Ok(_) => {}
        Err(_) => {
            println!("error while listening");
        }
    }
}
