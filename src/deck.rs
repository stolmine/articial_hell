use rand::seq::SliceRandom;

use crate::card::{Card, Rank, Suit};

#[derive(Debug)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new_scoundrel() -> Self {
        let mut cards = Vec::with_capacity(44);

        for &suit in &[Suit::Clubs, Suit::Spades] {
            for &rank in &[
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
                Rank::Ace,
            ] {
                cards.push(Card { suit, rank });
            }
        }

        for &suit in &[Suit::Diamonds, Suit::Hearts] {
            for &rank in &[
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
            ] {
                cards.push(Card { suit, rank });
            }
        }

        Self { cards }
    }

    pub fn shuffle(&mut self, rng: &mut impl rand::Rng) {
        self.cards.shuffle(rng);
    }

    pub fn draw(&mut self, n: usize) -> Vec<Card> {
        let split = self.cards.len().saturating_sub(n);
        self.cards.split_off(split)
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn push_many(&mut self, cards: Vec<Card>) {
        self.cards.extend(cards);
    }
}
