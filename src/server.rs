use crate::common::ClipboardCommand;
use mio::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ws::{CloseCode, Error, Handler, listen, Message, Result, Sender};

mod common;

struct Session {
    clients: HashMap<Token, Sender>
}

fn handle_command(command: ClipboardCommand, sessions: &mut HashMap<String, Session>, client: Sender) {
    match command {
        ClipboardCommand::Listen { session: session_name } => {
            sessions.entry(session_name).or_insert_with(|| Session {
                clients: HashMap::new()
            }).clients.insert(client.token(), client);
        }

        ClipboardCommand::Set { value, session: session_name } => {
            match sessions.get_mut(&session_name) {
                Some(session) => {
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
    let listen_address = format!("0.0.0.0:{}", port);

    println!("listening on: {:?}", listen_address);

    let sessions: Rc<RefCell<HashMap<String, Session>>> = Rc::new(RefCell::new(HashMap::new()));

    let result = listen(listen_address, |out| { Server { out, sessions: sessions.clone() } });
    match result {
        Ok(_) => {}
        Err(_) => {
            println!("error while listening");
        }
    }
}