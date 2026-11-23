# Piece Puzzle

Exile game, a race for 1–4 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/piece_puzzle.rs`.

## How to play

**Goal:** fill the target shape exactly with all your pieces, faster than the bots.

- There are three rounds, each a new puzzle: 4 pieces in 2 minutes, then 5 pieces in 3 minutes, then 7 pieces in 4 minutes.
- Tap a piece in the tray to pick it up. Use **⟳ Turn** and **⇋ Flip** to set it the right way.
- Tap the shape where the piece's dotted cell should go. Tap a placed piece to pick it up again.
- Use every piece. Together they must cover the shape exactly, with no overlaps. Any covering that works counts.
- The round ends when you finish, tap **Give up this round**, or time runs out.
- The first four to finish score 200, 140, 90 and 60, plus 1 point for every 2 seconds left. Not finished: 0.
- The highest total after three rounds wins.

## Rules in detail

Fill a target shape exactly with a given set of flat pieces, like a tangram on a square grid.

- **Three rounds**, everyone races on the same puzzle:

  | Round | Target | Pieces | Time |
  |---|---|---|---|
  | 1 | 16 cells | 4 | 2 min |
  | 2 | 24 cells | 5 | 3 min |
  | 3 | 32 cells | 7 | 4 min |

- Pieces are polyominoes of 3–6 cells. They may be **turned and flipped**. Every piece must be used; the
  pieces cover the target exactly, no overlaps. Any covering counts, not only the one the puzzle was cut from.
- The target is a connected shape without holes, cut into pieces by the seeded generator, so it always has
  a solution; the pieces are handed out turned at random.
- **Race.** The round ends when you cover the target, give up, or time runs out. The bots finish at their
  own seeded times (a quarter of the round time up to a little over it; past the time limit means they
  did not finish). When you are done first, the bots' times are played out at once.
- **Points per round** by finishing place among those who finished: **200, 140, 90, 60**, plus 1 point for
  every 2 seconds left at the finish. Not finished: 0. Highest total wins; ties share a place.
- Moves: `{type:"place", piece, cells:[[x,y],…]}` (the cells must be the piece's shape, turned or flipped;
  placing an already placed piece moves it), `{type:"remove", piece}`, `{type:"giveUp"}`.
- View: `round`, `rounds`, `timeLeftMs`, `roundTimeMs`, `racers` (id, score, done this round), `last`
  (each racer's time, place and points in the previous round), `width`, `height`, `target` (row-major),
  `pieces` (shape as handed out, and the cells where it lies, if placed), `over`.
- Seats: seat 0 is the solver; the other seats are racing bots driven by time (`to_move` lists only seat 0).
- Bot (seat 0 in tests): places the pieces where the puzzle was cut from, taking back a piece that is in
  the way. The solver (tests) checks every puzzle by backtracking.
- Screen: pick a piece in the tray, turn ⟳ / flip ⇋ it, tap the board where its marked cell should go;
  tap a placed piece to pick it up again. Keys: 1–7 pick a piece, R turns, F flips, Esc drops it.
