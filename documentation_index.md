# Articial Hell — Documentation Index

## Design & Planning
- [ideas.md](ideas.md) — Design ideas and future systems: exhaustion cycles, damage ranges, arcana rework, court rank differentiation, progression/affinity, AI personality/combat heuristics, randomness philosophy
- [progress.md](progress.md) — Implementation progress tracker: completed features and backlog

## Source Modules

### Core
- `src/card.rs` — Tarot card types: 78-card deck with Major Arcana (22), Court (16), Numbered (40). Suit/rank enums, display formatting
- `src/deck.rs` — Deck management: shuffling, drawing court/numbered/arcana cards from separate pools
- `src/stats.rs` — Stat derivation: hero base stats, equipment primary/secondary bonuses, rank bonuses (Page/Knight diversity, Queen/King matching), `count_unique_suits` helper
- `src/arcana.rs` — Major Arcana effects: 22 arcana with stat bonuses and special abilities (always-first, double-first-hit, heal/turn, death curse)

### Combat
- `src/combat.rs` — Combat engine: `Side` enum (Player/Ai) with name/opponent accessors, `Fighter` struct with `reassign_equipment`, `CombatState` with exhaustion cycles + Knight doubled tracking + Queen reassignment flow, `FighterFlags` for per-side turn state, weapon/apparel/item resolution, damage/healing with rolls
- `src/ai.rs` — AI systems: `AiPersonality` (hero-seeded rank/suit weights), draft scoring (synergy + playstyle + stat delta), combat action scoring (personality + situational + opponent read + Knight doubled bonus + noise), `queen_reassign` permutation evaluator, `stat_delta_for_pick` helper

### Game Flow
- `src/game.rs` — Game state machine: title, draft (5-step sequential), combat, game over. `PlayerState::set_slot` for draft picks, Queen reassignment cycling/confirmation, 10-fight campaign
- `src/main.rs` — Entry point: terminal setup, input handling, render loop

### UI
- `src/ui/mod.rs` — UI router: delegates to phase-specific renderers
- `src/ui/combat_ui.rs` — Combat screen: fighter panes, action bar with exhaustion + Knight doubled display, Queen reassignment screen, combat log
- `src/ui/draft.rs` — Draft screen: card choices, tooltip, stat preview, picks list
- `src/ui/title.rs` — Title screen
- `src/ui/card_art.rs` — Card visual rendering
- `src/ui/widgets.rs` — Shared UI components: HP bars, stat blocks
- `src/ui/tooltip.rs` — Card tooltip: stats, actions, synergy analysis

### Theme
- `src/theme.rs` — Adaptive theming: dark/light detection (macOS + COLORFGBG), color palettes
