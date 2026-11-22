# Treasure Island

Main game, 3–6 players (on the page: you + 3 bots). Code: `crates/rules/src/games/treasure_island.rs`.

- **The island:** a 9 × 9 grid (81 squares). The **treasure** is buried on one secret square.
- **Clue boxes** stand on other squares. Opening a box shows an **arrow**: the direction from the box
  towards the treasure, one of 8 (straight north / south / east / west only when the treasure is in the
  same column or row). The treasure is never under a box.
- **Start:** every player gets **100 coins** and one **private box** of their own, already opened; nobody
  else sees where it is or what it says.
- **5 auctions:** all five auction boxes are on the map from the start. Each round one box is up; everyone
  bids **in secret** (0 up to their coins), **20 seconds**, no bid = 0. The highest bid wins: the winner pays
  and opens the box, and only they see its arrow. **Tied highest bids all pay and all see it.** If every
  bid is 0, the box stays closed. After each auction all bids are shown.
- **Dig:** then every player secretly picks one square to dig (**30 seconds**; no pick = the bot's pick).
- **Score:** dig points by the distance to the treasure (in king steps: diagonal counts as one):
  **0 → 500, 1 → 300, 2 → 150, 3 → 50**, farther → 0; **plus the coins you kept**.
  Highest total wins; equal totals share a rank.
- **View:** the map with every auction box, your own private box, the arrows of the boxes you opened,
  everyone's coins, who has bid, past bids. The treasure, other players' arrows and digs are shown only at
  the end.
- **Moves:** `{type:"bid", amount}`, `{type:"dig", x, y}` (x column, y row, 0–8).
- **Bot:** keeps the squares that fit its arrows; works out how many dig points the box up for auction
  would add on average, and bids 25–50% of that (at most 1.5 × its coins / auctions left); digs where its expected dig points are highest.
- **Screen:** the island in SVG (squares that still fit your arrows are lit); bid with a stepper and quick
  amounts; tap a square to dig. Desktop keys: ↑/↓ change the bid by 1, PgUp/PgDn by 10, Enter bids; when digging the arrows move the spade.
