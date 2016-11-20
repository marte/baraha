use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

enum State {
    Start,
    Wait(usize),
    Deal,
    Error(String),
    End,
}

impl State {
    fn player_input(&self) -> usize {
        match self {
            &State::Wait(x) => x,
            _ => 0,
        }
    }
}

struct Output(usize, String);

fn actuate(s: State, inp: &str) -> (State, Vec<Output>) {
    match s {
        State::Start => {
            (State::Wait(1),
             vec![Output(1, String::from("U #1"))])
        }
        State::Wait(x) => {
            if inp.chars().nth(0).unwrap() != 'G' {
                (State::Error(format!("#{} is not ready.", x)), vec![])
            } else if x == 4 {
                (State::Deal, vec![])
            } else {
                (State::Wait(x+1), vec![Output(x+1, format!("U #{}", x+1))])
            }
        }
        State::Deal => {
            unimplemented!()
        }
        _ => unreachable!()
    }
}

fn fsm(mut streams: Vec<TcpStream>) {
    let mut state = State::Start;
    loop {
        let p = state.player_input();
        let mut inp = String::new();
        if p > 0 {
            inp = read_line(&streams[p-1]);
        }
        let (new_state, outputs) = actuate(state, &inp);
        for out in outputs {
            &streams[out.0 - 1].write((out.1 + "\r\n").as_bytes());
        }
        state = new_state;
        match state {
            State::Error(msg) => {
                println!("ERROR: {}", msg);
                for mut stream in streams {
                    stream.write_fmt(format_args!("! {}\r\n", msg))
                        .expect("write error");
                }
                break;
            }
            State::End => {
                println!("Game has ended.");
                break;
            }
            _ => (),
        }
    }
}

fn read_line(mut stream: &TcpStream) -> String {
    let mut line = String::with_capacity(256);
    loop {
        let mut buf = [0u8];
        let size = stream.read(&mut buf).unwrap();
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
