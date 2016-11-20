use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use bots::DealerBot;


fn fsm(mut streams: Vec<TcpStream>) {
    let mut bot = DealerBot::new();
    let mut player_input = None;
    loop {
        let mut inp = String::new();
        if let Some(p) = player_input {
            inp = read_line(&streams[p-1]);
        }
        let (outputs, player_input_, stop) = bot.actuate(&inp);
        player_input = player_input_;
        for out in outputs {
            streams[out.0 - 1].write((out.1 + "\r\n").as_bytes())
                .expect("write error");
        }
        if stop {
            break;
        }
        // state = new_state;
        // match state {
        //     State::Error(msg) => {
        //         println!("ERROR: {}", msg);
        //         for mut stream in streams {
        //             stream.write_fmt(format_args!("! {}\r\n", msg))
        //                 .expect("write error");
        //         }
        //         break;
        //     }
        //     State::End => {
        //         println!("Game has ended.");
        //         break;
        //     }
        //     _ => (),
        // }
    }
}

fn read_line(mut stream: &TcpStream) -> String {
    let mut line = String::with_capacity(256);
    loop {
        let mut buf = [0u8];
        let size = stream.read(&mut buf).expect("read error");
        assert_eq!(1, size); // we don't expect EOF in our protocol
        line.push(buf[0] as char);
        if buf[0] == 10 { // '\n'
            return line
        }
    }
}

pub fn host() {
    let listener = TcpListener::bind("127.0.0.1:2222").unwrap();

    println!("Waiting for 4 players.");

    let mut streams: Vec<TcpStream> = vec![];
    for stream in listener.incoming() {
        streams.push(stream.unwrap());
        let len = streams.len();
        if len < 4 {
            println!("Waiting for {} player(s).", 4-len);
        } else {
            println!("Game!");
            break;
        }
    }
    fsm(streams);
}
