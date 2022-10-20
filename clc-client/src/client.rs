use std::io::{stdin, stdout, Write};
use std::{thread, time};
use std::fmt::{Formatter};
use std::process::exit;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use getch::Getch;
use terminal_size::{Height, terminal_size, Width};
use websocket::OwnedMessage;
use clc_lib::protocol::{ChatId, ServerUrl, UserId, UserName};
use crate::input_handler::handle_input;
use crate::web_client::{Location};

pub(crate) type ThreadClient = Arc<Mutex<Client>>;

pub(crate) struct Client {
    pub(crate) input: String,
    pub(crate) loc: Location,
    pub(crate) user_id: Option<UserId>,
    pub(crate) name: Option<UserName>,
    pub(crate) chat_id: Option<ChatId>,
    pub(crate) server: Option<ServerUrl>,
    pub(crate) socket: Option<(JoinHandle<()>, JoinHandle<()>)>,
    pub(crate) sender: Option<Sender<OwnedMessage>>
}

pub(crate) trait ClientSeal {
    fn seal(&self) -> MutexGuard<Client>;
}

impl ClientSeal for ThreadClient {
    fn seal(&self) -> MutexGuard<Client> {
        self.lock().unwrap()
    }
}

impl Client {
    pub(crate) fn new() -> Self {
        Self {
            input: String::new(),
            loc: Location::Home,
            user_id: None,
            name: None,
            chat_id: None,
            server: None,
            socket: None,
            sender: None
        }
    }

    pub(crate) fn run_cli(self){
        let client = Arc::new(Mutex::new(self));
        loop {
            Self::prompt_input(&client);
            println!(); // keep input
            handle_input(&client)
        }
    }

    fn term_size() -> (u16, u16){
        if let Some((Width(w), Height(h))) = terminal_size() {
            (w, h)
        } else {
            panic!("\rUnable to get terminal size. Please use different terminal");
        }
    }

    pub(crate) fn writeln(&self, line: &str) {
        let (w, _) = Self::term_size();
        println!("\r{:1$}", line, w as usize);
        self.refresh_input();
    }

    fn refresh_input(&self){
        let (w, _) = Self::term_size();
        let s = format!("> {}", self.input);
        print!("\r{:1$}", s, w as usize);
        let _ = stdout().flush();
    }

    fn prompt_input(client: &ThreadClient) {
        let getch = Getch::new();
        client.seal().input = String::new();
        client.seal().refresh_input();
        let _ = stdout().flush();
        loop {
            if let Ok(c) = getch.getch() {
                let ch = c as char;
                if c == 3 { // ^c
                    exit(0);
                }
                else if c == 8 { // delete
                    client.seal().input.pop();
                    client.seal().refresh_input();
                }
                else if c == 13 { // \n
                    break
                }
                // is printable?
                else if ch.is_ascii() {
                    client.seal().input.push(ch);
                    client.seal().refresh_input();
                }
            }
        }
    }
}