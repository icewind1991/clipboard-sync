use clipboard::{ClipboardContext, ClipboardProvider};
use crate::common::ClipboardCommand;
use std::{thread, thread::JoinHandle, time};
use std::env;
use std::sync::{Arc, Mutex};
use ws::{connect, Message, Sender};

mod common;

fn handle_command(command: ClipboardCommand, ctx: &mut ClipboardContext, current_clipboard: Arc<Mutex<String>>) {
    match command {
        ClipboardCommand::Set { value, session: _ } => {
            let mut clip = current_clipboard.lock().unwrap();
            if *clip != value {
                let _ = ctx.set_contents(value.clone());
                *clip = value;
            }
        }
        _ => {}
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

        move |msg: Message| {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            let result: serde_json::Result<ClipboardCommand> = serde_json::from_str(msg.as_text().unwrap_or_default());
            match result {
                Ok(command) => handle_command(command, &mut ctx, current_clipboard.clone()),
                Err(_) => {}
            }
            Ok(())
        }
    }).unwrap();
}

fn clipboard_thread(session: String, out: Sender, current_clipboard: Arc<Mutex<String>>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        let hundred_millis = time::Duration::from_millis(100);
        {
            let mut clip = current_clipboard.lock().unwrap();
            *clip = ctx.get_contents().unwrap_or_default();
        }

        thread::sleep(hundred_millis);

        // we need to do the listen after returning the closure for the websocket
        // thus we can't send this message in the factory
        send_to_server(&out, &ClipboardCommand::Listen {
            session: session.clone()
        });
        loop {
            thread::sleep(hundred_millis);
            let new_clipboard = ctx.get_contents().unwrap_or_default();
            let mut clip = current_clipboard.lock().unwrap();
            if *clip != new_clipboard {
                send_to_server(&out, &ClipboardCommand::Set {
                    session: session.clone(),
                    value: new_clipboard.clone(),
                });
                *clip = new_clipboard;
            }
        }
    })
}

fn send_to_server(out: &Sender, command: &ClipboardCommand) {
    let command_text = serde_json::to_string(command).unwrap();
    out.send(Message::from(command_text.clone())).ok();
}