extern crate clipboard;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate ws;
extern crate env_logger;

use clipboard::{ClipboardContext, ClipboardProvider};
use common::ClipboardCommand;
use std::{thread, thread::JoinHandle, time};
use ws::{connect, Message, Sender};
use std::env;

mod common;

fn handle_command(command: ClipboardCommand, ctx: &mut ClipboardContext) {
    match command {
        ClipboardCommand::Set { value, session: _ } => {
            let old_clipboard = ctx.get_contents().unwrap_or_default();
            if value != old_clipboard {
                let _ = ctx.set_contents(value);
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
        clipboard_thread(session.clone(), out);

        move |msg: Message| {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            let result: serde_json::Result<ClipboardCommand> = serde_json::from_str(msg.as_text().unwrap_or_default());
            match result {
                Ok(command) => handle_command(command, &mut ctx),
                Err(_) => {}
            }
            Ok(())
        }
    }).unwrap();
}

fn clipboard_thread(session: String, out: Sender) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        let hundred_millis = time::Duration::from_millis(100);
        let mut old_clipboard = ctx.get_contents().unwrap_or_default();

        thread::sleep(hundred_millis);

        // we need to do the listen after returning the closure for the websocket
        // thus we can't send this message in the factory
        send_to_server(&out, &ClipboardCommand::Listen {
            session: session.clone()
        });
        loop {
            thread::sleep(hundred_millis);
            let new_clipboard = ctx.get_contents().unwrap_or_default();
            if new_clipboard != old_clipboard {
                old_clipboard = new_clipboard;
                send_to_server(&out, &ClipboardCommand::Set {
                    session: session.clone(),
                    value: old_clipboard.clone(),
                });
            }
        }
    })
}

fn send_to_server(out: &Sender, command: &ClipboardCommand) {
    let command_text = serde_json::to_string(command).unwrap();
    out.send(Message::from(command_text.clone())).ok();
}