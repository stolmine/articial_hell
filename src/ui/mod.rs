mod title;
mod draft;
mod combat_ui;
mod card_art;
mod widgets;
mod tooltip;

use ratatui::prelude::*;
use crate::game::{GamePhase, GameState};

pub fn draw_ui(frame: &mut Frame, game: &GameState) {
    match &game.phase {
        GamePhase::Title => title::render_title(frame, &game.theme),
        GamePhase::Draft { .. } => draft::render_draft(frame, game),
        GamePhase::DraftReveal { .. } => draft::render_draft_reveal(frame, game),
        GamePhase::Combat => combat_ui::render_combat(frame, game),
        GamePhase::GameOver { .. } => title::render_game_over(frame, game),
    }
}
