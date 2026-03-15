use crate::card::{CourtRank, MinorSuit, TarotCard};

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub attack: i32,
    pub speed: i32,
    pub hp: i32,
    pub defense: i32,
}

impl Stats {
    pub fn add(&mut self, other: &Stats) {
        self.attack += other.attack;
        self.speed += other.speed;
        self.hp += other.hp;
        self.defense += other.defense;
    }

    pub fn add_flat(&mut self, amount: i32) {
        self.attack += amount;
        self.speed += amount;
        self.hp += amount;
        self.defense += amount;
    }
}

pub fn hero_base_stats(suit: MinorSuit) -> Stats {
    match suit {
        MinorSuit::Swords => Stats { attack: 8, speed: 5, hp: 14, defense: 3 },
        MinorSuit::Wands => Stats { attack: 4, speed: 8, hp: 14, defense: 4 },
        MinorSuit::Cups => Stats { attack: 4, speed: 4, hp: 20, defense: 2 },
        MinorSuit::Pentacles => Stats { attack: 3, speed: 3, hp: 14, defense: 10 },
    }
}

pub fn equipment_primary(slot: EquipSlot, value: u8) -> Stats {
    let v = value as i32;
    match slot {
        EquipSlot::Weapon => Stats { attack: v, ..Default::default() },
        EquipSlot::Apparel => Stats { defense: v, ..Default::default() },
        EquipSlot::Item => Stats { hp: v, ..Default::default() },
    }
}

pub fn equipment_secondary(suit: MinorSuit, value: u8) -> Stats {
    let bonus = (value as i32 / 3).max(1);
    match suit {
        MinorSuit::Swords => Stats { attack: bonus, ..Default::default() },
        MinorSuit::Wands => Stats { speed: bonus, ..Default::default() },
        MinorSuit::Cups => Stats { hp: bonus, ..Default::default() },
        MinorSuit::Pentacles => Stats { defense: bonus, ..Default::default() },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipSlot {
    Weapon,
    Apparel,
    Item,
}

pub fn count_unique_suits(cards: &[TarotCard]) -> usize {
    let mut suits: Vec<MinorSuit> = cards.iter().filter_map(|c| c.suit()).collect();
    suits.sort_by_key(|s| *s as u8);
    suits.dedup();
    suits.len()
}

pub fn rank_bonus(hero_rank: CourtRank, hero_suit: MinorSuit, equipment: &[TarotCard]) -> Stats {
    let unique_suits = count_unique_suits(equipment) as i32;
    let matching_suits = equipment
        .iter()
        .filter(|c| c.suit() == Some(hero_suit))
        .count() as i32;

    let bonus = match hero_rank {
        CourtRank::Page => unique_suits * 3,
        CourtRank::Knight => unique_suits * 2,
        CourtRank::Queen => matching_suits * 2,
        CourtRank::King => matching_suits * 3,
    };

    let mut s = Stats::default();
    s.add_flat(bonus);
    s
}

pub fn derive_stats(
    hero: TarotCard,
    weapon: TarotCard,
    apparel: TarotCard,
    item: TarotCard,
) -> Stats {
    partial_derive(Some(hero), Some(weapon), Some(apparel), Some(item))
}

pub fn partial_derive(
    hero: Option<TarotCard>,
    weapon: Option<TarotCard>,
    apparel: Option<TarotCard>,
    item: Option<TarotCard>,
) -> Stats {
    let hero = match hero {
        Some(h) => h,
        None => return Stats::default(),
    };
    let hero_suit = hero.suit().unwrap_or(MinorSuit::Swords);
    let hero_rank = hero.court_rank().unwrap_or(CourtRank::Page);

    let mut stats = hero_base_stats(hero_suit);

    let equip: [(EquipSlot, Option<TarotCard>); 3] = [
        (EquipSlot::Weapon, weapon),
        (EquipSlot::Apparel, apparel),
        (EquipSlot::Item, item),
    ];

    for (slot, card) in &equip {
        if let Some(c) = card {
            if let Some(suit) = c.suit() {
                let val = c.numbered_value().unwrap_or(1);
                stats.add(&equipment_primary(*slot, val));
                stats.add(&equipment_secondary(suit, val));
            }
        }
    }

    let equip_cards: Vec<TarotCard> = [weapon, apparel, item].iter().filter_map(|o| *o).collect();
    if !equip_cards.is_empty() {
        stats.add(&rank_bonus(hero_rank, hero_suit, &equip_cards));
    }

    stats.attack = stats.attack.max(1);
    stats.speed = stats.speed.max(1);
    stats.hp = stats.hp.max(1);
    stats.defense = stats.defense.max(0);

    stats
}
