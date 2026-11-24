# Balance Mancala

Main game, 2–4 players (on the page: you + 3 bots). Code: `crates/rules/src/games/colour_mancala.rs`.

## How to play

**Goal:** fill your store with stones, balancing all three colours.

- The board is a ring. You own a row of 5 pits and a store; each pit starts with 4 stones in red, green and blue.
- On your turn, tap a colour under **Sow first**, then tap one of your pits.
- You drop its stones one by one around the ring: the chosen colour first, then the next colours in order. You skip other players' stores.
- If your last stone lands in your own store, you move again.
- If it lands in an empty pit of your own and the facing pit has stones, you capture both into your store.
- **The catch:** only your weakest colour really counts. Points are 50 × your lowest colour total plus all your stones.

**On the show:** a player's final score was also the lowest of the three colour totals, and everyone needed at least 15 points (here there is no such bar: points are 50 × your lowest colour plus all your stones). Nobody reached 15, so every player lost half of their game tokens. The player with the lowest score then had to give their last 4 tokens to the top scorer and was out of the game.

## Rules in detail

- **Board:** a ring. Every player owns a row of **5 pits** followed by their own **store**. Players sit
  around the ring in seat order.
- **Stones** come in three colours: **red, green, blue**. At the start every pit holds **4 stones**;
  the colour pattern is random but the same in every player's row, so nobody starts ahead.
- **Turn:** pick one of your non-empty pits and a **first colour**. Take all its stones and sow them one by
  one along the ring: first every stone of the chosen colour, then the next colour (red → green → blue →
  red), then the last. Sowing passes through your own store but skips the other players' stores.
- **Extra turn:** if the last stone lands in your own store, you move again.
- **Capture:** if the last stone lands in an **empty pit of your own** and the facing pit (counting pits 1–5,
  pit 6 − i of the next player's row) has stones, both pits go into your store.
- **End:** when the player to move has no stones left in their pits, or after **40 turns per player**.
  Every player then moves the stones left in their own pits into their own store.
- **Score:** your **lowest colour total** in the store counts. Points = **50 × lowest colour + total stones**
  (so the total breaks ties). Highest points win; equal points share a rank.
- **View:** everything is public (all pits, stores, whose turn, the last move's path).
- **Moves:** `{type:"sow", pit, first}`: `pit` 0–4 in your own row, `first` 0 = red, 1 = green, 2 = blue.
- **Bot:** tries every pit and first colour, and picks the one that raises its own points most
  (an extra turn and a capture count extra), with a little noise.
- **Screen:** the ring drawn in SVG (your row at the bottom); below it the colour toggle and your five pits
  as big buttons. Desktop keys: 1–5 sow from a pit, R / G / B pick the first colour.
- **Replay moments:** a capture of 10 or more stones, counting the capturing stone
  ("12 stones captured from {bot-1}"). Public: the whole board is open.
