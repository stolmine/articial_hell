# Articial Hell — Documentation Index

## Design & Planning
- [ideas.md](ideas.md) — Design ideas and future systems: progression/affinity, AI counter-drafting, AI simulated draft scaling, randomness philosophy
- [progress.md](progress.md) — Implementation progress tracker: completed features and backlog

## Source Modules

### Core
- `src/card.rs` — Tarot card types: 78-card deck with Major Arcana (22), Court (16), Numbered (40). Suit/rank enums, display formatting
- `src/deck.rs` — Deck management: shuffling, drawing court/numbered cards from separate pools
- `src/stats.rs` — Stat derivation: hero base stats (Swords/Wands/Cups/Pents), equipment primary/secondary bonuses, rank bonuses (Page/Knight diversity, Queen/King matching), `count_unique_suits` helper
- `src/fate.rs` — Fate card system: `FateEffect` struct with 21 fields, `resolve_fate()` maps all 22 Major Arcana to effects, `fate_description()` for UI display

### Combat
- `src/combat.rs` — Combat engine: `BalanceTweaks` (thorns/bulk/tempo/experimental), `Side` enum, `Fighter` with `reassign_equipment`, `CombatState` with exhaustion cycles + Knight doubled + Queen reassignment + fate tracking (Star barriers, momentum, Magician first-weapon, Judgement second-strike, World stat bonus, Fool item swap, tempo meter), `effective_atk`/`effective_def` for Hanged Man, `apply_heal` with Devil/Empress guards, `deal_damage` interceptor chain, `roll_effect` with Priestess/Wheel/Moon modifiers
- `src/ai.rs` — AI systems: `AiPersonality` (hero-seeded rank/suit weights), draft scoring with optional trace logging, `combat_pick_for(side)` with trace, `queen_reassign` permutation evaluator

### Simulation
- `src/sim.rs` — Headless simulator: AI-vs-AI drafts and fights, `SimConfig` with tweaks/seed/verbose/compare, `BalanceTweaks` A/B comparison across same draft pairs, `QuickStats` analysis (suit/rank winrates, stat deltas, off-vs-def, fight length), full and per-hero reporting

### Game Flow
- `src/game.rs` — Game state machine: title, draft (4-step: hero/weapon/apparel/item), draft reveal, combat, game over. `PlayerState`, Queen reassignment cycling, 10-fight campaign
- `src/main.rs` — Entry point: terminal setup, input handling, render loop, `--sim` CLI with `-n`/`--seed`/`-v`/`--compare` flags

### UI
- `src/ui/mod.rs` — UI router: delegates to phase-specific renderers
- `src/ui/combat_ui.rs` — Combat screen: fighter panes with effective stats, action bar with exhaustion + Knight doubled display, Queen reassignment, combat log
- `src/ui/draft.rs` — Draft screen: card choices, tooltip, stat preview with diffs, picks list, opponent summary
- `src/ui/title.rs` — Title screen
- `src/ui/card_art.rs` — Card visual rendering
- `src/ui/widgets.rs` — Shared UI components: HP bars, stat blocks
- `src/ui/tooltip.rs` — Card tooltip: stats, actions, synergy analysis, Knight Wildcard/Queen Shapeshifter combat mechanic descriptions

### Theme
- `src/theme.rs` — Adaptive theming: dark/light detection (macOS + COLORFGBG), color palettes
