# Tiger and Goat

Final duel, 2 players. Code: `crates/rules/src/games/tiger_and_goat.rs`, screen: `apps/web/src/games/TigerAndGoat.tsx`.

The classic hunt game, played on **two boards at once**: on board A you are the **tigers** and the opponent
the goats; on board B the sides are swapped. Both players play the same boards, so the results compare
directly.

## One board

- 5×5 points. Every point joins its orthogonal neighbours; points where x + y is even also join their
  diagonal neighbours.
- **4 tigers** start on the corners. **20 goats** wait in hand.
- **Goats** move first. While goats are in hand, each goat turn places one on an empty point; after all 20
  are placed, a goat turn slides one goat along a line to a neighbouring empty point.
- **Tigers** slide one tiger to a neighbouring empty point, or **jump** over a neighbouring goat along a
  line to the empty point right behind it, capturing the goat. One jump per turn.
- **Tigers win** a board by capturing **5 goats**. **Goats win** a board when the tigers cannot move.
- A board also ends after **100 turns** (50 each) without a winner.

## Two boards together

- Play goes in **steps**. In even steps the goats move on both boards, in odd steps the tigers: so in every
  step each player makes exactly one move (the opponent on the other board at the same time). A finished
  board is skipped.
- Step timer: **60 s**; when it runs out, the bot moves for whoever has not moved.
- The game ends when both boards have ended.

## Points

Base **100**, plus for each board:

- as tigers: **50 per goat captured** (at most 5), **+250** for a tiger win;
- as goats: **50 per goat short of 5 captured** (5 − captured), **+250** for a goat win.

Range about 100–1100; more points wins.

## Moves and view

- Moves: `{type:"place", board, to}`, `{type:"move", board, from, to}` (a jump is a move two points away).
  Points are `y × 5 + x`, boards 0 (A) and 1 (B).
- View: for each board its points (0 empty, 1 tiger, 2 goat), your side, goats in hand, goats captured,
  whose turn, whether it has ended and who won, your legal moves there, and the last move; your points so
  far, the step and the timer.
- Bot: one ply with lookahead built into the evaluation. Tigers take a capture when they can, otherwise
  maximise open jumps and mobility; goats avoid leaving any goat jumpable (the tigers move next) and
  squeeze the tigers' mobility.
