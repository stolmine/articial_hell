mod card;
mod deck;
mod stats;
mod combat;
mod fate;
mod game;
mod ai;
mod theme;
mod ui;
mod sim;
mod progression;

use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::prelude::*;
use game::{GameState, GamePhase};
use combat::CombatAction;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--sim") {
        let num = args.iter().position(|a| a == "-n")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        let seed = args.iter().position(|a| a == "--seed")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok());
        let compare = args.iter().any(|a| a == "--compare");
        let campaign = args.iter().any(|a| a == "--campaign");
        let num_campaigns = args.iter().position(|a| a == "--campaigns")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        // Logging: -v = all, --log-draft, --log-combat, --log-turns
        let verbose = args.iter().any(|a| a == "-v" || a == "--verbose");
        let log = if verbose {
            sim::LogFlags::all()
        } else {
            sim::LogFlags {
                draft: args.iter().any(|a| a == "--log-draft"),
                combat: args.iter().any(|a| a == "--log-combat"),
                turns: args.iter().any(|a| a == "--log-turns"),
            }
        };

        let config = sim::SimConfig {
            num_fights: num, seed, compare, campaign, num_campaigns, log,
            tweaks: combat::BalanceTweaks::default(),
        };
        sim::balance_report(&config);
        return Ok(());
    }

    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let result = run(&mut terminal);

    terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut game = GameState::new_title();
    let mut last_theme_check = std::time::Instant::now();

    loop {
        if last_theme_check.elapsed() >= std::time::Duration::from_secs(2) {
            game.theme = theme::detect_theme();
            last_theme_check = std::time::Instant::now();
        }
        terminal.draw(|frame| ui::draw_ui(frame, &game))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match &game.phase {
                GamePhase::Title => match key.code {
                    KeyCode::Char('n') => game = GameState::new_game(),
                    KeyCode::Char('q') => break,
                    _ => {}
                },
                GamePhase::Draft { .. } => match key.code {
                    KeyCode::Left => game.move_cursor(-1),
                    KeyCode::Right => game.move_cursor(1),
                    KeyCode::Enter | KeyCode::Char(' ') => game.draft_pick(game.cursor),
                    KeyCode::Char('1') => game.draft_pick(0),
                    KeyCode::Char('2') => game.draft_pick(1),
                    KeyCode::Char('3') => game.draft_pick(2),
                    KeyCode::Char('4') => game.draft_pick(3),
                    KeyCode::Char('q') => break,
                    _ => {}
                },
                GamePhase::DraftReveal { .. } => match key.code {
                    KeyCode::Char(' ') | KeyCode::Enter => game.advance_from_reveal(),
                    KeyCode::Char('q') => break,
                    _ => {}
                },
                GamePhase::Combat => {
                    let queen_mode = game.combat.as_ref()
                        .map(|c| c.awaiting_queen_reassign).unwrap_or(false);
                    if queen_mode {
                        match key.code {
                            KeyCode::Left => game.queen_cycle_assignment(-1),
                            KeyCode::Right => game.queen_cycle_assignment(1),
                            KeyCode::Enter | KeyCode::Char(' ') => game.queen_confirm_assignment(),
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char(' ') | KeyCode::Enter => game.advance_from_combat(),
                            KeyCode::Char('1') => game.combat_action(CombatAction::Weapon),
                            KeyCode::Char('2') => game.combat_action(CombatAction::Apparel),
                            KeyCode::Char('3') => game.combat_action(CombatAction::Item),
                            KeyCode::Char('q') => break,
                            _ => {}
                        }
                    }
                },
                GamePhase::GameOver { .. } => match key.code {
                    KeyCode::Char('n') => game = GameState::new_game(),
                    KeyCode::Char('q') => break,
                    _ => {}
                },
            }
        }
    }

    Ok(())
}
