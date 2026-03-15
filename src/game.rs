use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use crate::card::TarotCard;
use crate::deck::TarotDeck;
use crate::combat::{CombatState, CombatAction, Fighter, Side};
use crate::theme::Theme;
use crate::ai::{self, AiPersonality};
use crate::progression;
use crate::stats::Stats;

pub const MAX_FIGHTS: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DraftStep {
    PickHero,
    PickWeapon,
    PickApparel,
    PickItem,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub hero: Option<TarotCard>,
    pub weapon: Option<TarotCard>,
    pub apparel: Option<TarotCard>,
    pub item: Option<TarotCard>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self { hero: None, weapon: None, apparel: None, item: None }
    }

    pub fn set_slot(&mut self, step: &DraftStep, card: TarotCard) {
        match step {
            DraftStep::PickHero => self.hero = Some(card),
            DraftStep::PickWeapon => self.weapon = Some(card),
            DraftStep::PickApparel => self.apparel = Some(card),
            DraftStep::PickItem => self.item = Some(card),
        }
    }
}

#[derive(Clone, Debug)]
pub enum GamePhase {
    Title,
    Draft {
        step: DraftStep,
        choices: Vec<TarotCard>,
        ai_choices: Vec<TarotCard>,
    },
    DraftReveal {
        step: DraftStep,
    },
    Combat,
    GameOver {
        victory: bool,
    },
}

pub const QUEEN_PERMUTATIONS: [[usize; 3]; 6] = [
    [0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0],
];

pub struct GameState {
    pub phase: GamePhase,
    pub player: PlayerState,
    pub ai_state: PlayerState,
    pub player_deck: TarotDeck,
    pub ai_deck: TarotDeck,
    pub combat: Option<CombatState>,
    pub fight: usize,
    pub fights_won: u8,
    pub message: String,
    pub cursor: usize,
    pub theme: Theme,
    pub ai_personality: Option<AiPersonality>,
    rng: ChaCha8Rng,
    pub queen_perm_index: usize,
    pub campaign: Option<progression::ProgressionState>,
}

impl GameState {
    pub fn new_title() -> Self {
        Self {
            phase: GamePhase::Title, player: PlayerState::new(), ai_state: PlayerState::new(),
            player_deck: TarotDeck::new(), ai_deck: TarotDeck::new(),
            combat: None, fight: 1, fights_won: 0,
            message: String::new(),
            cursor: 0, theme: crate::theme::detect_theme(),
            ai_personality: None,
            rng: ChaCha8Rng::from_rng(&mut rand::rng()),
            queen_perm_index: 0,
            campaign: None,
        }
    }

    pub fn new_game() -> Self {
        let rng = ChaCha8Rng::from_rng(&mut rand::rng());
        let mut state = Self {
            phase: GamePhase::Title, player: PlayerState::new(), ai_state: PlayerState::new(),
            player_deck: TarotDeck::new(), ai_deck: TarotDeck::new(),
            combat: None, fight: 1, fights_won: 0,
            message: String::new(),
            cursor: 0, theme: crate::theme::detect_theme(),
            ai_personality: None, rng,
            queen_perm_index: 0,
            campaign: Some(progression::ProgressionState::default()),
        };
        state.start_fight();
        state
    }

    fn start_fight(&mut self) {
        self.player = PlayerState::new();
        self.ai_state = PlayerState::new();
        self.combat = None;
        self.ai_personality = None;

        // Fresh decks each fight
        self.player_deck = TarotDeck::new();
        self.ai_deck = TarotDeck::new();
        self.player_deck.shuffle_all(&mut self.rng);
        self.ai_deck.shuffle_all(&mut self.rng);

        let choices = self.player_deck.draw_court(4);
        let ai_choices = self.ai_deck.draw_court(4);
        self.phase = GamePhase::Draft {
            step: DraftStep::PickHero,
            choices,
            ai_choices,
        };
        self.message = format!("Fight {}/{} — Draft your Hero.", self.fight, MAX_FIGHTS);
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let max = match &self.phase {
            GamePhase::Draft { choices, .. } => choices.len(),
            _ => return,
        };
        if max == 0 { return; }
        self.cursor = ((self.cursor as i32 + delta).rem_euclid(max as i32)) as usize;
    }

    pub fn draft_pick(&mut self, index: usize) {
        let (step, choices, ai_choices) = match &self.phase {
            GamePhase::Draft { step, choices, ai_choices } => {
                (step.clone(), choices.clone(), ai_choices.clone())
            }
            _ => return,
        };

        if index >= choices.len() {
            return;
        }

        let card = choices[index];
        self.player.set_slot(&step, card);

        // Record progression
        if let Some(ref mut prog) = self.campaign {
            if step == DraftStep::PickHero {
                progression::record_hero_pick(prog, card);
            } else {
                progression::record_equipment_pick(prog, card);
            }
        }

        let ai_idx = ai::draft_pick(&ai_choices, &step, &self.ai_state, self.ai_personality.as_ref(), &mut self.rng, None);
        let ai_card = ai_choices[ai_idx];
        self.ai_state.set_slot(&step, ai_card);
        if step == DraftStep::PickHero {
            self.ai_personality = Some(AiPersonality::from_hero(ai_card));
        }

        self.phase = GamePhase::DraftReveal { step };
    }

    pub fn advance_from_reveal(&mut self) {
        let step = match &self.phase {
            GamePhase::DraftReveal { step } => step.clone(),
            _ => return,
        };

        let next_step = match step {
            DraftStep::PickHero => Some(DraftStep::PickWeapon),
            DraftStep::PickWeapon => Some(DraftStep::PickApparel),
            DraftStep::PickApparel => Some(DraftStep::PickItem),
            DraftStep::PickItem => None,
        };

        self.cursor = 0;
        match next_step {
            None => self.start_combat(),
            Some(step) => {
                let label = match &step {
                    DraftStep::PickWeapon => "Pick your Weapon.",
                    DraftStep::PickApparel => "Pick your Apparel.",
                    DraftStep::PickItem => "Pick your Item.",
                    _ => "Draft.",
                };
                let choices = self.player_deck.draw_numbered(4);
                let ai_choices = self.ai_deck.draw_numbered(4);
                self.message = label.to_string();
                self.phase = GamePhase::Draft { step, choices, ai_choices };
            }
        }
    }

    fn start_combat(&mut self) {
        let mut player_fighter = Fighter::new(
            self.player.hero.unwrap(),
            self.player.weapon.unwrap(),
            self.player.apparel.unwrap(),
            self.player.item.unwrap(),
        );
        let mut ai_fighter = Fighter::new(
            self.ai_state.hero.unwrap(),
            self.ai_state.weapon.unwrap(),
            self.ai_state.apparel.unwrap(),
            self.ai_state.item.unwrap(),
        );

        let mut p_delta = Stats::default();
        let mut a_delta = Stats::default();

        if let Some(ref prog) = self.campaign {
            let equip = [
                self.player.weapon.unwrap(),
                self.player.apparel.unwrap(),
                self.player.item.unwrap(),
            ];
            p_delta = progression::progression_bonus(prog, self.player.hero.unwrap(), &equip);
            player_fighter.stats.add(&p_delta);
            player_fighter.max_hp = player_fighter.stats.hp;
            player_fighter.current_hp = player_fighter.max_hp;

            let ai_bonus = progression::ai_scaling_bonus(prog)
                + progression::ai_boss_bonus(self.fight, MAX_FIGHTS);
            if ai_bonus > 0 {
                a_delta.add_flat(ai_bonus);
                ai_fighter.stats.add_flat(ai_bonus);
                ai_fighter.max_hp = ai_fighter.stats.hp;
                ai_fighter.current_hp = ai_fighter.max_hp;
            }
        }

        let combat_rng = ChaCha8Rng::from_rng(&mut self.rng);
        let mut combat = CombatState::new(player_fighter, ai_fighter, combat_rng);
        combat.progression_delta = [p_delta, a_delta];
        self.combat = Some(combat);
        self.phase = GamePhase::Combat;
        self.message = "Combat begins! Choose your action.".to_string();
    }

    pub fn combat_action(&mut self, action: CombatAction) {
        {
            let combat = match self.combat.as_ref() {
                Some(c) => c,
                None => return,
            };
            if combat.awaiting_queen_reassign {
                return;
            }
            if !combat.awaiting_action || combat.combat_over {
                return;
            }
            if !combat.action_available(Side::Player, action) {
                return;
            }
        }

        let ai_action = {
            let combat = self.combat.as_ref().unwrap();
            ai::combat_pick(combat, self.ai_personality.as_ref(), &mut self.rng, None)
        };
        let combat = self.combat.as_mut().unwrap();
        combat.resolve_turn(action, ai_action);

        if combat.combat_over {
            self.message = if combat.player_won {
                format!("Fight {}/{} — Victory! [Space] to continue", self.fight, MAX_FIGHTS)
            } else {
                format!("Fight {}/{} — Defeated. [Space] to continue", self.fight, MAX_FIGHTS)
            };
        }
    }

    pub fn queen_cycle_assignment(&mut self, delta: i32) {
        let combat = match self.combat.as_ref() {
            Some(c) if c.awaiting_queen_reassign => c,
            _ => return,
        };
        if combat.queen_original_cards[0].is_none() { return; }
        self.queen_perm_index = ((self.queen_perm_index as i32 + delta).rem_euclid(6)) as usize;
    }

    pub fn queen_confirm_assignment(&mut self) {
        let cards = match self.combat.as_ref() {
            Some(c) if c.awaiting_queen_reassign => {
                match c.queen_original_cards[0] {
                    Some(cards) => cards,
                    None => return,
                }
            }
            _ => return,
        };
        let perm = QUEEN_PERMUTATIONS[self.queen_perm_index];
        let weapon = cards[perm[0]];
        let apparel = cards[perm[1]];
        let item = cards[perm[2]];
        self.combat.as_mut().unwrap().queen_reassign_complete(weapon, apparel, item);
        self.queen_perm_index = 0;
    }

    pub fn advance_from_combat(&mut self) {
        let combat = match self.combat.as_ref() {
            Some(c) if c.combat_over => c,
            _ => return,
        };
        let player_won = combat.player_won;
        let player_hp = combat.player.current_hp;
        let player_max_hp = combat.player.max_hp;

        if let Some(ref mut prog) = self.campaign {
            let hp_margin_pct = if player_won {
                player_hp * 100 / player_max_hp.max(1)
            } else {
                0
            };
            progression::record_fight(prog, player_won, hp_margin_pct);
        }

        if player_won {
            self.fights_won += 1;
            if self.fight >= MAX_FIGHTS {
                self.phase = GamePhase::GameOver { victory: true };
            } else {
                self.fight += 1;
                self.start_fight();
            }
        } else {
            self.phase = GamePhase::GameOver { victory: false };
        }
    }
}
