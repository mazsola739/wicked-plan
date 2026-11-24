# Four Player Three in a Row

Mini team game, 3 players (you and two bots). Code: `crates/rules/src/games/hidden_fourth_player.rs`,
screen: `apps/web/src/games/HiddenFourthPlayer.tsx`.

## How to play

**Goal:** make your own lines of three, while the team stops the Shadow from making one.

- You and two bots play on a 7×7 board, each in your own colour. The Shadow, run by the game, plays too.
- Each round you place a stone first, then each bot, then the Shadow. Tap an empty square.
- Three of one colour in a row (across, down or diagonal) is a line. Four in a row counts as 2 lines.
- If the Shadow makes a line, the game ends at once.
- If the board fills with no Shadow line, the team survives: everyone gets 100, plus 100 per own line, plus a 300 bonus.
- If the Shadow wins, everyone gets 100 plus only 50 per own line.
- **Tip:** blocking pays more than your own lines. Squares where the Shadow could finish a line are marked as danger.

**On the show:** Three players sat in separate rooms and played three-in-a-row against one another and an unknown fourth player, without being told how the colours were handed out. Their way to keep that fourth player from winning was to pick starting squares that only the three of them knew. In the end Player A wrongly took Player B for the impostor, and Player B was eliminated (here the Shadow always plays openly in its own colour, so nobody has to be unmasked).

## Rules in detail

Three players play three-in-a-row on one board, each in their own colour, against a fourth player nobody
controls: **the Shadow**, run by the rules. Its plan is unknown; all you see is where it plays.

- **7×7 board.** The Shadow opens with one stone on a square dealt by the seed.
- **Rounds:** the three players place one stone each, in seat order (you first), then the Shadow places
  one at once. Repeat.
- **Lines:** every run of **3 squares in a row** (across, down or diagonal) all in one colour is a line.
  Longer runs count once per 3-square stretch (4 in a row = 2 lines).
- **The Shadow wins** the moment it has a line: the game ends at once.
- **The team survives** when the board is full without a Shadow line.
- **The Shadow's play:** complete a line if it can; otherwise the square that gives it the most open
  twos (two Shadow stones and an empty square in a 3-stretch), ties dealt by the seed.
- **Replay moments:** a stone that makes two or more own lines at once ("2 lines with one stone"), and a
  stone that blocks a Shadow line and makes an own line ("One stone: Shadow blocked, line made"). Public:
  the whole board is open.

## Points (cooperation matters)

- Survived: **100 + 100 per own line + 300** team bonus for everyone.
- Shadow won: **100 + 50 per own line** (lines are worth half, no bonus).

So each player wants their own lines, but if nobody blocks, everybody loses most of their points. The
last player before the Shadow is the one who must cover the threats the others left open.

## Moves and view

- Moves: `{type:"place", x, y}` (0–6 from the top-left).
- View: the board (0 empty, 1–3 the seats in order, 4 the Shadow), your seat, each seat's lines, whose turn,
  the squares where the Shadow would complete a line now (danger), the last stones played, the round, and
  at the end whether the team survived.
- Bot: scores every empty square: blocking a Shadow line (a must when the threats outnumber the players
  still to move before the Shadow), blocking squares where the Shadow would make two threats at once, its
  own lines and open twos, a little for blocking the other players, and a little for the centre.
