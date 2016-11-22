//! # Protocol
//!
//! ## Server to Client
//! * `U #{N}` - You: where N is your player number
//! * `D [{C} ..]` - Deal: where C.. is a list of space-separated cards
//!
//! ## Server to All
//! * `! {M}` - Error: where M is message
//! * `P #{N} [{C} ..]` - Play: N played C..
//! * `T #{N} [S|F|A]` - Turn: N's turn -- S to start, F to follow, A to any
//! * `W #{N}` - Win: where N emptied their hand
//! * `E [#{N} ..]` - End: where N.. is a list of winners (from 1st to 3rd)
//!
//! ## Client to Server
//! * `G` - Game: ready for game
//! * `P [{C} ..]` - Play: play C..

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use bots::dealer::{self, Output};
use game;

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
    run(streams);
}

fn run(mut streams: Vec<TcpStream>) {
    let mut bot = dealer::new();
    let mut player_input = None;
    loop {
        let mut inp = String::new();
        if let Some(p) = player_input {
            inp = read_line(&streams[p-1]);
        }
        let (outputs, player_input_, stop) = bot.actuate(&inp);
        player_input = player_input_;
        for output in outputs {
            for sout in stream_outputs(output) {
                streams[sout.0 - 1].write((sout.1 + "\r\n").as_bytes())
                    .expect("write error");
            }
        }
        if stop {
            break;
        }
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

fn stream_outputs(out: Output) -> Vec<(game::PlayerNum, String)> {
    match out {
        Output::You(p) => {
            vec![(p, format!("U #{}", p))]
        }
        Output::Error(ref msg) => {
            out_to_all(format!("! #{}", msg))
        }
        Output::Deal(p, ref cards) => {
            vec![(p, format!("D {}", cards))]
        }
        Output::Turn(ref t) => {
            out_to_all(format!("T #{} {}", t.player(), match *t {
                game::Turn::Start(_) => 'S',
                game::Turn::Follow(_) => 'F',
                game::Turn::Any(_) => 'A',
                game::Turn::End => unreachable!(),
            }))
        }
        Output::Play(p, ref cards) => {
            out_to_all(format!("P #{} {}", p, cards))
        }
        Output::PlayError(p) => {
            out_to_all(format!("! #{} didn't play properly.", p))
        }
        Output::Win(p) => {
            out_to_all(format!("W #{}", p))
        }
        Output::End(ref winners) => {
            let winners: Vec<_> = winners.iter().map(|w| format!("#{}", w))
                .collect();
            out_to_all(format!("E {}", winners.join(" ")))
        }
    }
}

fn out_to_all(s: String) -> Vec<(game::PlayerNum, String)> {
    let mut res = vec![];
    for p in 1..5 {
        res.push((p, s.clone()));
    }
    res
}
