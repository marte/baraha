use std::fmt;
use std::cmp::Ordering;
use std::collections::{HashSet, BTreeSet};
use std::str::FromStr;
use std::iter::FromIterator;

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

type Value = usize;

const LOWEST_CARD: Card = Card{rank: '3', suit: 'C'};

const STRAIGHT_FLUSH: Value = 5;
const QUADRO: Value = 4;
const FULL_HOUSE: Value = 3;
const FLUSH: Value = 2;
const STRAIGHT: Value = 1;
const NO_COMBI: Value = 0;

impl Card {
    fn value(&self) -> Value {
        RANKS.find(self.rank).unwrap()*4
            + SUITS.find(self.suit).unwrap()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl FromStr for Card {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            Err("must be of length 2")
        } else {
            let mut chars = s.chars();
            let card = Card{
                rank: chars.next().unwrap(),
                suit: chars.next().unwrap(),
            };
            if RANKS.find(card.rank).is_none() {
                Err("invalid rank")
            } else if SUITS.find(card.suit).is_none() {
                Err("invalid suit")
            } else {
                Ok(card)
            }
        }
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

#[derive(Clone)]
pub struct Cards(Vec<Card>);

impl fmt::Display for Cards {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str_cards: Vec<_> = self.0.iter()
            .map(|c| c.to_string())
            .collect();
        write!(f, "{}", str_cards.join(" "))
    }
}

impl FromStr for Cards {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cards = Cards(vec![]);
        for c in s.split_whitespace() {
            let card = try!{Card::from_str(c)};
            if cards.0.contains(&card) {
                return Err("cards are not unique")
            }
            cards.0.push(card);
        }
        Ok(cards)
    }
}

impl Cards {
    fn value(&self) -> Result<Value, &'static str> {
        fn is_same_rank(cards: &[Card]) -> bool {
            cards[0].rank == cards[cards.len()-1].rank
        }

        fn straight(cards: &[Card]) -> Option<Value> {
            fn num(card: Card) -> Value {
                let r = RANKS.find(card.rank).unwrap();
                (r + 2) % 13 // shift it so that A = 0, 2 = 1, etc.
            }
            let num_set = BTreeSet::from_iter(cards.iter().map(|&c| num(c)));
            if num_set.len() == 5 {
                let mut nums: Vec<Value> = num_set.iter().cloned().collect();
                if nums[0]+4 == nums[4] {
                    return Some(nums[4])
                } else if nums[0] == 0 {
                    // let Ace be a high card
                    nums[0] = 13;
                    nums.sort();
                    if nums[0]+4 == nums[4] {
                        return Some(nums[4])
                    }
                }
            }
            None
        }

        fn flush(cards: &[Card]) -> Option<Value> {
            if cards.iter().all(|c| c.suit == cards[0].suit) {
                Some(SUITS.find(cards[0].suit).unwrap())
            } else {
                None
            }
        }

        fn quadro(cards: &[Card]) -> Option<Value> {
            if is_same_rank(&cards[0..4]) || is_same_rank(&cards[1..5]) {
                Some(cards[1].value())
            } else {
                None
            }
        }

        fn full_house(cards: &[Card]) -> Option<Value> {
            if is_same_rank(&cards[0..2]) && is_same_rank(&cards[2..5]) {
                Some(cards[2].value())
            } else if is_same_rank(&cards[0..3]) && is_same_rank(&cards[3..5]) {
                Some(cards[2].value())
            } else {
                None
            }
        }

        let cards: &mut [Card] = &mut self.0.clone();
        cards.sort();

        match cards.len() {
            0 => Ok(0),
            1 => Ok(cards[0].value()),
            2 => {
                if is_same_rank(cards) {
                    Ok(cards[1].value())
                } else {
                    Err("pair doesn't match")
                }
            }
            3 => {
                if is_same_rank(cards) {
                    Ok(cards[0].value())
                } else {
                    Err("trio doesn't match")
                }
            }
            5 => {
                let (combi, val) = if let Some(s_val) = straight(cards) {
                    if let Some(f_val) = flush(cards) {
                        (STRAIGHT_FLUSH, s_val*4 + f_val)
                    } else {
                        (STRAIGHT, s_val)
                    }
                } else {
                    if let Some(val) = quadro(cards) {
                        (QUADRO, val)
                    } else if let Some(val) = full_house(cards) {
                        (FULL_HOUSE, val)
                    } else if let Some(val) = flush(cards) {
                        (FLUSH, val)
                    } else {
                        (NO_COMBI, 0)
                    }
                };
                if combi == NO_COMBI {
                    Err("invalid 5-card combination")
                } else {
                    Ok(combi*1000 + val)
                }
            }
            _ => {
                Err("invalid length")
            }
        }
    }

    fn is_pass(&self) -> bool {
        self.0.is_empty()
    }
}

pub type PlayerNum = usize;

type Hand = HashSet<Card>;

pub enum Turn {
    Start(PlayerNum),
    Follow(PlayerNum),
    Any(PlayerNum),
    End,
}

impl Turn {
    pub fn player(&self) -> PlayerNum {
        match *self {
            Turn::Start(p) | Turn::Follow(p) | Turn::Any(p) => p,
            Turn::End => unreachable!(),
        }
    }
}

pub struct Game {
    curr_player: PlayerNum,
    hands: Vec<Hand>,
    discard_pile: HashSet<Card>,
    last_play: Option<(PlayerNum, Cards)>,
    winners: Vec<PlayerNum>,
}

impl Game {
    pub fn new() -> Game {
        let mut game = Game {
            curr_player: 0,
            hands: vec![],
            discard_pile: HashSet::new(),
            last_play: None,
            winners: vec![],
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
        if self.winners.len() == 3 {
            Turn::End
        } else if let Some((p, _)) = self.last_play {
            if p == self.curr_player || self.hands[p-1].is_empty() {
                Turn::Any(self.curr_player)
            } else {
                Turn::Follow(self.curr_player)
            }
        } else {
            Turn::Start(self.curr_player)
        }
    }

    pub fn hand(&self, p: PlayerNum) -> Cards {
        Cards(self.hands[p-1].iter().map(|x| *x).collect())
    }

    pub fn play(&mut self, cards: &Cards) -> Result<bool, &'static str> {
        let t = self.turn();
        if !self.is_in_hand(t.player(), cards) {
            return Err("some cards are not in player's hands")
        }
        let val = try!{cards.value()};
        match t {
            Turn::Start(_) => {
                if cards.is_pass() {
                    return Err("cannot pass")
                } else if !cards.0.contains(&LOWEST_CARD) {
                    return Err("first play must include three of clubs")
                }
            }
            Turn::Follow(_) => {
                if !cards.is_pass() {
                    let last_cards = &self.last_play.as_ref().unwrap().1;
                    if last_cards.0.len() != cards.0.len() {
                        return Err("should follow the cardinality of last play")
                    }
                    let last_val = last_cards.value().unwrap();
                    if val <= last_val {
                        return Err("played cards are lower than last")
                    }
                }
            }
            Turn::Any(_) => {
                if cards.is_pass() {
                    return Err("cannot pass")
                }
            }
            Turn::End => unreachable!(),
        }
        for card in &cards.0 {
            self.hands[t.player()-1].remove(card);
            self.discard_pile.insert(*card);
        }
        if !cards.is_pass() {
            self.last_play = Some((t.player(), cards.clone()));
        }
        let mut wins = false;
        if self.hands[t.player()-1].is_empty() {
            self.winners.push(t.player());
            wins = true;
        }
        self.inc_turn();
        Ok(wins)
    }

    pub fn winners(&self) -> Vec<PlayerNum> {
        self.winners.clone()
    }

    fn is_in_hand(&self, p: PlayerNum, cards: &Cards) -> bool {
        self.hands[p-1].is_superset(&cards.0.iter().cloned().collect())
    }

    fn inc_turn(&mut self) {
        if self.winners.len() == 3 {
            return
        }
        loop {
            self.curr_player = (self.curr_player)%4 + 1;
            if !self.hands[self.curr_player-1].is_empty() {
                break
            }
        }
    }
}
