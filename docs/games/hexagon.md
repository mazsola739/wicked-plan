# Hexagon

Final duel, played solo here. Code: `crates/rules/src/games/hexagon.rs`.

## How to play

**Goal:** remember the board, then find lines that add up to the target in 5 rounds.

- 19 cells in a hexagon hold numbers from 1 to 9. You see them for 30 seconds; tap **Ready** to start sooner.
- Then the numbers are turned over and the cells show the letters A–S.
- Each round shows a target; there is no time limit.
- Tap three neighbouring cells in a straight line, in any of the three directions, whose numbers add up to the target. The order does not matter.
- Tap **Submit**. A right answer scores 100.

## Rules in detail

- 19 cells in a hexagon (rows of 3, 4, 5, 4, 3) hold numbers 1–9, lettered A–S.
- **Study:** the numbers are shown for **30 seconds** (or until you tap Ready), then turned over.
- **5 rounds**, no time limit: a target is shown; pick **three neighbouring cells in a straight line**
  (any of the three directions) whose numbers add up to the target. The order of picking does not matter.
- Targets differ from round to round where possible; each one has at least one line.
- Scoring: **100** for a right answer.
- Moves: `{type:"ready"}`, `{type:"answer", cells:[a,b,c]}` (indices 0–18).
- Bot: remembers the board 60% of the time.
- Desktop: letters A–S pick cells, Backspace undoes, Enter submits (or ends the study time).
- **Replay moments:** every round right ("All five lines from memory"). Public: a solo game.
