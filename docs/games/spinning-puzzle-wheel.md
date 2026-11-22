# Spinning Puzzle Wheel

Team mini game, 1–4 players (on the page: you + 3 bot teammates). Code:
`crates/rules/src/games/spinning_puzzle_wheel.rs`.

- **The wheel** holds **6 shape puzzles**. Each puzzle is a silhouette on a 5 × 5 grid with its own tray of
  **4 pieces** (polyominoes of 3–5 squares): three of them fill the silhouette exactly, one is a decoy.
  (Another way to fill it counts too.)
- **Turns:** the wheel points at one unfinished puzzle and one player, in seat order. That player has
  **12 seconds** to do one thing on that puzzle: **place** a tray piece (turned or flipped any way, only on
  empty silhouette squares), **take back** a piece placed there before, or **pass**. Time out = pass.
  Then the wheel turns to the next unfinished puzzle and the next player.
  (The catalogue's 5 seconds became 12 so that turning a piece by touch is possible.)
- A puzzle is **done** when every square of its silhouette is covered.
- **End:** when all six are done, or after **36 turns**.
- **Score (team):** **100 per finished puzzle**; if all six are done, **+10 per turn left**. Every player gets
  the team's points and shares rank 1.
- **View:** everything is public (all puzzles, trays, placed pieces, whose turn it is).
- **Moves:** `{type:"place", piece, cells:[[x,y]…]}` (squares of the 5 × 5 grid), `{type:"remove", piece}`,
  `{type:"pass"}`; `piece` is the index in the current puzzle's tray.
- **Bot:** solves the puzzle in front of it (search over the tray) and places one piece of a solution; if
  the pieces already placed cannot be finished, it takes one back. Now and then (10%) it runs out of time.
- **Screen:** the wheel with the six puzzles in SVG, the current puzzle large; pick a tray piece, turn ↻ /
  flip ⇋ it, tap the grid to put the ghost there, then Place. Desktop keys: 1–4 pick, R turns, F flips,
  arrows move, Enter places, P passes.
