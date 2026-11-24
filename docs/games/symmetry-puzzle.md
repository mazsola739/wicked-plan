# Symmetry Puzzle

Exile game, a race for 1–4 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/symmetry_puzzle.rs`.

## How to play

**Goal:** finish each figure so it is a perfect mirror image, before the other racers.

- A coloured figure should be symmetric about the marked axis, but some cells are missing. The given cells are fixed.
- Pick a colour, then tap cells to paint them. Tap a cell in the same colour again (or use the eraser ⌫) to clear it.
- Paint exactly what the mirror needs: the right colour in every cell and no extra cells. There is no check button; the round ends the moment it is right.
- **Three rounds:** a vertical axis (1.5 min), a diagonal axis (2 min), then both a vertical and a horizontal axis (3 min).
- Points by finishing place: **200, 140, 90, 60**, plus 1 point for every 2 seconds left. Not finished: 0.
- Stuck? **Give up this round** and wait for the next one.

## Rules in detail

A coloured figure on a grid should be symmetric about the marked axis (or axes), but parts of it are
missing. Paint the missing cells so the figure becomes symmetric.

- **Three rounds**, everyone races on the same figure:

  | Round | Grid | Axis | Colours | Time |
  |---|---|---|---|---|
  | 1 | 6×6 | vertical | 2 | 1.5 min |
  | 2 | 7×7 | diagonal (top left to bottom right) | 3 | 2 min |
  | 3 | 8×8 | vertical **and** horizontal | 3 | 3 min |

- The given cells are fixed. Every coloured cell of the figure has at least one of its mirror copies given,
  so the figure can be worked out; at least a quarter of the coloured cells are missing. Cells on an axis
  are their own mirror and are always given. On the last board a cell has three mirror copies (across each
  axis and through the centre).
- Paint only what the mirror needs: the round is solved when the grid is exactly the symmetric figure
  (the right colour in every cell, no extra cells). There is no check button: the round ends the moment it is right.
- **Race and points** as in Piece Puzzle: the bots finish at seeded times; points by place among those
  who finished, **200, 140, 90, 60**, plus 1 point per 2 seconds left. Not finished: 0.
- Moves: `{type:"paint", cell, colour}` (cell row-major, colour 1… or 0 to clear), `{type:"giveUp"}`.
- View: the race part (`round`, `rounds`, `timeLeftMs`, `roundTimeMs`, `racers`, `last`, `over`),
  `size`, `axis` (`vertical` | `diagonal` | `both`), `colours`, `cells` (0 = empty),
  `given`.
- Seats: seat 0 is the solver; the other seats are racing bots driven by time.
- Bot (seat 0 in tests): paints the next wrong cell right.
- Screen: pick a colour (or the eraser), tap cells. Tapping a cell in the picked colour clears it.
  Keys: 1–3 pick a colour, 0 the eraser.
- **Replay moments:** the solver finishing first less than 2 s ahead of the next racer ("Photo
  finish: {bot-2} beaten by 1.5 s"), and finishing first in all three rounds ("Clean sweep: first in
  all three rounds"). Public: every finish time is shown in the round results anyway.
