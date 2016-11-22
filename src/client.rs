use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::str::FromStr;

use bots::player::{self, Status, ServerInput, UserInput, ServerOutput};
use game;
use utils;

pub fn play() {
    let player = Arc::new(Mutex::new(player::new()));
    let channel = Channel::new();
    {
        let channel = channel.clone();
        thread::spawn(move || run(player, channel));
    }
    interact(channel);
}

struct ChannelInfo {
    can_play: bool,
    cards: Option<game::Cards>,
}

#[derive(Clone)]
struct Channel(Arc<(Mutex<ChannelInfo>, Condvar)>);

impl Channel {

    fn new() -> Self {
        let info = ChannelInfo {
            can_play: false,
            cards: None,
        };
        Channel(Arc::new((Mutex::new(info), Condvar::new())))
    }

    fn can_play(&self) -> bool {
        (self.0).0.lock().unwrap().can_play
    }

    fn play_cards(&mut self, cards: game::Cards) {
        let mut info = (self.0).0.lock().unwrap();
        assert!(info.can_play);
        info.cards = Some(cards);
        (self.0).1.notify_all();
    }

    fn wait_for_cards(&mut self) -> game::Cards {
        let mut info = (self.0).0.lock().unwrap();
        info.can_play = true;
        while info.cards.is_none() {
            info = (self.0).1.wait(info).unwrap();
        }
        info.can_play = false;
        info.cards.take().unwrap()
    }
}

fn interact(mut channel: Channel) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if !channel.can_play() {
            println!("It's not yet your turn.");
            continue;
        }
        match line.unwrap().trim().parse() {
            Ok(cards) => {
                channel.play_cards(cards);
            }
            Err(e) => {
                println!("Invalid cards: {}", e);
            }
        }
    }
}

fn run(player: Arc<Mutex<player::Player>>, mut channel: Channel) {
    let mut stream = TcpStream::connect("127.0.0.1:2222").expect("connection failed");
    let mut status = None;
    loop {
        let (mut s_inp, mut u_inp) = (None, None);
        if let Some(s) = status {
            match s {
                Status::ServerInput => {
                    let inp = utils::read_line(&stream);
                    let inp = inp.trim();
                    // echo server input
                    println!("{}", inp);
                    s_inp = Some(inp.parse().expect("invalid server response"));
                }
                Status::UserInput => {
                    u_inp = Some(UserInput::Play(channel.wait_for_cards()));
                }
                Status::End => break,
            }
        }
        let output;
        {
            let mut p = player.lock().unwrap();
            let res = p.actuate(s_inp, u_inp);
            output = res.0;
            status = Some(res.1);
        }
        if let Some(output) = output {
            stream.write((output.to_string() + "\r\n").as_bytes()).expect("write error");
        }
    }
}

impl FromStr for ServerInput {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<_> = s.splitn(2, ' ').collect();
        if tokens.len() != 2 {
            return Err("no args")
        }
        match tokens[0] {
            "U" => {
                Ok(ServerInput::You(try!(parse_player_num(tokens[1]))))
            }
            "D" => {
                Ok(ServerInput::Deal(try!(tokens[1].parse())))
            }
            "P" => {
                let args: Vec<_> = tokens[1].splitn(2, ' ').collect();
                if args.len() <= 2 {
                    let p = try!(parse_player_num(args[0]));
                    let cards_str = if args.len() == 2 { args[1] } else { "" };
                    let cards = try!(cards_str.parse());
                    return Ok(ServerInput::Play(p, cards))
                }
                Err("invalid args for P")
            }
            "T" => {
                let args: Vec<_> = tokens[1].split_whitespace().collect();
                if args.len() == 2 {
                    let p = try!(parse_player_num(args[0]));
                    let turn = match args[1] {
                        "S" => game::Turn::Start(p),
                        "F" => game::Turn::Follow(p),
                        "A" => game::Turn::Any(p),
                        _ => return Err("invalid turn type")
                    };
                    return Ok(ServerInput::Turn(turn))
                }
                Err("invalid args for T")
            }
            "W" => {
                Ok(ServerInput::Win(try!(parse_player_num(tokens[1]))))
            }
            "E" => {
                let args: Vec<_> = tokens[1].split_whitespace().collect();
                let mut winners = vec![];
                for arg in args {
                    winners.push(try!(parse_player_num(arg)));
                }
                Ok(ServerInput::End(winners))
            }
            "!" => {
                Ok(ServerInput::Error(tokens[1].to_string()))
            }
            _ => Err("invalid input")
        }
    }
}

fn parse_player_num(s: &str) -> Result<game::PlayerNum, &'static str> {
    let bytes = s.as_bytes();
    if bytes.len() == 2 && bytes[0] == ('#' as u8) {
        let num = bytes[1] - ('0' as u8);
        if 1 <= num && num <= 4 {
            return Ok(num.into())
        }
    }
    Err("invalid player number")
}

impl ToString for ServerOutput {
    fn to_string(&self) -> String {
        match *self {
            ServerOutput::Game => "G".to_string(),
            ServerOutput::Play(ref cards) => {
                format!("P {}", cards)
            }
        }
    }
}
