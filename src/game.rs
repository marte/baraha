use std::fmt;
use std::cmp::Ordering;

use rand::{self, Rng};

const RANKS: &'static str = "3456789TJQKA2";
const SUITS: &'static str = "CSHD";

#[derive(Debug)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
pub struct Card {
    rank: char,
    suit: char,
}

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

struct Game {
}

impl Game {

}
