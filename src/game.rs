use std::fmt;
use std::cmp::Ordering;
use std::collections::HashSet;

use rand::{self, Rng};

const RANKS: &'static str = "3456789TJQKA2";
const SUITS: &'static str = "CSHD";

#[derive(Debug)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
#[derive(Hash)]
pub struct Card {
    rank: char,
    suit: char,
}

const LOWEST_CARD: Card = Card{rank: '3', suit: 'C'};

impl Card {
    fn value(&self) -> usize {
        RANKS.find(self.rank).unwrap()*4
            + SUITS.find(self.suit).unwrap()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Card) -> Ordering {
        self.value().cmp(&other.value())
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Card) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn new_deck() -> Vec<Card> {
    let mut d = vec![];
    for rank in RANKS.chars() {
        for suit in SUITS.chars() {
            d.push(Card{rank: rank, suit: suit});
        }
    }
    rand::thread_rng().shuffle(&mut d);
    d
}

pub type PlayerNum = usize;

type Hand = HashSet<Card>;

type Combi = HashSet<Card>;

pub enum Turn {
    Start(PlayerNum),
    Follow(PlayerNum),
    Any(PlayerNum),
}

impl Turn {
    pub fn player(&self) -> PlayerNum {
        match *self {
            Turn::Start(p) | Turn::Follow(p) | Turn::Any(p) => p
        }
    }
}

pub struct Game {
    curr_player: PlayerNum,
    hands: Vec<Hand>,
    discard_pile: HashSet<Card>,
    last_play: Option<(PlayerNum, Combi)>,
}

impl Game {
    pub fn new() -> Game {
        let mut game = Game {
            curr_player: 0,
            hands: vec![],
            discard_pile: HashSet::new(),
            last_play: None,
        };
        let mut deck = new_deck();
        for p in 1..5 {
            let hand: Hand = deck.drain(..13).collect();
            if hand.contains(&LOWEST_CARD) {
                assert_eq!(0, game.curr_player);
                game.curr_player = p;
            }
            game.hands.push(hand);
        }
        assert_ne!(0, game.curr_player);
        game
    }

    pub fn turn(&self) -> Turn {
        Turn::Start(self.curr_player)
    }

    pub fn hand(&self, p: PlayerNum) -> Vec<Card> {
        self.hands[p-1].iter().map(|x| *x).collect()
    }
}
