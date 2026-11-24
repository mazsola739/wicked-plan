# Cooperative Puzzle

Team mini game, 1–4 players (on the page: you + 3 bot teammates). Code:
`crates/rules/src/games/spinning_puzzle_wheel.rs`.

## How to play

**Goal:** fill all six shape puzzles on the wheel together with your team, as fast as you can.

- Each puzzle is a shape on a small grid with a tray of 4 pieces. Three of them fill the shape; one is a decoy.
- The wheel turns: each turn it points at one unfinished puzzle and one player.
- On your turn you have **12 seconds** for one action on that puzzle: place a piece, take one back, or **Pass**.
- To place: tap a tray piece, use **↻ Turn** and **⇋ Flip**, tap the grid to move it, then tap **Place**.
- To take a piece back, tap a placed piece in the tray, then **Take back**. If time runs out, you pass.
- The team scores **100 per finished puzzle**. Finish all six and you get **+10 for every turn left** (36 turns in all).

## Rules in detail

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
- **Replay moments:** the piece that finishes all six puzzles with 15 or more turns to spare, by the
  player who placed it ("All six shapes done, 17 turns to spare"; in the fewest turns possible, 18:
  "Perfect wheel: all six in 18 turns"). Public: everything in this game is.
