use crate::clip::ClipboardHandler;
use crate::common::ClipboardCommand;
use clipboard::ClipboardProvider;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{thread, thread::JoinHandle};
use ws::{connect, Message, Sender};

mod clip;
mod common;

fn handle_command(
    command: ClipboardCommand,
    ctx: &mut ClipboardHandler,
    current_clipboard: Arc<Mutex<String>>,
) {
    if let ClipboardCommand::Set { value, .. } = command {
        let mut clip = current_clipboard.lock().unwrap();
        if *clip != value && value != "" {
            let _ = ctx.set_contents(value.clone());
            *clip = value;
        }
    }
}

fn main() {
    env_logger::init();
    let args: Vec<_> = env::args().collect();

    if args.len() != 3 {
        println!("usage: {} <ws_url> <channel>", args[0]);
        return;
    }

    let session = args[2].clone();
    let url = args[1].clone();

    println!("connecting to {} on channel {}", url, session);

    connect(url, |out| {
        let current_clipboard = Arc::new(Mutex::new(String::new()));
        clipboard_thread(session.clone(), out, current_clipboard.clone());

        let ctx: RefCell<ClipboardHandler> = RefCell::new(ClipboardHandler::new().unwrap());

        move |msg: Message| {
            if let Ok(command) = ClipboardCommand::try_from(msg) {
                handle_command(command, &mut ctx.borrow_mut(), current_clipboard.clone());
            }
            Ok(())
        }
    })
    .unwrap();
}

const HUNDRED_MS: Duration = Duration::from_millis(100);

fn clipboard_thread(
    session: String,
    out: Sender,
    current_clipboard: Arc<Mutex<String>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut ctx: ClipboardHandler = ClipboardHandler::new().unwrap();

        {
            let mut clip = current_clipboard.lock().unwrap();
            *clip = ctx.get_contents().unwrap_or_default();
        }

        thread::sleep(HUNDRED_MS);

        // we need to do the listen after returning the closure for the websocket
        // thus we can't send this message in the factory
        send_to_server(
            &out,
            &ClipboardCommand::Listen {
                session: session.clone(),
            },
        );
        loop {
            thread::sleep(HUNDRED_MS);
            let new_clipboard = ctx.get_contents().unwrap_or_default();
            let mut clip = current_clipboard.lock().unwrap();
            if *clip != new_clipboard && new_clipboard != "" {
                send_to_server(
                    &out,
                    &ClipboardCommand::Set {
                        session: session.clone(),
                        value: new_clipboard.clone(),
                    },
                );
                *clip = new_clipboard;
            }
        }
    })
}

fn send_to_server(out: &Sender, command: &ClipboardCommand) {
    let _ = out.send(command);
}
