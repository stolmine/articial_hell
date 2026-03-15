use rand::seq::SliceRandom;

use crate::card::{CourtRank, MajorArcana, MinorSuit, TarotCard};

#[derive(Debug, Clone)]
pub struct TarotDeck {
    pub court: Vec<TarotCard>,
    pub numbered: Vec<TarotCard>,
    pub arcana: Vec<TarotCard>,
}

impl TarotDeck {
    pub fn new() -> Self {
        let mut court = Vec::with_capacity(16);
        let mut numbered = Vec::with_capacity(40);
        let mut arcana = Vec::with_capacity(22);

        for suit in MinorSuit::ALL {
            for rank in CourtRank::ALL {
                court.push(TarotCard::Court { suit, rank });
            }
            for value in 1..=10 {
                numbered.push(TarotCard::Numbered { suit, value });
            }
        }

        for a in MajorArcana::ALL {
            arcana.push(TarotCard::Major(a));
        }

        Self { court, numbered, arcana }
    }

    pub fn shuffle_all(&mut self, rng: &mut impl rand::Rng) {
        self.court.shuffle(rng);
        self.numbered.shuffle(rng);
        self.arcana.shuffle(rng);
    }

    pub fn draw_court(&mut self, n: usize) -> Vec<TarotCard> {
        draw_n(&mut self.court, n)
    }

    pub fn draw_numbered(&mut self, n: usize) -> Vec<TarotCard> {
        draw_n(&mut self.numbered, n)
    }

    pub fn draw_arcana(&mut self, n: usize) -> Vec<TarotCard> {
        draw_n(&mut self.arcana, n)
    }
}

fn draw_n(pool: &mut Vec<TarotCard>, n: usize) -> Vec<TarotCard> {
    let split = pool.len().saturating_sub(n);
    pool.split_off(split)
}
