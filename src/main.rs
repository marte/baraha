extern crate baraha;

use std::env;

use baraha::{server, client};

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        panic!("invalid usage")
    }
    match &*args[1] {
        "host" => server::host(),
        "play" => {
            if args.len() != 3 {
                panic!("invalid usage")
            }
            client::play(args[2].clone());
        }
        "bot" => {
            if args.len() != 3 {
                panic!("invalid usage")
            }
            client::bot(args[2].clone());
        }
        _ => panic!("invalid usage")
    }
}
