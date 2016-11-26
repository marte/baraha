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

pub enum Output {
    You(PlayerNum),
    Deal(PlayerNum, Vec<game::Card>),
    Turn(game::Turn),
    Play(PlayerNum, game::Cards),
    Win(PlayerNum),
    End(Vec<PlayerNum>),
    Error(String),
    PlayError(PlayerNum, String),
}


pub struct Dealer {
    state: State,
}

pub fn new() -> Dealer {
    Dealer{state: State::Start}
}

impl Dealer {

    pub fn actuate(&mut self, inp: &str)
                   -> (Vec<Output>, Option<PlayerNum>, bool) {
        let outputs = self.transition(inp);
        (outputs, self.state.player_input(), self.state.has_ended())
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
                    (State::Play(game.clone()),
                     vec![Output::PlayError(player, "invalid input".into())])
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
                                    (State::Play(game.clone()),
                                     vec![Output::PlayError(player, e.into())])
                                }
                            }
                        }
                        Err(e) => {
                            (State::Play(game.clone()),
                             vec![Output::PlayError(player, e.into())])
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
