/// # Protocol
///
/// # Server to Client
/// `U #{N}` - where N is your player number
/// `D {C..}` - where C.. is a list of space-separated cards
///
/// # Server to All
/// `! #{M}` - where M is message
/// `P #{N} {C..}` - N played C..
/// `T #{N} [S|F|A]` - N's turn: S to start, F to follow, A to any
/// `W #{N}` - where N emptied their hand
/// `E #{N..}` - where N is a list of winners (from 1st to 3rd)
///
/// # Client to Server
/// `P {C..}` - play C..

use std::rc::Rc;

use game::{self, PlayerNum, Game};

enum State {
    Start,
    Wait(PlayerNum),
    Deal,
    Play(Rc<Game>),
    Error,
    End,
}

impl State {
    fn player_input(&self) -> Option<PlayerNum> {
        match *self {
            State::Wait(x) => Some(x),
            State::Play(ref g) => Some(g.turn().player()),
            _ => None,
        }
    }
    fn has_ended(&self) -> bool {
        match *self {
            State::End | State::Error => true,
            _ => false,
        }
    }
}

enum Output {
    You(PlayerNum),
    Deal(PlayerNum, game::Cards),
    Turn(game::Turn),
    Play(PlayerNum, game::Cards),
    Win(PlayerNum),
    End(Vec<PlayerNum>),
    Error(String),
    PlayError(PlayerNum),
}

impl Output {
    fn stream_outputs(&self) -> Vec<(PlayerNum, String)> {
        match *self {
            Output::You(p) => {
                vec![(p, format!("U #{}", p))]
            }
            Output::Error(ref msg) => {
                Self::out_to_all(format!("! #{}", msg))
            }
            Output::Deal(p, ref cards) => {
                vec![(p, format!("D {}", cards))]
            }
            Output::Turn(ref t) => {
                Self::out_to_all(format!("T #{} {}", t.player(), match *t {
                    game::Turn::Start(_) => 'S',
                    game::Turn::Follow(_) => 'F',
                    game::Turn::Any(_) => 'A',
                    game::Turn::End => unreachable!(),
                }))
            }
            Output::Play(p, ref cards) => {
                Self::out_to_all(format!("P #{} {}", p, cards))
            }
            Output::PlayError(p) => {
                Self::out_to_all(format!("! #{} didn't play properly.", p))
            }
            Output::Win(p) => {
                Self::out_to_all(format!("W #{}", p))
            }
            Output::End(ref winners) => {
                let winners: Vec<_> = winners.iter().map(|w| format!("#{}", w))
                    .collect();
                Self::out_to_all(format!("E {}", winners.join(" ")))
            }
        }
    }

    fn out_to_all(s: String) -> Vec<(PlayerNum, String)> {
        let mut res = vec![];
        for p in 1..5 {
            res.push((p, s.clone()));
        }
        res
    }
}

pub struct DealerBot {
    state: State,
}

impl DealerBot {

    pub fn new() -> DealerBot {
        DealerBot{state: State::Start}
    }

    pub fn actuate(&mut self, inp: &str)
                   -> (Vec<(PlayerNum, String)>, Option<PlayerNum>, bool) {
        let outputs = self.transition(inp);
        let mut stream_outputs = vec![];
        for output in outputs {
            stream_outputs.extend(output.stream_outputs());
        }
        (stream_outputs, self.state.player_input(), self.state.has_ended())
    }

    fn transition(&mut self, inp: &str) -> Vec<Output> {
        let (new_state, outputs) = match self.state {
            State::Start => {
                (State::Wait(1), vec![Output::You(1)])
            }
            State::Wait(x) => {
                if inp.chars().nth(0).unwrap() != 'G' {
                    (State::Error,
                     vec![Output::Error(format!("#{} is not ready.", x))])
                } else if x == 4 {
                    (State::Deal, vec![])
                } else {
                    (State::Wait(x+1), vec![Output::You(x+1)])
                }
            }
            State::Deal => {
                let game = Rc::new(Game::new());
                let mut outputs = vec![];
                for p in 1..5 {
                     outputs.push(Output::Deal(p, game.hand(p)));
                }
                let turn = game.turn();
                println!("Game is starting. #{} to start.", turn.player());
                outputs.push(Output::Turn(turn));
                (State::Play(game), outputs)
            }
            State::Play(ref mut game) => {
                let tokens: Vec<_> = inp.trim().splitn(2, ' ').collect();
                let player = game.turn().player();
                if tokens.len() == 0 || tokens[0] != "P" {
                    println!("Invalid input for #{}.", player);
                    (State::Play(game.clone()), vec![Output::PlayError(player)])
                } else {
                    let token = if tokens.len() == 1 { "" } else { tokens[1] };
                    match token.parse() {
                        Ok(cards) => {
                            match Rc::get_mut(game).unwrap().play(&cards) {
                                Ok(wins) => {
                                    let mut outputs = vec![];
                                    outputs.push(Output::Play(player, cards));
                                    if wins {
                                        outputs.push(Output::Win(player));
                                    }
                                    let turn = game.turn();
                                    match turn {
                                        game::Turn::End => {
                                            outputs.push(
                                                Output::End(game.winners()));
                                            (State::End, outputs)
                                        }
                                        _ => {
                                            outputs.push(Output::Turn(turn));
                                            (State::Play(game.clone()), outputs)
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("Bad play of #{}: {}", player, e);
                                    (State::Play(game.clone()),
                                     vec![Output::PlayError(player)])
                                }
                            }
                        }
                        Err(e) => {
                            println!("Cannot parse input of #{}: {}.", player, e);
                            (State::Play(game.clone()),
                             vec![Output::PlayError(player)])
                        }
                    }
                }
            }
            _ => unreachable!()
        };
        self.state = new_state;
        outputs
    }
}
