extern crate rand;
extern crate termion;

use std::env;

mod client;
mod server;
mod game;
mod bots;
mod utils;

fn main() {
    let args: Vec<_> = env::args().collect();
    match args.len() {
        2 => {
            match &*args[1] {
                "host" => server::host(),
                "play" => client::play(),
                _ => panic!("invalid usage")
            }
        }
        _ => panic!("invalid usage")
    }
}
