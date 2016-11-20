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
///
/// # Client to Server
/// `P {C..}` - play C..

use game::{self, PlayerNum, Game};

enum State {
    Start,
    Wait(PlayerNum),
    Deal,
    Play(Box<Game>),
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
    Deal(PlayerNum, Vec<game::Card>),
    Turn(game::Turn),
    Error(String),
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
                let str_cards: Vec<_> = cards.iter()
                    .map(|c| c.to_string())
                    .collect();
                vec![(p, format!("D {}", str_cards.join(" ")))]
            }
            Output::Turn(ref t) => {
                Self::out_to_all(format!("T #{} {}", t.player(), match *t {
                    game::Turn::Start(_) => 'S',
                    game::Turn::Follow(_) => 'F',
                    game::Turn::Any(_) => 'A',
                }))
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
        let (new_state, outputs) = self.transition(&self.state, inp);
        self.state = new_state;
        let mut stream_outputs = vec![];
        for output in outputs {
            stream_outputs.extend(output.stream_outputs());
        }
        (stream_outputs, self.state.player_input(), self.state.has_ended())
    }

    fn transition(&self, s: &State, inp: &str) -> (State, Vec<Output>) {
        match *s {
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
                let game = Box::new(Game::new());
                let mut outputs = vec![];
                for p in 1..5 {
                     outputs.push(Output::Deal(p, game.hand(p)));
                }
                let turn = game.turn();
                println!("Game is starting. #{} to start.", turn.player());
                outputs.push(Output::Turn(turn));
                (State::Play(game), outputs)
            }
            State::Play(ref game) => {
                unimplemented!()
            }
            _ => unreachable!()
        }
    }
}
