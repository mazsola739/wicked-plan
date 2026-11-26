# Wall Go

Prison game, 2–3 players (the page plays 3: you and two bots). Code: `crates/rules/src/games/wall_territory.rs`,
screen: `apps/web/src/games/WallTerritory.tsx`.

## How to play

**Goal:** wall off the biggest area of the 7×7 board for your own two pieces.

**The table:** you and two bots, **2 pieces** each, on fixed squares. You take turns; every turn is one move and one wall.

- **Move.** Tap one of your pieces, then one of its lit squares: up to **2 steps**, straight or round one corner. Walls and other pieces block the way. Staying put (0 steps) is allowed too.
- **Build.** Then put a wall on one side of the square that piece stands on: tap **Top**, **Right**, **Bottom** or **Left**. The board edge already works as a wall, and a wall never comes down.
- **Sealed in.** Once no rival can reach any square your pieces can reach, your area is closed for good, and you skip your turns from then on.
- **End.** The game ends when every player is sealed in.
- **Score.** Count the squares of the areas that hold only your pieces: you get **100 + 20 per square**. Most squares wins; the same number of squares shares the place.

**Example:** your pieces end up in two areas, one of 12 squares and one of 5. That is 17 squares: 100 + 17 × 20 = **440** points.

**Watch out:** an area with nobody's pieces in it counts for no one, and an area where a rival's piece stands is not yours yet. Close your area before a rival walks in.

**On the show:** Three players took turns moving one of their pieces and building a wall beside it on a Go board (here the board is 7×7), until each player's pieces were walled off from everyone else's. Player A finished with the largest territory, won a prize and was safe from elimination. Player B and Player C then replayed the game by the same rules, and Player B, left with the smaller territory, was eliminated.

## Rules in detail

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
- Moves: `{type:"turn", piece, x, y, wall:"n"|"e"|"s"|"w"}` (piece 0 or 1; x, y the square it ends on);
  `{type:"pass"}` only when no turn is possible (cannot happen in practice).
- View: the board size, every piece with its owner, the walls (east and south side of each square), who
  owns each square's region (only when exactly one player has pieces there), each player's secured area
  and whether they are sealed in, whose turn, your pieces' reachable squares, the last turn.
- Bot: tries every move and wall and keeps the one with the best territory estimate: each square belongs
  to the player with the strictly nearest piece (walking around walls), plus a bonus for area already
  sealed; it compares its own share with the best opponent's.
- **Replay moments:** a turn whose wall adds 15 squares or more to the mover's secured area ("Wall
  seals off 21 squares"), and a turn that leaves a rival sealed in with 4 squares or fewer ("{bot-2}
  shut in with only 2 squares"). Public: the board is open to all.
