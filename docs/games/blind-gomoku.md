# Blind Gomoku

Hidden game, solo against the house bot (1 seat). Code: `crates/rules/src/games/blind_gomoku.rs`.

## How to play

**Goal:** get five or more of your dark tiles in a row before the bot does.

- The board is 9×9. You play dark and move first; the bot plays light.
- On your turn, tap an empty square, then tap it again (or tap **Place**) to put your tile there.
- Rows count across, down or diagonally, and are checked after every move.
- **The catch:** only the newest tile shows its colour. Every older tile is face down, so you must remember which squares are yours.
- Tap **Peek** to see every tile face up for 3 seconds. You have two peeks per game.
- Winning fast scores more; each peek used costs 100 points. A full board is a draw.

## Rules in detail

- A **9×9** board. You play the dark tiles and move first; the bot plays the light tiles.
- Take turns placing one tile on an empty square. **Five or more in a row** (across, down or diagonal)
  of one colour wins; the rules check this after every move, so you can also win without noticing.
- **Blind:** only the newest tile shows its colour. Every older tile lies **face down**: you see that the
  square is taken, not by whom. Remember your own tiles.
- **Peek:** twice a game you may turn every tile face up for **3 seconds**.
- The bot thinks for 0.8 s. It remembers each of its own tiles with 90% and each of yours with 80%
  (decided when the tile is placed); a tile it forgot counts as blocked for both sides. It scores every
  empty square by the lines it would make for itself and the lines it would block for you.
- Full board without five in a row: a draw.
- Points: **win** max(400, 1000 − 15 × (your tiles − 5) − 100 × peeks used); **draw** 250;
  **loss** 30 × your longest line.
- Moves: `{type:"place", cell}` (0–80, row by row), `{type:"peek"}`.
- View: the board (`null` empty, `"down"`, `"dark"`, `"light"`), the newest tile, whose turn, peeks left,
  whether a peek is showing, the result and the winning line at the end (then every tile is shown).
- Desktop: arrows move the cursor, Enter or Space places, P peeks.
- **Replay moments:** your winning line, named by its length: "Five in a row without a peek" when
  no peek was used, else "Five in a row on a blind board" ("Six in a row…" for a longer one).
  Public: the game ends there and every tile is shown. The house bot's win is no moment.
