use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::str::FromStr;

use bots::player::{self, Status, ServerInput, UserInput, ServerOutput};
use game;
use utils;

pub fn play(host: String) {
    let (player, channel) = run_player(host);
    interact(player, channel);
}

pub fn bot(host: String) {
    let (player, channel) = run_player(host);
    greedy_bot(player, channel);
}

fn run_player(host: String) -> (Arc<Mutex<player::Player>>, Channel) {
    let player = Arc::new(Mutex::new(player::new()));
    let channel = Channel::new();
    {
        let player = player.clone();
        let channel = channel.clone();
        thread::spawn(move || run(host, player, channel));
    }
    (player, channel)
}

struct ChannelInfo {
    can_play: bool,
    cards: Option<game::Cards>,
    has_ended: bool,
}

#[derive(Clone)]
struct Channel(Arc<(Mutex<ChannelInfo>, Condvar)>);

impl Channel {

    fn new() -> Self {
        let info = ChannelInfo {
            can_play: false,
            cards: None,
            has_ended: false,
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
        while info.can_play {
            info = (self.0).1.wait(info).unwrap();
        }
    }

    fn wait_for_cards(&mut self) -> game::Cards {
        let mut info = (self.0).0.lock().unwrap();
        info.can_play = true;
        (self.0).1.notify_all();
        while info.cards.is_none() {
            info = (self.0).1.wait(info).unwrap();
        }
        info.can_play = false;
        (self.0).1.notify_all();
        info.cards.take().unwrap()
    }

    fn wait_to_play(&self) -> bool {
        let mut info = (self.0).0.lock().unwrap();
        while !info.can_play && !info.has_ended {
            info = (self.0).1.wait(info).unwrap();
        }
        info.can_play
    }

    fn has_ended(&mut self) {
        let mut info = (self.0).0.lock().unwrap();
        info.has_ended = true;
        (self.0).1.notify_all();
    }
}

fn interact(player: Arc<Mutex<player::Player>>, mut channel: Channel) {
    let stdin = io::stdin();
    let mut hints: Vec<game::Cards> = vec![];
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<_> = line.trim().splitn(2, ' ').collect();
        match tokens[0] {
            "" => (),
            "play" => {
                if !channel.can_play() {
                    println!("It's not yet your turn.");
                    continue;
                }
                if tokens.len() != 2 {
                    println!("What do you want to play?");
                }
                if let Ok(n) = tokens[1].trim().parse::<usize>() {
                    if 1 <= n && n <= hints.len() {
                        channel.play_cards(hints[n-1].clone());
                    } else {
                        println!("Invalid hint index.");
                    }
                } else {
                    match tokens[1].trim().parse() {
                        Ok(cards) => {
                            channel.play_cards(cards);
                        }
                        Err(e) => {
                            println!("Invalid cards: {}", e);
                        }
                    }
                }
            }
            "pass" => {
                if !channel.can_play() {
                    println!("It's not yet your turn.");
                    continue;
                }
                channel.play_cards("".parse().unwrap());
            }
            "last" => {
                let player = player.lock().unwrap();
                let last_play = player.last_play();
                if let Some((p, ref cards)) = *last_play {
                    print!("Player #{} played ", p);
                    pp_cards(cards);
                    println!("");
                } else {
                    println!("No one has played yet.");
                }
            }
            "hand" => {
                let player = player.lock().unwrap();
                let hand = player.hand();
                if hand.is_empty() {
                    println!("You are done!");
                } else {
                    print!("You have ");
                    pp_cards(hand.iter().cloned());
                    println!("");
                }
            }
            "hint" => {
                hints = player.lock().unwrap().hints();
                if hints.is_empty() {
                    println!("You can't play anything.");
                } else {
                    println!("Hints:");
                    for (i, cards) in hints.iter().enumerate() {
                        print!("{:>3}: ", i+1);
                        pp_cards(cards);
                        println!("");
                    }
                }
            }
            "help" => {
                print_usage();
            }
            _ => {
                println!("Invalid input.");
                print_usage();
            }

        }
    }
}

fn print_usage() {
    println!("Usage:
{bold}help{reset} - print this
{bold}play [C ..]{reset} - play list of cards C..
{bold}pass{reset} - pass
{bold}last{reset} - show last played
{bold}hand{reset} - show cards in your hand
{bold}hint{reset} - give hints on what can be played
{bold}play [N]{reset} - where N is the number of the hint",
             bold = style::Bold,
             reset = style::Reset,
    );
}

fn greedy_bot(player: Arc<Mutex<player::Player>>, mut channel: Channel) {
    loop {
        if !channel.wait_to_play() {
            break
        }
        let hints = player.lock().unwrap().hints();
        if hints.is_empty() {
            channel.play_cards("".parse().unwrap());
        } else {
            channel.play_cards(hints[0].clone());
        }
    }
}

fn run(host: String, player: Arc<Mutex<player::Player>>, mut channel: Channel) {
    let mut stream = TcpStream::connect((&*host, 2222)).expect("connection failed");
    let mut status = None;
    loop {
        let (mut s_inp, mut u_inp) = (None, None);
        if let Some(s) = status {
            match s {
                Status::ServerInput => {
                    let inp = utils::read_line(&stream);
                    let inp = inp.trim().parse().expect("invalid server response");
                    print_server_input(&inp);
                    s_inp = Some(inp);
                }
                Status::UserInput => {
                    print_your_turn();
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
    channel.has_ended();
}

impl FromStr for ServerInput {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<_> = s.splitn(2, ' ').collect();
        if tokens.len() != 2 {
            return Err("no args".into())
        }
        match tokens[0] {
            "U" => {
                Ok(ServerInput::You(parse_player_num(tokens[1])?))
            }
            "D" => {
                let mut hand = vec![];
                for s in tokens[1].split_whitespace() {
                    hand.push(s.parse()?);
                }
                Ok(ServerInput::Deal(hand))
            }
            "P" => {
                let args: Vec<_> = tokens[1].splitn(2, ' ').collect();
                if args.len() <= 2 {
                    let p = parse_player_num(args[0])?;
                    let cards_str = if args.len() == 2 { args[1] } else { "" };
                    let cards = cards_str.parse()?;
                    return Ok(ServerInput::Play(p, cards))
                }
                Err("invalid args for P".into())
            }
            "T" => {
                let args: Vec<_> = tokens[1].split_whitespace().collect();
                if args.len() == 2 {
                    let p = parse_player_num(args[0])?;
                    let turn = match args[1] {
                        "S" => game::Turn::Start(p),
                        "F" => game::Turn::Follow(p),
                        "A" => game::Turn::Any(p),
                        _ => return Err(format!("invalid turn type {}", args[1]))
                    };
                    return Ok(ServerInput::Turn(turn))
                }
                Err("invalid args for T".into())
            }
            "W" => {
                Ok(ServerInput::Win(parse_player_num(tokens[1])?))
            }
            "E" => {
                let args: Vec<_> = tokens[1].split_whitespace().collect();
                let mut winners = vec![];
                for arg in args {
                    winners.push(parse_player_num(arg)?);
                }
                Ok(ServerInput::End(winners))
            }
            "?" => {
                Ok(ServerInput::InvalidInput(tokens[1].to_string()))
            }
            "!" => {
                Ok(ServerInput::Error(tokens[1].to_string()))
            }
            _ => Err("invalid input".into())
        }
    }
}

fn parse_player_num(s: &str) -> Result<game::PlayerNum, String> {
    let bytes = s.as_bytes();
    if bytes.len() == 2 && bytes[0] == ('#' as u8) {
        let num = bytes[1] - ('0' as u8);
        if 1 <= num && num <= 4 {
            return Ok(num.into())
        }
    }
    Err("invalid player number".into())
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

use termion::{color, style};

fn print_server_input(inp: &ServerInput) {
    match *inp {
        ServerInput::You(p) => {
            println!("You are player #{}.", p);
        }
        ServerInput::Deal(ref cards) => {
            print!("Your cards are ");
            pp_cards(cards.iter().cloned());
            println!("");
        }
        ServerInput::Turn(turn) => {
            print!("Player #{}'s turn ", turn.player());
            match turn {
                game::Turn::Start(_) => print!("to start"),
                game::Turn::Follow(_) => print!("to follow"),
                game::Turn::Any(_) => print!("for control"),
                game::Turn::End => unreachable!(),
            }
            println!("");
        }
        ServerInput::Play(p, ref cards) => {
            print!("Player #{} ", p);
            if cards.is_pass() {
                print!("{}passed{}", style::Bold, style::Reset);
            } else {
                print!("played ");
                pp_cards(cards);
            }
            println!("");
        }
        ServerInput::Win(p) => {
            println!("Player #{} won.", p);
        }
        ServerInput::End(ref winners) => {
            println!("Game has ended. Winners are:");
            println!("1st: #{}", winners[0]);
            println!("2nd: #{}", winners[1]);
            println!("3rd: #{}", winners[2]);
        }
        ServerInput::InvalidInput(ref msg) => {
            println!("{}Invalid move: {}{}",
                     style::Bold,
                     msg,
                     style::Reset);
        }
        ServerInput::Error(ref msg) => {
            println!("Dealer says: {}", msg);
        }
    }
}

fn print_your_turn() {
    println!("{}It's your turn!{}", style::Bold, style::Reset);
}

fn pp_cards<T: IntoIterator<Item=game::Card>>(cards: T) {
    print!("{}", color::Bg(color::LightWhite));
    for card in cards {
        print!(" ");
        pp_card(card);
        print!(" ");
    }
    print!("{}{}", color::Fg(color::Reset), color::Bg(color::Reset));
}

fn pp_card(card: game::Card) {
    match card.suit {
        'C' | 'S' => print!("{}", color::Fg(color::Black)),
        'H' | 'D' => print!("{}", color::Fg(color::Red)),
        _ => unreachable!()
    }
    print!("{}", card.rank);
    match card.suit {
        'C' => print!("♣"),
        'S' => print!("♠"),
        'H' => print!("♥"),
        'D' => print!("♦"),
        _ => unreachable!()
    }
}
