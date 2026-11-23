# Nine Men's Morris

Final duel, 2 players. Code: `crates/rules/src/games/nine_mens_morris.rs`, screen: `apps/web/src/games/NineMensMorris.tsx`.

## How to play

**Goal:** bring your opponent down to 2 pieces, or leave them with no move.

- You each have **9 pieces**, and you move first.
- **Placing:** on each turn, tap an empty point to put a piece there, until your pieces are all on the board.
- **Moving:** then, on each turn, tap one of your pieces and then a neighbouring empty point along a line.
- **Flying:** when you are down to 3 pieces, a piece can jump to any empty point.
- **Mill:** three of your pieces in a row along a line. Making a mill lets you tap one glowing enemy piece to remove it.
- You have **60 seconds** a turn. If the time runs out, a bot plays your turn for you.
- Points: **100 + 50** for each piece you still have, plus **400** for a win. A draw gives each player **150** extra.

## Rules in detail

The classic mill game on 24 points (three nested squares joined at the middles of their sides).

- Each player has **9 pieces**. The first seat (you on the page) moves first.
- **Placing:** while you have pieces in hand, each turn put one on any empty point.
- **Moving:** with no pieces in hand, each turn slide one piece along a line to a neighbouring empty point.
- **Flying:** with only 3 pieces left, a piece may jump to any empty point.
- **Mill:** three of your pieces in a row along a line. Forming a mill lets you **remove one opponent piece**
  (the same turn) that is not in a mill, unless all of them are in mills.
- **Win:** the opponent is down to 2 pieces, or cannot move on their turn.
- **Draw:** 200 turns in all, or 60 turns in a row without a removal once both hands are empty.
- Turn timer: **60 s**; when it runs out, the bot plays your turn (including a removal).
- Points: **100 + 50 × your pieces** (on the board and in hand) **+ 400 for the win** (+150 each for a draw).
- Moves: `{type:"place", to}`, `{type:"move", from, to}`, `{type:"remove", at}`. Points are numbered 0–23:
  outer square 0–7, middle 8–15, inner 16–23, each clockwise from the top-left corner.
- View: the board (1 yours, −1 theirs, 0 empty), pieces in hand and on the board, whose turn, the phase
  (`place`, `move`, `fly`, `remove`, `wait`), your legal moves and removable pieces, the last move, the timer.
- Bot: a two-ply search (its turn and the reply, removals included) on pieces, open twos (two in a line with
  the third point empty) and mobility.
