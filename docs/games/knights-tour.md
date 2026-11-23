# Knight's Tour

Hidden game, solo. Code: `crates/rules/src/games/knights_tour.rs`.

## How to play

**Goal:** jump the knight onto every square of the board exactly once.

- There are three boards, 5×5, 6×6 and 7×7, and one **15-minute** clock for all of them.
- Tap any square to put the knight down.
- Then tap a square one knight jump away: two squares one way and one square sideways, like an L.
- You can't land on a square you have already visited.
- Visit every square to finish the board and move on to the next one.
- Stuck with no free jump? Tap **Restart this board**. The clock keeps running.
- A board is worth **200, 300 and 500** points. Finish all three and every second left adds 1 more point.
- Tip: squares near the edges and corners are the hardest to reach, so visit them early.

## Rules in detail

- Three boards: **5×5, 6×6, 7×7**, on one **15-minute** clock.
- Put the knight on any square, then jump like a chess knight (an L: two squares one way, one sideways) to a
  square not visited yet. Visit every square once to finish the board.
- Stuck (no free jump)? **Restart the board**; the clock keeps running.
- Points: **200, 300, 500** for the three boards; if all three are done, every second left is worth 1 more point.
- Moves: `{type:"step", x, y}`, `{type:"restart"}`.
- Bot: Warnsdorff's rule (jump to the square with the fewest onward jumps), starting from a corner.
