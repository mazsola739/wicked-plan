# Hexagon

Final duel, played solo here. Code: `crates/rules/src/games/hexagon.rs`.

- 19 cells in a hexagon (rows of 3, 4, 5, 4, 3) hold numbers 1–9, lettered A–S.
- **Study:** the numbers are shown for **30 seconds** (or until you tap Ready), then turned over.
- **5 rounds**, **25 seconds** each: a target is shown; pick **three neighbouring cells in a straight line**
  (any of the three directions) whose numbers add up to the target. The order of picking does not matter.
- Targets differ from round to round where possible; each one has at least one line.
- Scoring: **100 + up to 100 for speed** for a right answer.
- Moves: `{type:"ready"}`, `{type:"answer", cells:[a,b,c]}` (indices 0–18).
- Bot: remembers the board 60% of the time.
- Desktop: letters A–S pick cells, Backspace undoes, Enter submits (or ends the study time).
