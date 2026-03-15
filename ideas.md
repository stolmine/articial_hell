# Articial Hell — Design Ideas

## Combat: 3-Turn Exhaustion Cycle

The current combat allows spamming the same action every turn, which produces
stalemates (especially with Cups healing). Fix: all 3 actions (weapon, apparel,
item) must each be used exactly once per cycle. After all 3 are exhausted, they
reset and the next cycle begins.

This turns combat from isolated turn-by-turn decisions into a sequencing problem.
You're not just picking what's best *now*, you're committing to an order that
leaves you exposed on specific turns. "Do I Fortify first to survive, knowing
I'll have to attack unprotected next turn?" The opponent is solving the same
puzzle simultaneously.

## Damage Ranges

Flat damage is fully calculable, which kills tension once you've solved the
sequence. Instead: damage = random value between a floor and the card's rank.
A 7 of Swords weapon hits for somewhere in [floor, 7]. Small variance, but
enough that you can't guarantee a kill on turn 3.

Floor TBD — 0 or 1 might be too punishing. Aces are particularly affected since
their range is tiny (0-1 or 1-1). This creates an opportunity:

### Low Card Compensation (future)

Cards ranked 1-3 could gain special effects to offset their narrow damage range.
Examples: Ace weapons crit on first turn of cycle, Ace apparel reflects lethal
damage, low-rank items trigger twice. Makes the draft choice between a reliable
8 and a risky Ace with upside genuinely interesting.

## Arcana as Fate Cards

Currently arcana are drafted like equipment and mostly provide stat buffs. This
is the least interesting part of the game. Rework: arcana are drawn randomly at
the start of each 3-turn cycle as modifiers that change the rules of engagement.

Key constraints:
- **Shared pool** — both players draw from the same 22 Major Arcana deck. No
  duplicates possible. Pool depleting over a long fight becomes its own element.
- **No opponent-targeting effects** — no Fool-style swaps, no Death curses.
  Effects should alter the *rules*, not punish one side arbitrarily.
- **Rule-changing over stat-pumping** — move away from "+5 ATK" toward effects
  that change how you play:

Initiative effects:
- Always act first this cycle
- Always act last, but +damage
- Initiative alternates each turn within the cycle

Action economy effects:
- Use one action twice (player chooses which)
- Skip one action to double another's effect
- Apparel doesn't count toward exhaustion (free defensive turn)

Sequencing effects:
- Lock opponent's first action to mirror yours
- Reveal opponent's chosen sequence for this cycle
- Reverse your exhaustion order

Stat modification effects:
- Damage floor set to 3 (no low rolls)
- Healing capped at 50% effectiveness
- Defense piercing (ignore X defense)
- All damage becomes ranged (even if it wasn't)

Conditional effects:
- Below 50% HP: gain bonus ATK
- Take 0 damage in a turn: bonus heal next turn

Cycle-level effects:
- This cycle lasts 4 turns (one action used twice, player picks)
- This cycle lasts 2 turns (one action skipped, player picks)

22 Major Arcana = 22 distinct effects. The design space is more than sufficient.

## Court Rank Differentiation

Currently Knight and Queen are strictly worse Page and King — same diversity/
matching system but with +2 instead of +3. No reason to draft them. Each rank
needs a unique mechanical identity, not just a different bonus multiplier.

### Page — The Generalist
Diversity bonus: +3 all stats per unique equipment suit (max +9 with 3 unique).
Straightforward synergy ceiling. Best when you can grab 3 different suits.

### Knight — The Wildcard
Diversity bonus: +2 per unique suit (lower ceiling than Page). But: one random
action is **doubled** each cycle — it can be used twice across the same 3-turn
cycle. The doubled action shows as "x2" in the UI. The Knight still plays 3
turns like everyone else, still syncs on the same cycle timing, but has 4
slots to fill 3 turns from.

Example: if weapon is doubled this cycle, the Knight could go Weapon → Weapon →
Apparel, or Apparel → Weapon → Weapon, or Weapon → Item → Weapon. The doubled
action doesn't fully exhaust until used twice. The third (non-doubled,
non-chosen) action is skipped that cycle entirely.

The Knight doesn't choose which action doubles — it's random per cycle. This
creates adaptation pressure: if your Restore gets doubled, it's a healing cycle.
If your weapon doubles, it's an aggression cycle. You plan around what fate
gives you, not what you'd optimally pick.

The opponent knows a Knight has a doubled action but may not know *which* one
until they see the sequencing pattern. Adds a light read/bluff layer.

### Queen — The Shapeshifter
Matching bonus: +2 per equipment matching hero suit (lower ceiling than King).
But: the Queen can **reassign equipment slots** at the start of each cycle.
The three drafted equipment cards can be swapped between weapon/apparel/item
slots freely.

A Queen who drafted Swords 8 as weapon and Cups 6 as apparel can swap them
next cycle — Swords card becomes apparel (Riposte) and Cups card becomes
weapon (Drain). Same cards, different combat identity. One draft produces
multiple viable configurations.

More cognitive overhead than other ranks, but rewards players who think about
card properties across multiple roles. The draft decision becomes about card
*versatility* rather than slot-specific optimization.

### King — The Specialist
Matching bonus: +3 all stats per equipment matching hero suit (max +9 all same).
Highest synergy ceiling when fully committed. Best when you can grab 3 cards
matching your hero suit.

### Draft Decision Framework
The hero pick is no longer just "which suit do I want" — it's "what kind of
combat do I want to play":
- Page: highest synergy ceiling, straightforward diversity goal
- Knight: flexible/chaotic, adapts to random doubles, 4-turn cycles
- Queen: adaptive, multiple build configurations from same draft, strategic depth
- King: highest matched ceiling, straightforward commitment goal

Page and King are the "plan your build" ranks. Knight and Queen are the
"adapt during combat" ranks. Different skills, different appeal.

## Progression: Commitment System

The core problem with stat baking (keeping stats between fights) is that if the
AI scales the same way, it's a treadmill — both sides get bigger numbers but the
relative difference stays flat. Progression needs to be *asymmetric*.

### Suit Affinity

Each suit card picked during drafts accumulates affinity in that suit. Affinity
boosts the secondary stat bonus for that suit's cards in future drafts. Over
several fights, a player who keeps picking Swords equipment develops a Swords
identity where even mediocre Swords cards become competitive.

This creates real draft tension: "this Cups 8 is objectively stronger right now,
but my Swords 4 has +3 from accumulated affinity and feeds my long-term build."
You're not just drafting for this fight, you're investing in a trajectory.

### Rank Path Deepening

Court card rank bonuses scale with commitment across fights:
- Page: diversity bonus grows (+3 → +4 → +5 per unique suit) with repeated
  mixed drafting
- Knight: chance of favorable double increases, or double action granted more
  frequently, with repeated Knight picks
- Queen: slot reassignment becomes more powerful — e.g., stat bonuses when
  swapping, or partial stats from both slot roles
- King: matching bonus grows (+3 → +4 → +5 per matching suit) with repeated
  matched drafting

Rewards commitment without punishing experimentation. Switching rank paths
restarts the path bonus, but doesn't lose suit affinity.

Suit affinity and rank path compound: a committed Swords King gets scaling
bonuses from *both* systems. A diversifying Page gets strong rank bonuses but
spread-thin affinity. A veteran Knight gets more favorable doubles. All viable,
different texture and planning horizon.

### Single-Round Fights, Longer Campaign

Best-of-3 was inherited from Scoundrel and doesn't serve the new design well.
It can feel stale (re-solving the same matchup 3 times), and within-fight delta
baking creates inflation problems that collide with between-fight progression.

**Decision: drop best-of-3. Each fight is a single draft → single combat.**

What this changes:
- No within-fight stat baking. No momentum arc across rounds, because there
  are no rounds. Each fight is one draft, one combat, done.
- Drafting is higher stakes. Every pick matters — no round 2 to course-correct.
  More tension per draft, more weight per decision.
- Campaign gets longer and more varied. Instead of 3 fights × 3 rounds = 9
  drafts, you get 8-10 fights × 1 draft = 8-10 drafts. Each against a
  different AI personality. More varied matchups.
- Affinity accumulates at the same rate (same number of total drafts) but
  across more opponents. "I committed to Swords and it was great against that
  Cups turtle but now I'm facing a Pentacles wall." More interesting than
  re-facing the same opponent.
- Exhaustion cycles + fate cards provide internal fight progression. Each cycle
  changes the rules, so a 2-3 cycle fight (6-9 turns) still evolves without
  stat growth.
- The campaign becomes the narrative arc. You're not coming back within a fight
  (down 0-1, clutch round 3), you're coming back across fights.

### Progression: Affinity and Path Only

With no within-fight baking, all progression lives in the permanent systems:
- Suit affinity: each suit card picked adds affinity. Boosts secondary stats
  for that suit's cards in future drafts. Same card, better outcome over time.
- Rank path: diversity/matching bonuses scale with commitment. Same hero rank,
  stronger payoff for your playstyle over time.

This means progression makes you *better at drafting* (better options, stronger
synergies) rather than *stronger regardless of drafting*. Every fight starts
from a comparable baseline. The draft stays the core experience throughout.

### AI Scaling

Flat +2 per fight is linear. Player affinity compounds multiplicatively through
synergy. A focused player outpaces by fight 3-4. The flat bonus alone is not
enough.

**Preferred approach: adaptive scaling.**

Track how decisively the player wins each fight (HP margin at fight end, number
of turns to win). Scale AI bonus based on performance:
- Crushing opponents → AI bonus jumps (e.g., +4 next fight)
- Barely winning → AI bonus stays flat (+2)
- Losing → AI bonus holds or dips slightly

Self-balancing without the player needing to know the formula. A new player
learning the systems doesn't get steamrolled by escalating stats. A skilled
player doesn't trivialize the campaign after fight 4. Keeps fights competitive
naturally.

**Other approaches considered:**
- Escalating flat bonus (+2, +3, +4, +5...): feels like a timer. Eventually
  overwhelms synergy regardless of skill.
- AI affinity mirroring at discount: AI gets affinity from its own picks but
  at 50% rate. Gap exists but doesn't blow wide open. Could layer on top of
  adaptive scaling later.
- Pre-tuned opponent pool: hand-designed stat profiles per fight. Best-feeling
  curve but most work to set up. Good for bosses.
- Draft advantage as progression: later fights give player 5 choices instead
  of 4, or earlier AI pick visibility. Progression = information, not power.

Future improvements:
- Smarter AI drafting: values suit synergy, reads player's build, counter-drafts
- Mirrored affinity: AI accumulates organically but less efficiently than
  intentional player commitment

### Bosses

Milestone fights with pre-built loadouts or special rules. Details TBD, but they
serve as difficulty checkpoints in an extended campaign and break up the draft →
fight → bake loop with curated challenges.

## AI Draft Personality System

The current AI is a greedy value-maximizer: picks the first Swords/Pentacles
hero it sees, then always takes the highest numbered equipment card with zero
suit or synergy awareness. Combat AI is similarly flat — reactive HP threshold
checks, then default attack. No planning, no identity.

### The Random Hero Root

Each round, the AI draws a random court card as hero. This single card *is* the
personality seed. A King of Cups and a Page of Swords will draft completely
differently if the AI follows through on what those cards imply. Over 3 rounds,
the AI naturally produces variety because it's likely to draw different hero
archetypes each time.

### Rank → Draft Strategy (primary axis)

Rank determines how the AI values suit composition in equipment picks:
- Page: seeks maximum suit diversity. Scores cards higher when they introduce a
  new suit to the equipment loadout.
- Knight: mild diversity preference.
- Queen: mild matching preference.
- King: seeks full suit matching with hero. Scores cards higher when they match
  hero suit.

### Suit → Playstyle Tendency (secondary axis)

Hero suit determines which stats and equipment slots the AI weights more heavily:
- Swords (high ATK) → aggressive / glass cannon. Weights weapon quality higher,
  favors offensive item/apparel suits.
- Pentacles (high DEF) → turtle. Weights apparel quality higher, favors
  defensive options across all slots.
- Cups (high HP) → attrition / tank. Values sustain, tolerates longer fights,
  favors healing options.
- Wands (high SPD) → tempo / first-strike. Values initiative, favors Quick
  Strike weapons and Haste items.

### Combined Personality Examples

These emerge naturally from the two axes without special-casing:
- King of Swords: mono-Swords build, all-in offense, tries to end fights fast
- Page of Cups: mixed-suit drafter, high HP base, durable with diverse bonuses
- Queen of Pentacles: matched Pentacles, fortress build, walls of defense
- Page of Wands: mixed suits, fast, unpredictable action spread
- Knight of Swords: slight diversity lean but offensively weighted
- King of Cups: mono-Cups sustain, extremely hard to kill

### Scoring Function

Each equipment card gets a composite score:

    score = base_value                  // card number (1-10)
          + synergy_bonus               // rank-driven: diversity or matching reward
          + playstyle_weight            // suit-driven: does this card serve my tendency?
          + stat_delta                  // prospective stat improvement from this pick

With a threshold override: if the stat_delta for the objectively best card
exceeds the best personality pick by a meaningful margin (~5 total stats), take
the raw value card. Prevents self-destructive flavor picks while preserving
variety most of the time.

### Personality Weights (derived from hero card)

    hero → PersonalityWeights {
        diversity_weight: f32,    // Page=1.0, Knight=0.3, Queen=-0.3, King=-1.0
        suit_preference: MinorSuit,  // hero suit
        aggression: f32,          // Swords=1.0, Wands=0.5, Cups=-0.5, Pentacles=-1.0
        stat_override_threshold: i32,  // how big a stat gap before ignoring personality
    }

These weights translate personality into concrete draft behavior through the
scoring function. No complex branching — just weighted comparison across the
four dimensions.

### Sub-Personalities

Within the rank/suit framework, specific playstyles can emerge from stat
distribution awareness:
- Min-maxer: dumps everything into one stat (Swords King all-ATK)
- Balancer: spreads across stats (Page with diverse suits)
- Turtle: prioritizes DEF and HP over everything (Pentacles/Cups)
- Glass cannon: maximizes ATK, ignores survivability (Swords)
- Item-focused: weights item slot quality higher (Wands for Haste access)

These don't need explicit implementation — they fall out naturally from the
scoring weights when suit and rank align in certain ways.

### Round-Over-Round Variety

The random hero each round handles this automatically. Different heroes produce
different personality weights, which produce different draft patterns. Even if
the AI draws the same suit twice, different rank means different strategy. And
the equipment pool is different each time since cards don't reshuffle between
rounds.

## AI Combat Heuristics

The current combat AI is a fixed priority list: heal if low HP, finish off if
possible, otherwise always attack. Completely predictable after one fight.

### Scoring Over Branching

Instead of if/else chains, each available action gets a composite score each
turn. Highest score wins. This means the AI's behavior shifts fluidly with
game state rather than jumping between hardcoded modes.

    action_score = personality_weight      // base preference from draft personality
                 + situational_weight      // HP state, opponent threat level
                 + exhaustion_pressure     // must-use bonus on final turn of cycle
                 + opponent_read           // what can/must opponent do this turn?
                 + noise                   // small random term (±1-2 on 0-10 scale)

The noise term is critical — the AI makes the "right" choice most of the time
but occasionally surprises you. Not random enough to feel stupid, not
deterministic enough to be solvable.

### Personality Carrying Into Combat

Draft personality (rank/suit weights) should influence combat sequencing:
- Swords/aggressive: leads with weapon, uses item offensively, apparel is
  afterthought. Sequence: weapon → item → apparel.
- Pentacles/turtle: leads with apparel (fortify), attacks from behind a wall.
  Sequence: apparel → weapon → item.
- Cups/attrition: leads with apparel (restore), then weapon (drain), item as
  cleanup or emergency heal. Sequence: apparel → weapon → item.
- Wands/tempo: leads with weapon (quick strike for initiative), item for burst,
  apparel last. Sequence: weapon → item → apparel.

These are *default* sequences encoded as personality weights, not fixed orders.
Situational scoring can override them — a turtle that's about to die might
lead with an emergency heal instead of fortify.

### Exhaustion Reading (natural difficulty ramp)

The exhaustion cycle creates an information gradient within each cycle:

- Turn 1: most open. 3 actions available for both sides. AI leans on
  personality defaults. Highest uncertainty.
- Turn 2: 2 options each. AI can weight its choice against both opponent
  possibilities. Moderate tactical reasoning.
- Turn 3: both sides forced into their remaining action. AI knows exactly what
  the opponent must do. Pure tactical play.

This means the AI gets "smarter" as each cycle progresses just from the
information structure. Early turns feel personality-driven, late turns feel
tactical. No extra logic needed — it falls out of the exhaustion system.

The AI should explicitly reason about turn 3: if the opponent must use their
weapon next turn, use your defensive action *now*. If the opponent must use
their defensive action, save your weapon for *now* when they can't fight back.

### Counter-Play Heuristics

Simple conditionals that interact with personality weights rather than
overriding them:

- Opponent ATK >> my DEF: prioritize defensive actions early (survive burst).
  An aggressive AI becomes slightly less reckless; a turtle doubles down.
- Opponent has healing, I don't: front-load damage. Race them before sustain
  wins. An aggressive AI gets *more* aggressive; a turtle might still turtle
  and hope to outlast.
- Roughly even stats: follow personality defaults. Trust the build.
- Opponent low HP: weight offensive actions higher regardless of personality.
  Even a turtle goes for the kill.

These produce variety because the same heuristic shifts differently depending
on the personality weights it's layered over.

### Predictability Budget

The AI should feel like it has patterns you can learn but can't fully exploit.
The personality gives it a recognizable style (this opponent is aggressive, that
one turtles). The situational adjustments mean it adapts when pressured. The
noise term means even if you've read the pattern, there's a chance it zigs
when you expected a zag.

Over a campaign with stat baking, the AI could also develop tendencies from its
combat history — if weapon-first worked well last fight, slightly increase its
weapon-first weight. Emergent learning from accumulated experience, not
hand-tuned difficulty curves.

## AI Simulated Draft Scaling

Instead of flat stat bonuses, the AI could simulate the full draft process at
the start of each fight to produce a meaningfully synergized build. At fight
start, the AI runs N simulated drafts (with its personality weights), evaluates
the resulting stat totals and synergy scores, and picks the best build to
actually field.

Why this works as a scaling lever:
- **Early fights (N=1)**: AI drafts once, takes whatever it gets. Same as now.
- **Mid campaign (N=3-5)**: AI picks from a few simulated drafts. Builds become
  noticeably more coherent — a King of Swords reliably achieves mono-suit, a
  Page consistently finds 3 unique suits.
- **Late game (N=10+)**: AI approaches optimal builds for its personality. Still
  constrained by the card pool, but rarely gets stuck with off-synergy picks.

This is fundamentally different from flat stat scaling because the AI gets
*smarter at drafting*, not arbitrarily stronger. A well-synergized AI build is
beatable if you outdraft it; a +20 flat stat bonus just overwhelms regardless.

Could also layer with the personality system: higher N means the AI more
consistently *expresses* its personality archetype, making late-game opponents
feel like refined versions of the archetypes rather than stat-inflated generics.

Pairs with adaptive scaling — performance-based N adjustment means dominant
players face craftier opponents, not just bigger numbers.

## Enemy Combat Pane: Ability Icons

The enemy pane during combat should show icons for each of the AI's three
actions (weapon, apparel, item) with their suit-specific ability names. Icons
should visually indicate which actions are still available vs exhausted this
cycle — e.g., dimmed/crossed-out when used, lit when available.

This gives the player the information they need to read the opponent's
exhaustion state at a glance, which is critical for the turn-3 forced-action
reads that make the exhaustion cycle interesting. Without it, players have to
mentally track what the AI has used, which is bookkeeping rather than strategy.

## DRY Refactor: Combat Code

combat.rs has heavy `if is_player { ... } else { ... }` branching throughout —
every method duplicates field access patterns for player vs AI. This makes every
combat edit touch many lines and creates maintenance risk.

Potential approach: extract a `FighterRef` / `FighterContext` that bundles the
active fighter, opponent, exhaustion state, and label strings ("You"/"Enemy",
"enemy"/"you") into a single struct. Methods operate on the context instead of
branching on `is_player` everywhere. Flags could also be per-fighter rather than
paired booleans.

## Randomness Philosophy

Core tension: player agency vs. uncertainty. Roguelikes live in this space.

Bad randomness: "your attack missed (30% chance)." Overrides player decisions.
Good randomness: visible modifiers that change the stakes of your next choice.
The fate card system is the primary vehicle — both players see the modifier,
both adapt their sequencing. You're navigating uncertainty, not being punished
by coin flips.

Damage ranges add a second, lighter layer. You can plan around expected values
but can't guarantee exact outcomes. Enough variance to maintain tension without
making calculation pointless.
