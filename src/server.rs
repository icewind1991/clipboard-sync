extern crate mio;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate ws;

use common::ClipboardCommand;
use mio::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::{CloseCode, Error, Handler, listen, Message, Result, Sender};

mod common;

struct Session {
    clients: HashMap<Token, Sender>,
    value: String,
}

fn handle_command(command: ClipboardCommand, sessions: &mut HashMap<String, Session>, client: Sender) {
    match command {
        ClipboardCommand::Listen { session: session_name } => {
            if !sessions.contains_key(&session_name) {
                let mut new_session = Session {
                    clients: HashMap::new(),
                    value: String::new(),
                };

                new_session.clients.insert(client.token(), client);
                sessions.insert(session_name.clone(), new_session);
            } else {
                let session = sessions.get_mut(&session_name).unwrap();
                session.clients.insert(client.token(), client);
            }
        }

        ClipboardCommand::Set { value, session: session_name } => {
            match sessions.get_mut(&session_name) {
                Some(mut session) => {
                    session.value = value.clone();
                    send_to_session(session, &ClipboardCommand::Set {
                        value,
                        session: session_name,
                    }, client.token());
                }
                None => println!("session {} not found", session_name)
            }
        }
    }
}

fn send_to_session(session: &Session, command: &ClipboardCommand, exclude: Token) {
    let command_text = serde_json::to_string(command).unwrap();
    for client in session.clients.values() {
        if client.token() != exclude {
            client.send(Message::from(command_text.clone())).ok();
        }
    }
}


struct Server {
    out: Sender,
    sessions: Rc<RefCell<HashMap<String, Session>>>,
}

impl Handler for Server {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let result: serde_json::Result<ClipboardCommand> = serde_json::from_str(msg.as_text().unwrap_or_default());
        match result {
            Ok(command) => {
                handle_command(command, &mut self.sessions.borrow_mut(), self.out.clone());
                Ok(())
            }
            Err(_) => {
                println!("invalid message: {}", msg.as_text().unwrap_or_default());
                Ok(())
            }
        }
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
    let listen_adress = format!("0.0.0.0:{}", port);

    println!("listening on: {:?}", listen_adress);

    let sessions: Rc<RefCell<HashMap<String, Session>>> = Rc::new(RefCell::new(HashMap::new()));

    let result = listen(listen_adress, |out| { Server { out, sessions: sessions.clone() } });
    match result {
        Ok(_) => {}
        Err(_) => {
            println!("error while listening");
        }
    }
}