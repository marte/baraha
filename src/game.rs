use std::fmt;
use std::cmp::Ordering;
use std::collections::{HashSet, BTreeSet};
use std::str::FromStr;
use std::iter::FromIterator;
use std::ops::Index;

use rand::{self, Rng};

const RANKS: &'static str = "3456789TJQKA2";
const SUITS: &'static str = "CSHD";

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Card {
    pub rank: char,
    pub suit: char,
}

type Value = usize;

pub const LOWEST_CARD: Card = Card{rank: '3', suit: 'C'};

enum Combi {
    None,
    Straight,
    Flush,
    FullHouse,
    Quadro,
    StraightFlush,
}

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
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            Err("must be of length 2".into())
        } else {
            let mut chars = s.chars();
            let card = Card{
                rank: chars.next().unwrap(),
                suit: chars.next().unwrap(),
            };
            if RANKS.find(card.rank).is_none() {
                Err(format!("invalid rank {}", card.rank))
            } else if SUITS.find(card.suit).is_none() {
                Err(format!("invalid suit {}", card.suit))
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

#[derive(Debug, Clone)]
pub struct Cards(Vec<Card>, Value);

impl fmt::Display for Cards {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str_cards: Vec<_> = self.0.iter()
            .map(|c| c.to_string())
            .collect();
        write!(f, "{}", str_cards.join(" "))
    }
}

impl FromStr for Cards {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cards = vec![];
        for c in s.split_whitespace() {
            let card = Card::from_str(c)?;
            if cards.contains(&card) {
                return Err("cards are not unique".into())
            }
            cards.push(card);
        }
        let val = Cards::value(cards.clone())?;
        Ok(Cards(cards, val))
    }
}

impl<'a> IntoIterator for &'a Cards {
    type Item = Card;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.clone().into_iter()
    }
}

impl Cards {
    pub fn new(cards: Vec<Card>) -> Result<Cards, String> {
        let val = Cards::value(cards.clone())?;
        Ok(Cards(cards, val))
    }

    pub fn is_pass(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn value(mut cards: Vec<Card>) -> Result<Value, String> {
        cards.sort();
        let cards = cards.as_slice();

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
                    // Use the value of the highest (by num) card.
                    let card = cards.iter().find(|&c| num(*c) == nums[4]).unwrap();
                    return Some(card.value())
                } else if nums[0] == 0 {
                    // Let Ace be a high card.
                    nums[0] = 13;
                    nums.sort();
                    if nums[0]+4 == nums[4] {
                        // Use the value of the Ace card.
                        let card = cards.iter().find(|&c| num(*c) == 0).unwrap();
                        return Some(card.value())
                    }
                }
            }
            None
        }

        fn flush(cards: &[Card]) -> Option<Value> {
            if cards.iter().all(|c| c.suit == cards[0].suit) {
                // order by suit then by rank
                Some(SUITS.find(cards[4].suit).unwrap()*13
                     + RANKS.find(cards[4].rank).unwrap())
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

        let res = match cards.len() {
            0 => Ok(0),
            1 => Ok(cards[0].value()),
            2 => {
                if is_same_rank(cards) {
                    Ok(cards[1].value())
                } else {
                    Err("pair doesn't match".into())
                }
            }
            3 => {
                if is_same_rank(cards) {
                    Ok(cards[0].value())
                } else {
                    Err("trio doesn't match".into())
                }
            }
            5 => {
                let (combi, val) = if let Some(val) = straight(cards) {
                    if let Some(_) = flush(cards) {
                        (Combi::StraightFlush, val)
                    } else {
                        (Combi::Straight, val)
                    }
                } else {
                    if let Some(val) = quadro(cards) {
                        (Combi::Quadro, val)
                    } else if let Some(val) = full_house(cards) {
                        (Combi::FullHouse, val)
                    } else if let Some(val) = flush(cards) {
                        (Combi::Flush, val)
                    } else {
                        (Combi::None, 0)
                    }
                };
                match combi {
                    Combi::None => Err("invalid 5-card combination".into()),
                    _ => Ok((combi as Value)*1000 + val)
                }
            }
            _ => {
                Err("invalid length".into())
            }
        };
        res
    }
}

impl Ord for Cards {
    fn cmp(&self, other: &Cards) -> Ordering {
        let (len1, len2) = (self.len(), other.len());
        if len1 == len2 {
            self.1.cmp(&other.1)
        } else {
            // Comparing cards with different cardinalities doesn't
            // really make sense, we just do it here for convenience
            // (e.g. in sorting).
            len2.cmp(&len1)
        }
    }
}

impl PartialOrd for Cards {
    fn partial_cmp(&self, other: &Cards) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Cards {
    fn eq(&self, other: &Cards) -> bool {
        self.1 == other.1
    }
}
impl Eq for Cards {}

impl Index<usize> for Cards {
    type Output = Card;
    fn index(&self, index: usize) -> &Card {
        self.0.index(index)
    }
}

pub type PlayerNum = usize;

#[derive(Debug, Copy, Clone)]
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
    hands: Vec<HashSet<Card>>,
    discard_pile: Vec<Card>,
    last_play: Option<(PlayerNum, Cards)>,
    winners: Vec<PlayerNum>,
}

impl Game {
    pub fn new() -> Game {
        let mut game = Game {
            curr_player: 0,
            hands: vec![],
            discard_pile: vec![],
            last_play: None,
            winners: vec![],
        };
        let mut deck = new_deck();
        for p in 1..5 {
            let hand: HashSet<_> = deck.drain(..13).collect();
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

    pub fn hand(&self, p: PlayerNum) -> Vec<Card> {
        self.hands[p-1].iter().cloned().collect()
    }

    pub fn play(&mut self, cards: &Cards) -> Result<bool, String> {
        let t = self.turn();
        if !self.is_in_hand(t.player(), cards) {
            return Err("some cards are not in player's hands".into())
        }
        match t {
            Turn::Start(_) => {
                if cards.is_pass() {
                    return Err("cannot pass".into())
                } else if !cards.0.contains(&LOWEST_CARD) {
                    return Err("first play must include three of clubs".into())
                }
            }
            Turn::Follow(_) => {
                if !cards.is_pass() {
                    let last_cards = &self.last_play.as_ref().unwrap().1;
                    if last_cards.0.len() != cards.0.len() {
                        return Err("should follow the number of cards \
                                    of last play".into())
                    }
                    if cards <= last_cards {
                        return Err("played cards are lower than last".into())
                    }
                }
            }
            Turn::Any(_) => {
                if cards.is_pass() {
                    return Err("cannot pass".into())
                }
            }
            Turn::End => unreachable!(),
        }
        for card in &cards.0 {
            self.hands[t.player()-1].remove(card);
            self.discard_pile.push(*card);
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


#[cfg(test)]
mod tests {
    use super::*;

    fn gt(c1: Cards, c2: Cards) -> bool {
        c1 > c2
    }

    #[test]
    fn straight_flush() {
        // The high card in these cases are not the ace or two.
        let c1 = "AD 2D 3D 4D 5D".parse().unwrap();
        let c2 = "2D 3D 4D 5D 6D".parse().unwrap();
        assert!(gt(c2, c1));
        let c1 = "2D 3D 4D 5D 6D".parse().unwrap();
        let c2 = "3D 4D 5D 6D 7D".parse().unwrap();
        assert!(gt(c2, c1));

        let c1 = "9D TD JD QD KD".parse().unwrap();
        // This time the high card is the Ace.
        let c2 = "AD KD QD JD TD".parse().unwrap();
        assert!(gt(c2, c1));
    }

    #[test]
    fn straight() {
        // The high card in these cases are not the ace or two.
        let c1 = "AC 2D 3D 4D 5D".parse().unwrap();
        let c2 = "2C 3D 4D 5D 6D".parse().unwrap();
        assert!(gt(c2, c1));
        let c1 = "2C 3D 4D 5D 6D".parse().unwrap();
        let c2 = "3C 4D 5D 6D 7D".parse().unwrap();
        assert!(gt(c2, c1));

        let c1 = "9C TD JD QD KD".parse().unwrap();
        // This time the high card is the Ace.
        let c2 = "AC KD QD JD TD".parse().unwrap();
        assert!(gt(c2, c1));

        // High card rank tied, break by suit.
        let c1 = "AC KD QD JD TD".parse().unwrap();
        let c2 = "AS KH QH JH TH".parse().unwrap();
        assert!(gt(c2, c1));
    }

    #[test]
    fn flush() {
        // Suit is more important than high card.
        let c1 = "AC 2C 3C 4C 6C".parse().unwrap();
        let c2 = "9D KD 8D JD TD".parse().unwrap();
        assert!(gt(c2, c1));

        // If suit is tied, then high card is used as breaker.
        let c1 = "AD KD 8D JD TD".parse().unwrap();
        let c2 = "9D 2D 3D 4D 6D".parse().unwrap();
        assert!(gt(c2, c1));
    }
}
