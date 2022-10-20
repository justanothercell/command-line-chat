#![feature(duration_constants)]

use std::{io, thread, time};
use crate::client::Client;

mod client;
mod input_handler;
mod web_client;

fn main() {
    let client = Client::new();
    client.run_cli();
}
