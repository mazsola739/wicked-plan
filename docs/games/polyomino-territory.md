# Laying Grass

Main game, 2–4 players (on the page: you + 3 bots). Code: `crates/rules/src/games/polyomino_territory.rs`.
Our short version of the 30 × 30 original: a **14 × 14** board.

## How to play

**Goal:** own the most squares on the 14 × 14 board.

- You start with one square in a corner and hold 3 pieces. After each placement you draw a new one.
- On your turn, pick a piece, use **↻ Turn** or **⇋ Flip**, tap the board to put it there, then tap **Place**.
- A piece must cover empty squares and touch your own squares edge to edge.
- Cover a **⟳ Encore** square and you place again at once.
- Cover a **↗ Leap** square to earn a leap. Tap **↗ Leap** before placing, and that piece may go anywhere empty.
- If you cannot place any piece, you are skipped.
- The game ends when nobody can place, or after 40 turns each. Every square you own is worth 10 points; the highest score wins.

## Rules in detail

- **Start:** every player owns one start square: the corners (with 3 players: two top corners and the
  middle of the bottom edge).
- **Hand:** everyone holds **3 pieces** (polyominoes of 1–5 squares, drawn at random from 17 shapes) and
  draws a new one after placing. Only you see your hand.
- **Turn:** place one piece from your hand, turned and flipped any way, on empty squares, so that at least
  one of its squares touches your territory **edge to edge**. Its squares become yours.
- **Key squares (8, placed the same in every quarter of the board):**
  - **⟳ Encore:** cover it and you place again at once.
  - **↗ Leap:** cover it and you gain a leap: one later piece may go **anywhere** empty, touching nothing.
- **Stuck:** a player who cannot place any piece is skipped. The game ends when **nobody** can place, or
  after **40 turns** per player.
- **Score:** **10 points per square** you own. Highest wins; equal scores share a rank.
- **View:** the board, the key squares left, everyone's leaps and squares, your own hand.
- **Moves:** `{type:"place", piece, cells:[[x,y]…], leap}`: `piece` is the hand index, `cells` the board
  squares it covers (any turn or flip of the shape), `leap` true to use a leap.
- **Bot:** tries every piece, orientation and place; prefers big pieces, key squares, more room to grow and
  squares next to rivals; saves leaps until they are worth it.
- **Screen:** board in SVG; pick a piece, turn ↻ / flip ⇋ it, tap the board to put the ghost there, then
  Place. Desktop keys: 1–3 pick a piece, R turns, F flips, arrows move the ghost, Enter places, L toggles the
  leap.
- **Replay moments:** a piece placed with a leap that lands on a key square ("Leap grabs an encore
  square", "Leap grabs another leap", "Leap grabs two key squares"). Public: the board, the key squares
  and everyone's leaps are shown to everyone.
