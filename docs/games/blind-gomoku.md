# Blind Gomoku

Hidden game, solo against the house bot (1 seat). Code: `crates/rules/src/games/blind_gomoku.rs`.

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
