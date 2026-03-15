# Articial Hell — Progress

## Implemented

### Core Game (2026-03-14)
- [x] Tarot deck: 78 cards — 16 court, 40 numbered (Ace-10), 22 Major Arcana
- [x] 4 suits: Swords (ATK), Wands (SPD), Cups (HP), Pentacles (DEF)
- [x] 5-step sequential draft: hero → weapon → apparel → item → arcana
- [x] Each player has own deck, fresh shuffle per fight
- [x] Stat derivation: hero base + equipment primary/secondary + rank bonuses
- [x] Court rank system: Page/Knight diversity, Queen/King matching (basic bonuses only)
- [x] 22 Major Arcana with effects (stat bonuses, always-first, double-first-hit, heal/turn, death curse)
- [x] Combat: simultaneous action selection, 4 weapon types, 4 apparel types, 4 item types
- [x] 3-turn exhaustion cycle: all 3 actions must be used before reset
- [x] Items reusable per cycle (no longer one-shot)
- [x] Haste (Wands item) blows entire cycle in one turn
- [x] Single-round campaign: 10 fights, loss ends run, fresh decks each fight
- [x] AI draft: greedy (highest value), prefers Swords/Pentacles heroes
- [x] AI combat: picks from available actions, reads opponent exhaustion state

### Balance (2026-03-14)
- [x] Diminishing healing returns (all sources: Drain, Restore, Elixir, Temperance)
- [x] Damage floor of 2 (prevents zero-damage stalemates)
- [x] Restore reworked: heals 60% of damage taken last turn (breaks DEF→heal loop)

### TUI (2026-03-14)
- [x] Ratatui-based terminal UI
- [x] Arrow key navigation with focused card highlight (double border)
- [x] Card tooltip: stats, actions, synergy analysis with current hero/rank
- [x] Live stat diff preview in Build pane (current → prospective with +/- coloring)
- [x] Running picks list with slot status (filled / picking / empty)
- [x] Combat screen stays after fight end, [Space] advances to next fight or game over
- [x] Adaptive theming: live dark/light detection (macOS + COLORFGBG), re-checks every 2s
- [x] Exhausted actions shown with strikethrough, cycle counter in title
- [x] Combat log adaptive height (fills available space, not fixed 5 lines)
- [x] Campaign progress in UI: "Fight X/10", game over shows wins/total

## To Implement

### Combat
- [ ] Damage ranges: random between floor and card rank
- [ ] Arcana as fate cards: drawn per cycle from shared pool, rule-changing effects

### Code Quality (2026-03-15)
- [x] DRY refactor: `Side` enum replaces `is_player: bool` throughout combat.rs
- [x] `FighterFlags` struct replaces 8 paired booleans in `TurnFlags`
- [x] `fighter(side)` / `fighter_mut(side)` accessors centralize player/ai branching
- [x] `count_unique_suits` helper shared by stats.rs and arcana.rs
- [x] `rank_bonus` takes `&[TarotCard]` slice; `partial_derive` calls it instead of reimplementing
- [x] `PlayerState::set_slot` eliminates duplicated 5-arm match in draft
- [x] `stat_delta_for_pick` / `slot_with_pick` collapse 3 identical tuple-construction blocks in AI

### AI (2026-03-14)
- [x] Personality system: random hero as seed, rank → draft strategy, suit → playstyle
- [x] Scoring function for draft: base_value + synergy + playstyle + stat_delta
- [x] Combat scoring: personality_weight + situational + opponent_read + noise
- [x] Exhaustion reading: tactical play on forced turns (opponent-read scoring)

### Progression
- [ ] Suit affinity: accumulates per pick, boosts secondary stats in future drafts
- [ ] Rank path deepening: diversity/matching bonuses scale with commitment
- [ ] Adaptive AI scaling: based on player win margin (HP remaining, turns to win)

### Court Rank Differentiation
- [ ] Knight: random action doubled per cycle (x2), still 3 turns, one action skipped
- [ ] Queen: slot reassignment — swap equipment between weapon/apparel/item at cycle start
- [ ] Page/King keep current synergy bonuses, Knight/Queen get unique mechanics instead

### Balance
- [ ] Low card compensation (Ace-3 special effects)

### UI Wishlist
- [ ] Average stat diff shown in tooltip when focusing equipment during draft
- [ ] Combat log scrollback (arrow keys to scroll full history)
- [x] Replace emoji with nerdfont/unicode icons for suit Display

### Future
- [ ] Bosses: pre-built loadouts or special rules as campaign milestones
- [ ] Smarter AI: suit synergy awareness, counter-drafting
- [ ] Draft advantage as progression: 5 choices instead of 4 in later fights
