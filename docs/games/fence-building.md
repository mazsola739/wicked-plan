# Fence Building

Exile game, a race for 1–4 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/fence_building.rs`.

Build one closed fence along the grid lines of a field. A number in a plot tells how many of the plot's
four sides carry fence.

- **The fence** runs between the posts (the dots), one segment per side of a plot. It must be a single
  closed loop: it never branches, never crosses or touches itself, and has no loose ends.
- **Numbers**: a plot with a number has exactly that many fenced sides. Plots without a number may have
  any count.
- **Three rounds**, everyone races on the same field:

  | Round | Field | Numbers shown | Time |
  |---|---|---|---|
  | 1 | 5×5 | 45% of the plots, plus what a single fence needs | 2 min |
  | 2 | 6×6 | 38%, plus … | 3 min |
  | 3 | 7×7 | 34%, plus … | 5 min |

- The generator grows a random fenced-in area (connected, no holes, never touching itself at a corner),
  numbers every plot and hides numbers at random down to the share above. Then the solver looks for two
  fences; while it finds two, a hidden number where they differ is shown again. So every field has
  **exactly one** fence. Proving that takes a while, so the game draws from a bank of 48 generated fields
  per round (in the code, made by the `print_bank` test), turned or mirrored at random (8 ways).
- The round is solved the moment the fence is right; there is no check button. Crosses (×) are only notes
  for yourself.
- **Race and points** as in Piece Puzzle: the bots finish at seeded times; points by place among those
  who finished, **200, 140, 90, 60**, plus 1 point per 2 seconds left. Not finished: 0.
- Edges: the horizontal ones first, row by row (`(n+1)·n`), then the vertical ones row by row (`n·(n+1)`).
- Moves: `{type:"edge", edge, mark}` (mark 0 empty, 1 fence, 2 cross), `{type:"clear"}`, `{type:"giveUp"}`.
- View: the race part (`round`, `rounds`, `timeLeftMs`, `roundTimeMs`, `racers`, `last`, `over`),
  `size`, `clues` (row-major, null where hidden), `edges` (marks).
- Seats: seat 0 is the solver; the other seats are racing bots driven by time.
- Bot (seat 0 in tests): fences the solution edge by edge, with a slip now and then.
- Screen: tap near a side of a plot: empty → fence → cross → empty. A number above 0 turns green when its
  count is met and red when it is over.
