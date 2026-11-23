# Equation Target

Prison game, played solo here. Code: `crates/rules/src/games/equation_target.rs`.

## How to play

**Goal:** make the target number from three cards, as fast as you can, in 5 rounds.

- Each round shows ten number cards (1–9) and a target. You have 40 seconds.
- Tap three different cards in order (keys 1–9 and 0 on desktop).
- Tap an operator between them to switch it between +, − and ×.
- The sum is worked out left to right: the first two cards first, then the third. × does not go first here.
- Tap **Submit**. A right answer scores 100, plus up to 100 more for speed.
- Targets grow each round, from 5–20 in round 1 up to 30–99 in round 5. Every target can be reached.

## Rules in detail

- **5 rounds**, **40 seconds** each. Ten number cards (1–9) and a target are shown.
- Pick **three different cards** in order and **two operators** (+ − ×). The expression is worked out
  **left to right**: `(a ∘ b) ∘ c`. It must equal the target.
- Targets grow by round: 5–20, 10–30, 15–45, 20–70, 30–99. Every target has at least one solution.
- Scoring: **100 + up to 100 for speed** for a right answer.
- Move: `{type:"answer", value:{picks:[i,j,k], ops:["+","*"]}}` (card indices 0–9).
- Bot: right 70% of the time, otherwise the right cards with random operators.
- Desktop: keys 1–9 and 0 pick the cards in order, + − * set the operators, Backspace undoes, Enter submits.
- **Replay moments:** the last (biggest) target hit within 10 seconds ("Target 84 hit with 37 s to
  spare"), and a perfect run with every round right ("All five targets hit"). Public: a solo game.
