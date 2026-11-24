# Metal Cast Puzzle

Prison game, solo; our digital take on a cast-metal disentanglement puzzle. Code:
`crates/rules/src/games/metal_cast_puzzle.rs`.

## How to play

**Goal:** slide every ring off the metal loop in as few moves as you can.

- Tap a ring, or its number button, to slide it on or off.
- **Ring 1 can always move.** Any other ring can move only when the ring just below it is **on** and every ring below that one is **off**.
- If a ring is blocked it doesn't move, and this costs you nothing.
- There are three levels: 4, 5 and 6 rings.
- A solved level is worth **150, 250 or 400** points. You get the full amount for the fewest possible moves, and less for each extra move.
- **Reset level** puts the rings back to the start, but your moves still count.
- **Give up** and the level scores 0.

## Rules in detail

The puzzle is a row of rings hanging on a long metal loop (the "linked rings" family of puzzles).
Ring 1 sits at the free end of the loop, the highest ring at the handle.

- **The one rule.** Ring 1 can always slide on or off. Any other ring *k* can slide on or off only when
  ring *k − 1* is **on** and every ring below *k − 1* is **off**. A blocked ring does not move (the move
  is refused, it costs nothing).
- **Goal.** Take every ring off the loop.
- Three levels: **4, 5 and 6 rings**. Level 1 starts with all rings on;
  levels 2 and 3 start from a seeded mix that is at least 12 and 24 moves from the goal.
- The fewest moves for each start is known (the solver walks the whole state space, at most 2⁶ states).
- **Points** per solved level: base **150, 250, 400** × (fewest moves ÷ your moves). Give up: 0 points,
  next level.
- **Reset** puts the level back to its start; the moves made so far still count.
- Moves: `{type:"toggle", ring}` (ring is 1-based), `{type:"reset"}`, `{type:"skip"}`.
- View: `level`, `levels`, `rings` (true = on, index 0 is ring 1), `moves`, `best` (the fewest moves
  from the start), `solved` per level, `score`, `over`.
- Bot: follows the shortest path, with a wrong turn now and then.
- Desktop: keys 1–6 slide that ring, Backspace resets.
- **Replay moments:** level 2 or 3 (the mixed starts) solved in the fewest possible moves ("6 rings off
  in 31 moves, the fewest possible"). Level 1 always starts the same, so it is no moment. Public: solo game.
