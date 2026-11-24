# Bagh Chal

Final duel, 2 players. Code: `crates/rules/src/games/tiger_and_goat.rs`, screen: `apps/web/src/games/TigerAndGoat.tsx`.

## How to play

**Goal:** score more points than your opponent over two boards: you are the tigers on board A and the goats on board B.

- Each board has 4 tigers in the corners and 20 goats waiting in hand. Goats move first.
- Goats: tap an empty point to place a goat. Once all 20 are placed, slide a goat one step along a line.
- Tigers: slide one step, or jump over a goat to the empty point right behind it to capture it.
- Tigers win a board with **5 captures**; goats win by leaving the tigers no move.
- Every step you make one move. Switch boards with the board buttons, tap a piece, then where it goes.
- Points: as tigers 50 per capture and +250 for a win; as goats 50 per capture short of 5 and +250 for a win.

**On the show:** In the final, both finalists played two boards at the same time, as the tigers on one and the goats on the other. Player A won the game by being the first to win one of the boards (here both boards are played to the end and the points decide).

## Rules in detail

The classic hunt game, played on **two boards at once**: on board A you are the **tigers** and the opponent
the goats; on board B the sides are swapped. Both players play the same boards, so the results compare
directly.

- **Replay moments:** the goats trapping every tiger ("Every tiger trapped: board B won"), the goats
  lasting all 100 turns ("Board A held for all 100 turns"), and the tigers taking the fifth goat after
  all 20 were placed ("Board A won: fifth goat taken after all 20 placed"). Public: both boards are
  open to both players.

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
- No timer: a step ends once every board still in play has had its move.
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
  far and the step.
- Bot: one ply with lookahead built into the evaluation. Tigers take a capture when they can, otherwise
  maximise open jumps and mobility; goats avoid leaving any goat jumpable (the tigers move next) and
  squeeze the tigers' mobility.
