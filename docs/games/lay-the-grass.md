# Lay the Grass

Mini game, 1–4 seats (the page: you + 3 bots). Code: `crates/rules/src/games/lay_the_grass.rs`.

## How to play

**Goal:** grow the biggest connected patch of grass in your colour, and the biggest solid square.

- The game lasts **7 rounds**. Each round everyone gets the same grass piece and lays it in turn.
- On your turn, use **Turn** and **Flip** to change the piece's shape, tap where it goes, then tap again or press **Lay**.
- A piece covers only empty squares. Rocks and grass already on the field block it.
- You also see the next round's piece, so you can plan ahead.
- Score: **10 points** for each square of your largest connected patch, plus **20 × side²** of your largest solid square of grass. A 3×3 square gives 180.
- Tip: keep your grass together. A bigger square pays off fast.

## Rules in detail

- A square field of **6 + players** squares a side (7×7 solo, 10×10 for four) with **players + 2 rocks**.
- **7 rounds.** Each round one grass piece (2–5 squares) is drawn for everyone; the players place it
  one after the other. The first player moves one seat on each round. The next round's piece is shown.
- A piece may be **turned and flipped** and goes on **any empty squares** (no rocks, no grass). If it fits
  nowhere, that player's turn is skipped.
- Score (always shown): **10 × your largest connected patch** (squares joined by edges) +
  **20 × side² of your largest square of grass** (a 3×3 square gives 180). Results: `rank_by_score`.
- Moves: `{type:"place", cells:[[x,y],…]}`: the squares the turned piece covers (x right, y down).
- View: the field (`-2` rock, `-1` empty, otherwise the seat number), the seats with patch, square and
  score, this and the next piece, whose turn, the last placement.
- Bot: tries every turn and spot next to its own grass (anywhere for its first piece) and keeps the one
  that raises its score most, preferring spots that touch more of its grass.
- Desktop: R turns, F flips, arrows move the piece, Enter places.
- **Replay moments:** the first 3×3 square of grass anyone lays ("First 3×3 square of grass on the
  field"), and any 4×4 square ("A 4×4 square of grass: 320 bonus"). Public: the field is open to all.
