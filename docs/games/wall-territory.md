# Wall Territory

Prison game, 2–3 players (the page plays 3: you and two bots). Code: `crates/rules/src/games/wall_territory.rs`,
screen: `apps/web/src/games/WallTerritory.tsx`.

- A **7×7** board of squares. Each player has **2 pieces** on fixed start squares (which start set a seat
  gets is dealt by the seed):
  - set A: (1,1) and (5,5); set B: (5,1) and (1,5); set C (3 players only): (3,0) and (3,6) — x, y from the
    top-left, 0–6.
- Turns go round the table. On your turn:
  1. **move** one of your pieces **0, 1 or 2 squares** orthogonally (turning is allowed), never through a
     wall or onto/through another piece;
  2. **build a wall** on one side of the square that piece now stands on. The side must not be the board
     edge and must not have a wall yet.
- **Regions** are the groups of squares that are still connected through sides without walls. A player
  whose pieces all stand in regions with no other player's pieces is **sealed in** and skips their turns
  (walls never come down, so that is final).
- **End:** when every player is sealed in (no region holds pieces of two players).
- **Score:** the area (number of squares) of the regions that hold only your pieces. Regions without
  pieces count for nobody. Points: **100 + 20 × area**; most area wins, equal area shares the rank.
- Turn timer: **60 s**; when it runs out, the bot plays your turn.
- Moves: `{type:"turn", piece, x, y, wall:"n"|"e"|"s"|"w"}` (piece 0 or 1; x, y the square it ends on);
  `{type:"pass"}` only when no turn is possible (cannot happen in practice).
- View: the board size, every piece with its owner, the walls (east and south side of each square), who
  owns each square's region (only when exactly one player has pieces there), each player's secured area
  and whether they are sealed in, whose turn, your pieces' reachable squares, the last turn, the timer.
- Bot: tries every move and wall and keeps the one with the best territory estimate: each square belongs
  to the player with the strictly nearest piece (walking around walls), plus a bonus for area already
  sealed; it compares its own share with the best opponent's.
