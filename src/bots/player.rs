use std::collections::HashSet;

use game;

enum State {
    Start,
    WaitForId,
    WaitForCards,
    Game,
    MyTurn,
    MyPlay,
    End,
}

pub enum Status {
    ServerInput,
    UserInput,
    End,
}

impl State {
    fn status(&self) -> Status {
        match *self {
            State::Start
                | State::WaitForId
                | State::WaitForCards
                | State::Game
                | State::MyPlay
                => Status::ServerInput,
            State::MyTurn => Status::UserInput,
            State::End => Status::End,
        }
    }
}

#[derive(Debug)]
pub enum ServerInput {
    You(game::PlayerNum),
    Deal(game::Cards),
    Play(game::PlayerNum, game::Cards),
    Turn(game::Turn),
    Win(game::PlayerNum),
    End(Vec<game::PlayerNum>),
    InvalidInput(String),
    Error(String),
}

pub enum UserInput {
    Play(game::Cards),
}

pub enum ServerOutput {
    Game,
    Play(game::Cards),
}

pub struct Player {
    state: State,
    num: game::PlayerNum,
    hand: game::Cards,
    turn: Option<game::Turn>,
    last_play: Option<(game::PlayerNum, game::Cards)>,
    played: Option<game::Cards>,
}

pub fn new() -> Player {
    Player {
        state: State::Start,
        num: 0,
        hand: "".parse().unwrap(),
        turn: None,
        last_play: None,
        played: None,
    }
}

impl Player {

    pub fn actuate(&mut self, s_inp: Option<ServerInput>, u_inp: Option<UserInput>) -> (Option<ServerOutput>, Status) {
        // special handling, just ignore for now
        if let Some(ServerInput::Error(_)) = s_inp {
            return (None, self.state.status())
        }
        let (new_state, output) = match self.state {
            State::Start => (State::WaitForId, None),
            State::WaitForId => {
                let input = s_inp.unwrap();
                if let ServerInput::You(p) = input {
                    self.num = p;
                    (State::WaitForCards, Some(ServerOutput::Game))
                } else {
                    panic!("expected input You")
                }
            }
            State::WaitForCards => {
                let input = s_inp.unwrap();
                if let ServerInput::Deal(cards) = input {
                    self.hand = cards;
                    (State::Game, None)
                } else {
                    panic!("expected input Deal")
                }
            }
            State::Game => {
                let input = s_inp.unwrap();
                match input {
                    ServerInput::Play(p, cards) => {
                        if !cards.is_pass() {
                            self.last_play = Some((p, cards));
                        }
                        (State::Game, None)
                    }
                    ServerInput::Turn(turn) => {
                        self.turn = Some(turn);
                        match self.turn.unwrap() {
                            game::Turn::Start(p)
                                | game::Turn::Follow(p)
                                | game::Turn::Any(p)
                                if p == self.num => {
                                (State::MyTurn, None)
                            }
                            _ => (State::Game, None),
                        }
                    }
                    ServerInput::Win(_) => (State::Game, None),
                    ServerInput::End(_) => (State::End, None),
                    _ => panic!("unexpected input: {:?}", input)
                }
            }
            State::MyTurn => {
                let input = u_inp.unwrap();
                match input {
                    UserInput::Play(cards) => {
                        self.played = Some(cards.clone());
                        (State::MyPlay,
                         Some(ServerOutput::Play(cards)))
                    }
                }
            }
            State::MyPlay => {
                let input = s_inp.unwrap();
                match input {
                    ServerInput::Play(p, cards) => {
                        self.last_play = Some((p, cards));
                        let mut curr_cards: HashSet<_> =
                            self.hand.into_iter().collect();
                        for card in &self.played.take().unwrap() {
                            curr_cards.remove(&card);
                        }
                        self.hand = curr_cards.into_iter().collect();
                        (State::Game, None)
                    }
                    ServerInput::InvalidInput(_) => (State::MyTurn, None),
                    _ => panic!("unexpected input: {:?}", input)
                }
            }
            State::End => unreachable!(),
        };
        self.state = new_state;
        (output, self.state.status())
    }

    pub fn last_play(&self) -> &Option<(game::PlayerNum, game::Cards)> {
        &self.last_play
    }

    pub fn hand(&self) -> &game::Cards {
        &self.hand
    }

    pub fn hints(&self) -> Vec<game::Cards> {
        let mut compare = None;
        let mut start = false;
        match self.turn.unwrap() {
            game::Turn::Start(p) => {
                if p == self.num {
                    start = true;
                } else {
                    return vec![];
                }
            }
            game::Turn::Follow(_) => {
                compare = Some(&self.last_play.as_ref().unwrap().1);
            }
            game::Turn::Any(p) => {
                if p != self.num {
                    return vec![];
                }
            }
            game::Turn::End => unreachable!(),
        }
        let mut compare_value = None;
        if let Some(ref cards) = compare {
            compare_value = Some(cards.value().unwrap());
        }
        let mut hints = vec![];
        for mask in 1u32..(1<<self.hand.len()) {
            if let Some(ref cards) = compare {
                if mask.count_ones() != (cards.len() as u32) {
                    continue
                }
            }
            let mut cards = vec![];
            for i in 0..self.hand.len() {
                if mask&(1<<i) == 0 {
                    continue
                }
                cards.push(self.hand[i]);
            }
            if start && !cards.contains(&game::LOWEST_CARD) {
                continue
            }
            let card: game::Cards = cards.into_iter().collect();
            if let Ok(val) = card.value() {
                if let Some(val2) = compare_value {
                    if val > val2 {
                        hints.push(card);
                    }
                } else {
                    hints.push(card);
                }
            }
        }
        hints
    }
}
