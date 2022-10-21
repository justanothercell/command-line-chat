use std::process::exit;
use clc_lib::protocol::{ChatId, ChatTitle, ClientWsMessage, FilePath, InviteId, ServerUrl, UserName};
use crate::Client;
use crate::client::{ClientSeal, ThreadClient};
use crate::web_client::Location;

#[derive(Clone)]
pub(crate) enum Command {
    Help,
    Info,
    Connect(ServerUrl, UserName),
    CreateChat(ChatTitle),
    Join(ChatId, InviteId),
    ListMembers,
    CreateInvite,
    Kick(UserName),
    Quit,
    Upload(FilePath),
    Admin(UserName),
    SendMessage(String)
}

impl Command {
    fn cmd_ident(&self) -> String{
        if let Command::SendMessage(_) = self {
            String::from("message")
        }
        else {
            format!("/{}", match &self {
                Command::Help => '?',
                Command::Info => 'i',
                Command::Connect(_, _) => 'c',
                Command::CreateChat(_) => 'p',
                Command::Join(_, _) => 'j',
                Command::ListMembers => 'l',
                Command::CreateInvite => 'n',
                Command::Kick(_) => 'k',
                Command::Quit => 'q',
                Command::Upload(_) => 'f',
                Command::Admin(_) => 'y',
                Command::SendMessage(_) => unreachable!()
            })
        }
    }
}

const COMMAND_HELP: &'static str = include_str!("../command-help.md");

pub(crate) fn handle_input(client: &ThreadClient) {
    let mut input = client.seal().input.to_owned();
    client.seal().input = String::new();
    input = input.trim().to_string();
    if input.is_empty() {
        return;
    }
    if input.starts_with('/'){
        client.seal().writeln(&input);
    }

    match parse_command(input) {
        Ok(Command::Help) => {
            client.seal().writeln(COMMAND_HELP);
        }
        Ok(cmd) => match {
            let loc = client.seal().loc.clone();
            loc
        } {
            Location::Home => {
                match cmd {
                    Command::Info => {
                        let mut info = String::new();
                        info.push_str(&format!("client-version: {}\n", env!("CARGO_PKG_VERSION")));
                        info.push_str(&format!("location: {}\n", client.seal().loc));
                        info.push_str("\nConnect to a server with '/c <url> <name>'");
                        client.seal().writeln(info.trim_end());
                    }
                    Command::Quit => {
                        exit(0)
                    }
                    Command::Connect(url, name) => {
                        Client::connect_server(client, url, name);
                    }
                    other => {
                        client.seal().writeln(&format!("'{}' is not available in this context", other.cmd_ident()));
                    }
                }
            }
            Location::Lobby => {
                match cmd {
                    Command::Info => {
                        let c = client.seal();
                        let mut info = String::new();
                        info.push_str(&format!("client-version: {}\n", env!("CARGO_PKG_VERSION")));
                        info.push_str(&format!("location: {}\n", c.loc));
                        info.push_str(&format!("server: {}\n", c.server.as_ref().unwrap()));
                        info.push_str(&format!("server-version: {}\n", c.server_version.as_ref().unwrap()));
                        info.push_str("\nJoin a chat with '/j <chat_id> <invite>'\nor create a new one with '/p <title>'");
                        client.seal().writeln(info.trim_end());
                    }
                    Command::Quit => {
                        Client::disconnect_server(client);
                    }
                    Command::CreateChat(title) => {
                        Client::send_ws_message(client, ClientWsMessage::ChatCreate(title));
                    }
                    Command::Join(chat_id, invite_id) => {
                        Client::send_ws_message(client, ClientWsMessage::ChatJoin(chat_id, invite_id));
                    }
                    other => {
                        client.seal().writeln(&format!("'{}' is not available in this context", other.cmd_ident()));
                    }
                }
            }
            Location::Chat => {
                match cmd {
                    Command::Info => {
                        let c = client.seal();
                        let mut info = String::new();
                        info.push_str(&format!("client-version: {}\n", env!("CARGO_PKG_VERSION")));
                        info.push_str(&format!("location: {}\n", c.loc));
                        info.push_str(&format!("server: {}\n", c.server.as_ref().unwrap()));
                        info.push_str(&format!("server-version: {}\n", c.server_version.as_ref().unwrap()));
                        info.push_str(&format!("chat: {}\n", c.chat_title.as_ref().unwrap()));
                        info.push_str(&format!("is-admin: {}\n", c.is_admin));
                        c.writeln(info.trim_end());
                    }
                    Command::Quit => {
                        Client::send_ws_message(client, ClientWsMessage::ChatLeave);
                        let mut c = client.seal();
                        c.writeln(&format!("Disconnected from chat {}", c.chat_title.as_ref().unwrap()));
                        c.loc = Location::Lobby;
                        c.chat_id = None;
                        c.chat_title = None;
                    }
                    Command::SendMessage(content) => {
                        Client::send_ws_message(&client, ClientWsMessage::Message(content));
                    }
                    Command::CreateInvite => {
                        Client::send_ws_message(&client, ClientWsMessage::ChatCreateInvite);
                    }
                    Command::ListMembers => {
                        Client::send_ws_message(&client, ClientWsMessage::ChatListMembers);
                    }
                    other => {
                        client.seal().writeln(&format!("'{}' is not available in this context", other.cmd_ident()));
                    }
                }
            }
        }
        Err(err) => {
            client.seal().writeln(&err);
        }
    }
}

fn parse_command(command: String) -> Result<Command, String> {
    macro_rules! invalid_command {
        () => {format!("Invalid command '{}'. Type '/?' for help", command)};
    }

    if command.starts_with("/") {
        let mut args: Vec<String> = command.split(' ').filter(|&x| !x.is_empty()).map(|x| String::from(x)).collect();
        macro_rules! args_len {
            ($len: literal, $cmd: literal) => {
                if args.len() != $len {
                    Err(format!("Command /{} expects {} args, found {}", $cmd, $len, args.len()))
                }
                else {
                    Ok(())
                }
            }
        }
        macro_rules! arg {
            () => {
                args.remove(0)
            }
        }
        if args.remove(0).len() > 2 {
            return Err(invalid_command!())
        }
        if command.len() >= 2 {
            match command.as_bytes()[1] as char {
                '?' => Ok(Command::Help),
                'i' => Ok(Command::Info),
                'c' => {
                    args_len!(2, 'c')?;
                    Ok(Command::Connect(arg!(), arg!()))
                },
                'p' => {
                    args_len!(1, 'p')?;
                    Ok(Command::CreateChat(arg!()))
                },
                'j' => {
                    args_len!(2, 'j')?;
                    Ok(Command::Join(arg!(), arg!()))
                },
                'l' => Ok(Command::ListMembers),
                'k' => {
                    args_len!(1, 'k')?;
                    Ok(Command::Kick(args.remove(0)))
                },
                'q' => Ok(Command::Quit),
                'f' => {
                    args_len!(1, 'f')?;
                    Ok(Command::Upload(args.remove(0)))
                },
                'y' => {
                    args_len!(1, 'y')?;
                    Ok(Command::Admin(args.remove(0)))
                },
                'n' => {
                    args_len!(0, 'n')?;
                    Ok(Command::CreateInvite)
                },
                _ => Err(invalid_command!())
            }
        }
        else {
            Err(invalid_command!())
        }
    }
    else {
        Ok(Command::SendMessage(command))
    }
}

