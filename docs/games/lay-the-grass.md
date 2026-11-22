# Lay the Grass

Mini game, 1–4 seats (the page: you + 3 bots). Code: `crates/rules/src/games/lay_the_grass.rs`.

- A square field of **6 + players** squares a side (7×7 solo, 10×10 for four) with **players + 2 rocks**.
- **7 rounds.** Each round one grass piece (2–5 squares) is drawn for everyone; the players place it
  one after the other. The first player moves one seat on each round. The next round's piece is shown.
- A piece may be **turned and flipped** and goes on **any empty squares** (no rocks, no grass). If it fits
  nowhere, that player's turn is skipped. **30 seconds** a turn; when the time runs out the turn is lost.
- Score (always shown): **10 × your largest connected patch** (squares joined by edges) +
  **20 × side² of your largest square of grass** (a 3×3 square gives 180). Results: `rank_by_score`.
- Moves: `{type:"place", cells:[[x,y],…]}`: the squares the turned piece covers (x right, y down).
- View: the field (`-2` rock, `-1` empty, otherwise the seat number), the seats with patch, square and
  score, this and the next piece, whose turn, the turn timer, the last placement.
- Bot: tries every turn and spot next to its own grass (anywhere for its first piece) and keeps the one
  that raises its score most, preferring spots that touch more of its grass.
- Desktop: R turns, F flips, arrows move the piece, Enter places.
