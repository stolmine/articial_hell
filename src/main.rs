mod card;
mod deck;
mod game;
mod ui;

use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::prelude::*;

use game::GameState;

fn main() -> io::Result<()> {
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

    loop {
        terminal.draw(|frame| ui::draw_ui(frame, &game))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('n') => game = GameState::new_game(),
                KeyCode::Char('r') => game.run(),
                KeyCode::Char('1') => game.resolve_card(0),
                KeyCode::Char('2') => game.resolve_card(1),
                KeyCode::Char('3') => game.resolve_card(2),
                KeyCode::Char('4') => game.resolve_card(3),
                _ => {}
            }
        }
    }

    Ok(())
}
