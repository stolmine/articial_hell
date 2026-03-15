# Articial Hell — Progress

## Implemented

### Core Game (2026-03-14)
- [x] Tarot deck: 78 cards — 16 court, 40 numbered (Ace-10), 22 Major Arcana
- [x] 4 suits: Swords (ATK), Wands (SPD), Cups (HP), Pentacles (DEF)
- [x] 4-step sequential draft: hero → weapon → apparel → item
- [x] Each player has own deck, fresh shuffle per fight
- [x] Stat derivation: hero base + equipment primary/secondary + rank bonuses
- [x] Combat: simultaneous action selection, 4 weapon types, 4 apparel types, 4 item types
- [x] 3-turn exhaustion cycle: all 3 actions must be used before reset
- [x] Items reusable per cycle (no longer one-shot)
- [x] Haste (Wands item) fires bonus weapon + apparel without exhausting them
- [x] Single-round campaign: 10 fights, loss ends run, fresh decks each fight

### Court Rank Differentiation (2026-03-15)
- [x] Page: diversity bonus +2 all stats per unique equipment suit (max +6)
- [x] Knight: diversity bonus +2 per unique suit + Wildcard (random action doubled per cycle, x2 uses, one action skipped)
- [x] Queen: matching bonus +3 per equipment matching hero suit (max +9) + Shapeshifter (reassign equipment slots at cycle start)
- [x] King: matching bonus +3 per equipment matching hero suit (max +9)
- [x] Knight/Queen tooltips on draft screen showing combat mechanics

### Fate Cards (2026-03-15)
- [x] Shared pool of 22 Major Arcana, drawn per cycle
- [x] All 22 effects implemented:
  - 0 Fool: swap items between fighters
  - I Magician: first weapon strike each cycle deals double
  - II High Priestess: all rolls use minimum values
  - III Empress: all healing doubled
  - IV Emperor: reverse initiative (slow strikes first)
  - V Hierophant: forced action order (Weapon → Apparel → Item)
  - VI Lovers: 50% damage reflects to attacker
  - VII Chariot: +3 per consecutive weapon action (momentum)
  - VIII Strength: minimum damage raised to 5
  - IX Hermit: all defenses doubled
  - X Wheel of Fortune: all rolls hit maximum
  - XI Justice: all defenses halved
  - XII Hanged Man: ATK and DEF swapped
  - XIII Death: fighters below 25% HP take double damage
  - XIV Temperance: both fighters regenerate 3 HP/turn
  - XV Devil: all healing disabled
  - XVI Tower: all attacks deal 150% damage
  - XVII Star: first hit against each fighter absorbed per cycle
  - XVIII Moon: damage rolls swing 0 to double
  - XIX Sun: all attacks deal +4 damage
  - XX Judgement: second actor deals 150% damage
  - XXI World: both fighters gain +3 all stats

### Balance Mechanics (2026-03-15)
- [x] Passive thorns: when hit, reflect 12% of DEF as damage to attacker
- [x] HP bulk bonus: +1 damage per 6 max HP above 20
- [x] Tempo system: accumulate (own_spd - opp_spd) per turn, bonus action at threshold 10
- [x] Diminishing healing: 15% decay per heal (all sources: Drain, Restore, Elixir, Temperance)
- [x] Damage floor of 2 (prevents zero-damage stalemates)
- [x] Restore: heals 60% of damage taken last turn
- [x] Drain ATK ratio: 70% (up from 60%)

### Base Stats (2026-03-15)
- [x] Swords: ATK 7, SPD 5, HP 14, DEF 3
- [x] Wands: ATK 5, SPD 8, HP 14, DEF 4
- [x] Cups: ATK 4, SPD 4, HP 24, DEF 2
- [x] Pentacles: ATK 3, SPD 3, HP 14, DEF 12

### Simulator (2026-03-15)
- [x] Headless AI-vs-AI draft and combat (`--sim` flag)
- [x] Balance reporting: suit/rank/matchup winrates, stat profiles, fight length
- [x] A/B comparison mode (`--compare`) with configurable BalanceTweaks
- [x] Granular AI decision tracing (`-v`): draft scoring + combat scoring per turn
- [x] Deterministic seeding (`--seed N`) for reproducible results

### AI (2026-03-14, updated 2026-03-15)
- [x] Personality system: random hero as seed, rank → draft strategy, suit → playstyle
- [x] Scoring function for draft: base_value + synergy + playstyle + stat_delta
- [x] Combat scoring: personality_weight + situational + opponent_read + noise
- [x] Exhaustion reading: tactical play on forced turns (opponent-read scoring)
- [x] Knight doubled action bonus in combat scoring
- [x] `combat_pick_for(side)`: generalized for both sides (sim uses for P1)
- [x] Optional trace logging for all draft and combat decisions

### Code Quality (2026-03-15)
- [x] DRY refactor: `Side` enum replaces `is_player: bool` throughout combat.rs
- [x] `FighterFlags` struct replaces 8 paired booleans in `TurnFlags`
- [x] `fighter(side)` / `fighter_mut(side)` accessors centralize player/ai branching
- [x] `count_unique_suits` helper shared by stats.rs
- [x] `rank_bonus` takes `&[TarotCard]` slice; `partial_derive` calls it instead of reimplementing
- [x] `PlayerState::set_slot` eliminates duplicated 5-arm match in draft
- [x] `stat_delta_for_pick` / `slot_with_pick` collapse 3 identical tuple-construction blocks in AI
- [x] `effective_atk`/`effective_def` helpers for Hanged Man swap
- [x] `apply_heal` helper centralizes Devil/Empress healing guards
- [x] `deal_damage(attacker, target, dmg)` with interceptor chain (Star/Death/Judgement/Lovers/Thorns/Bulk)

### TUI (2026-03-14, updated 2026-03-15)
- [x] Ratatui-based terminal UI
- [x] Arrow key navigation with focused card highlight (double border)
- [x] Card tooltip: stats, actions, synergy analysis with current hero/rank
- [x] Knight/Queen combat mechanic tooltips (Wildcard, Shapeshifter)
- [x] Live stat diff preview in Build pane (current → prospective with +/- coloring)
- [x] Combat stats show effective values (Hanged Man ATK↔DEF swap visible)
- [x] Running picks list with slot status (filled / picking / empty)
- [x] Combat screen stays after fight end, [Space] advances to next fight or game over
- [x] Adaptive theming: live dark/light detection (macOS + COLORFGBG), re-checks every 2s
- [x] Exhausted actions shown with strikethrough, cycle counter in title
- [x] Combat log adaptive height (fills available space, not fixed 5 lines)
- [x] Campaign progress in UI: "Fight X/10", game over shows wins/total
- [x] Damage readout shows fate modifiers (Sun bonus, Tower multiplier, Chariot momentum, Magician double)
- [x] Replace emoji with nerdfont/unicode icons for suit Display

## To Implement

### Progression
- [ ] Suit affinity: accumulates per pick, boosts secondary stats in future drafts
- [ ] Rank path deepening: diversity/matching bonuses scale with commitment
- [ ] Adaptive AI scaling: based on player win margin (HP remaining, turns to win)

### AI
- [ ] Counter-draft scoring: exploit player's visible picks (weakness targeting, archetype countering)
- [ ] Simulated draft scaling: AI runs N drafts, picks best (N increases with difficulty)

### Balance
- [ ] Low card compensation (Ace-3 special effects)
- [ ] Wands King/Queen underperformance (~41% WR, remaining outlier)

### UI Wishlist
- [ ] Average stat diff shown in tooltip when focusing equipment during draft
- [ ] Combat log scrollback (arrow keys to scroll full history)

### Future
- [ ] Bosses: pre-built loadouts or special rules as campaign milestones
- [ ] Draft advantage as progression: 5 choices instead of 4 in later fights
