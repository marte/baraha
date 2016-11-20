extern crate rand;

mod client;
mod server;
mod game;
mod bots;

fn main() {
    server::host();
}
