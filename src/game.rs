use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::card::Card;
use crate::deck::Deck;

#[derive(Debug, Clone)]
pub struct WeaponState {
    pub card: Card,
    pub bound_to: Option<u8>,
}

impl WeaponState {
    fn can_fight(&self, monster_value: u8) -> bool {
        match self.bound_to {
            None => true,
            Some(bound) => bound >= monster_value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    Title,
    Playing,
    Won,
    Dead,
}

#[derive(Debug)]
pub struct GameState {
    pub hp: u32,
    pub max_hp: u32,
    pub deck: Deck,
    pub room: Vec<Option<Card>>,
    pub held_card: Option<Card>,
    pub weapon: Option<WeaponState>,
    pub used_potion_this_room: bool,
    pub can_run: bool,
    pub phase: GamePhase,
    pub cards_resolved: u8,
    pub room_size: usize,
    pub message: String,
    rng: ChaCha8Rng,
}

impl GameState {
    pub fn new_title() -> Self {
        Self {
            hp: 20,
            max_hp: 20,
            deck: Deck::new_scoundrel(),
            room: vec![None; 4],
            held_card: None,
            weapon: None,
            used_potion_this_room: false,
            can_run: true,
            phase: GamePhase::Title,
            cards_resolved: 0,
            room_size: 0,
            message: String::new(),
            rng: ChaCha8Rng::from_rng(&mut rand::rng()),
        }
    }

    pub fn new_game() -> Self {
        let mut rng = ChaCha8Rng::from_rng(&mut rand::rng());
        let mut deck = Deck::new_scoundrel();
        deck.shuffle(&mut rng);

        let mut state = Self {
            hp: 20u32,
            max_hp: 20u32,
            deck,
            room: vec![None; 4],
            held_card: None,
            weapon: None,
            used_potion_this_room: false,
            can_run: true,
            phase: GamePhase::Playing,
            cards_resolved: 0,
            room_size: 0,
            message: String::new(),
            rng,
        };

        state.deal_room();
        state
    }

    pub fn deal_room(&mut self) {
        self.used_potion_this_room = false;
        self.cards_resolved = 0;
        self.room = vec![None; 4];

        let held = self.held_card.take();
        let need = if held.is_some() { 3 } else { 4 };
        let available = self.deck.remaining().min(need);
        let mut drawn = self.deck.draw(available);

        let mut dealt: Vec<Card> = Vec::new();
        if let Some(h) = held {
            dealt.push(h);
        }
        dealt.append(&mut drawn);

        self.room_size = dealt.len();
        for (i, card) in dealt.into_iter().enumerate() {
            self.room[i] = Some(card);
        }
    }

    pub fn resolve_card(&mut self, index: usize) {
        if self.phase != GamePhase::Playing {
            return;
        }
        if index > 3 || self.room[index].is_none() {
            return;
        }

        let remaining_count = self.room.iter().filter(|s| s.is_some()).count();
        let is_final_room = self.deck.is_empty() && self.held_card.is_none();

        if !is_final_room && remaining_count == 1 {
            self.message = "That card must be held for the next room.".to_string();
            return;
        }

        let card = match self.room[index].take() {
            Some(c) => c,
            None => return,
        };

        if card.is_potion() && self.used_potion_this_room {
            self.room[index] = Some(card);
            self.message = "Already used a potion this room!".to_string();
            return;
        }

        if card.is_monster() {
            self.fight(card);
        } else if card.is_weapon() {
            self.equip_weapon(card);
        } else if card.is_potion() {
            self.use_potion(card);
        }

        if self.hp == 0 {
            self.phase = GamePhase::Dead;
            return;
        }

        self.cards_resolved += 1;
        self.finish_room();
    }

    fn fight(&mut self, monster: Card) {
        let damage: u32;

        if let Some(ref mut weapon) = self.weapon {
            if weapon.can_fight(monster.value()) {
                damage = monster.value().saturating_sub(weapon.card.value()) as u32;
                self.message = format!(
                    "Fought {} with {} — took {} damage.",
                    monster,
                    weapon.card,
                    damage
                );
                weapon.bound_to = Some(monster.value());
            } else {
                damage = monster.value() as u32;
                self.message = format!(
                    "Weapon can't fight {} (bound to {}). Bare-handed — took {} damage.",
                    monster,
                    weapon.bound_to.unwrap_or(0),
                    damage
                );
            }
        } else {
            damage = monster.value() as u32;
            self.message = format!("Fought {} bare-handed — took {} damage.", monster, damage);
        }

        self.hp = self.hp.saturating_sub(damage);
    }

    fn equip_weapon(&mut self, card: Card) {
        self.message = format!("Equipped {}.", card);
        self.weapon = Some(WeaponState {
            card,
            bound_to: None,
        });
    }

    fn use_potion(&mut self, card: Card) {
        let new_hp = (self.hp + card.value() as u32).min(self.max_hp);
        let healed = new_hp - self.hp;
        self.hp = new_hp;
        self.used_potion_this_room = true;
        self.message = format!(
            "Used {} — healed {} HP ({}/{}).",
            card, healed, self.hp, self.max_hp
        );
    }

    pub fn run(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }
        if !self.can_run {
            self.message = "Cannot run from this room!".to_string();
            return;
        }

        let remaining: Vec<Card> = self.room.iter_mut().filter_map(|s| s.take()).collect();
        self.deck.push_many(remaining);
        self.deck.shuffle(&mut self.rng);

        self.held_card = None;
        self.can_run = false;
        self.message = "You ran! Cards shuffled back into the deck.".to_string();

        self.deal_room();
    }

    fn finish_room(&mut self) {
        let remaining: Vec<(usize, Card)> = self
            .room
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.map(|c| (i, c)))
            .collect();

        let is_final_room = self.deck.is_empty() && self.held_card.is_none();

        if is_final_room {
            if remaining.is_empty() {
                self.phase = GamePhase::Won;
                self.message = "You survived the dungeon! You won!".to_string();
            }
            return;
        }

        if remaining.len() == 1 {
            let (idx, card) = remaining[0];
            self.room[idx] = None;
            self.held_card = Some(card);
            self.can_run = true;
            self.deal_room();
        }
    }
}
