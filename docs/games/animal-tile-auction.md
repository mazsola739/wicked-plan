# Animal Tile Auction

Main game, 3–6 seats (the page: you + 3 bots). Code: `crates/rules/src/games/animal_tile_auction.rs`.

- A shared **5×5 grid**. Four animals: **fox, owl, frog, bear** (7 tiles of each). At the start
  **5 tiles** are placed on random cells.
- Everyone gets a **secret pattern**: three animals in a line or an L shape, e.g. fox–owl–fox.
  A pattern may appear turned (any of 4 rotations, no mirror image). Everyone starts with **15 chips**.
- **12 auctions.** Each shows the next tile from the deck to everyone, then:
  1. **Bid** (secret, simultaneous, 15 s): 0 up to all your chips. Timeout: 0.
  2. The **single highest bidder** pays and wins the tile (the table sees who and for how much).
     A tie for the highest bid: nobody wins and the tile is lost.
  3. The winner **places** it on any empty cell (15 s; timeout: the best cell for their own pattern).
- **Score:** **25** per place on the grid where your whole pattern appears, **5** per animal in your best
  partial match (the most cells of your pattern that match anywhere), **+1** per chip left.
  All patterns are shown at the end. Points: 1000 for first down to 100 for last.
- Moves: `{type:"bid", chips}`, `{type:"place", cell}` (0–24, row by row).
- View: the grid, the tile on offer, your pattern, everyone's chips, who has bid, whose placement, the log.
- Bot: values the tile by how much it could grow its own pattern on the grid, paces its chips,
  and places the tile where it helps its pattern most.
- Desktop: digits + Enter bid; arrow keys move a cursor on the grid and Enter places.
