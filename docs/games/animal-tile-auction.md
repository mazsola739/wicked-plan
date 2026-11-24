# Animal Tile Auction

Main game, 3–6 seats (the page: you + 3 bots). Code: `crates/rules/src/games/animal_tile_auction.rs`.

## How to play

**Goal:** build your secret animal pattern on the shared 5×5 grid more often than anyone else.

- You get a secret pattern of three animals in a line or an L shape. It may appear turned, but not mirrored.
- You start with 15 chips. There are 12 auctions, each for the next tile shown to everyone.
- Set your bid with − and + or the quick buttons, then tap **Bid**. Bids are secret.
- Only the single highest bidder pays and wins the tile. A tie for the top bid: nobody gets it and the tile is lost.
- If you win, tap any empty cell to place the tile.
- You score 25 for every complete copy of your pattern, 5 per animal in your best partial match, and 1 per chip left.
- Others may place tiles to block you, so a cheap tile can matter as much as a dear one.

## Rules in detail

- A shared **5×5 grid**. Four animals: **fox, owl, frog, bear** (7 tiles of each). At the start
  **5 tiles** are placed on random cells.
- Everyone gets a **secret pattern**: three animals in a line or an L shape, e.g. fox–owl–fox.
  A pattern may appear turned (any of 4 rotations, no mirror image). Everyone starts with **15 chips**.
- **12 auctions.** Each shows the next tile from the deck to everyone, then:
  1. **Bid** (secret, simultaneous): 0 up to all your chips.
  2. The **single highest bidder** pays and wins the tile (the table sees who and for how much).
     A tie for the highest bid: nobody wins and the tile is lost.
  3. The winner **places** it on any empty cell.
- **Score:** **25** per place on the grid where your whole pattern appears, **5** per animal in your best
  partial match (the most cells of your pattern that match anywhere), **+1** per chip left.
  All patterns are shown at the end. Points: 1000 for first down to 100 for last.
- Moves: `{type:"bid", chips}`, `{type:"place", cell}` (0–24, row by row).
- View: the grid, the tile on offer, your pattern, everyone's chips, who has bid, whose placement, the log.
- Bot: values the tile by how much it could grow its own pattern on the grid, paces its chips,
  and places the tile where it helps its pattern most.
- Desktop: digits + Enter bid; arrow keys move a cursor on the grid and Enter places.
- **Replay moments:** a tile won with every chip left, 5 or more ("All 6 chips on the fox"), and a
  placement that completes a copy of the placer's own pattern ("Secret pattern completed (+25)").
  The all-in win is public (winning bids and chips are shown to the table); the completed pattern
  only the placer sees, since patterns stay secret until the end.
