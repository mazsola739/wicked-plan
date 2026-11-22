# Knight's Tour

Hidden game, solo. Code: `crates/rules/src/games/knights_tour.rs`.

- Three boards: **5×5, 6×6, 7×7**, on one **15-minute** clock.
- Put the knight on any square, then jump like a chess knight (an L: two squares one way, one sideways) to a
  square not visited yet. Visit every square once to finish the board.
- Stuck (no free jump)? **Restart the board**; the clock keeps running.
- Points: **200, 300, 500** for the three boards; if all three are done, every second left is worth 1 more point.
- Moves: `{type:"step", x, y}`, `{type:"restart"}`.
- Bot: Warnsdorff's rule (jump to the square with the fewest onward jumps), starting from a corner.
