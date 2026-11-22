# Metal Cast Puzzle

Prison game, solo; our digital take on a cast-metal disentanglement puzzle. Code:
`crates/rules/src/games/metal_cast_puzzle.rs`.

The puzzle is a row of rings hanging on a long metal loop (the "linked rings" family of puzzles).
Ring 1 sits at the free end of the loop, the highest ring at the handle.

- **The one rule.** Ring 1 can always slide on or off. Any other ring *k* can slide on or off only when
  ring *k − 1* is **on** and every ring below *k − 1* is **off**. A blocked ring does not move (the move
  is refused, it costs nothing).
- **Goal.** Take every ring off the loop.
- Three levels: **4, 5 and 6 rings**, with **3, 4 and 6 minutes**. Level 1 starts with all rings on;
  levels 2 and 3 start from a seeded mix that is at least 12 and 24 moves from the goal.
- The fewest moves for each start is known (the solver walks the whole state space, at most 2⁶ states).
- **Points** per solved level: base **150, 250, 400** × (fewest moves ÷ your moves), plus 1 point for
  every 2 seconds left. Run out of time or give up: 0 points, next level.
- **Reset** puts the level back to its start; the moves made so far still count.
- Moves: `{type:"toggle", ring}` (ring is 1-based), `{type:"reset"}`, `{type:"skip"}`.
- View: `level`, `levels`, `rings` (true = on, index 0 is ring 1), `moves`, `best` (the fewest moves
  from the start), `solved` per level, `score`, `timeLeftMs`, `levelTimeMs`, `over`.
- Bot: follows the shortest path, with a wrong turn now and then.
- Desktop: keys 1–6 slide that ring, Backspace resets.
