# Minus One

Prison game, 3–6 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/minus_one.rs`.

- Everyone holds the same ten cards, **1 to 10**. **7 rounds.**
- **Show (20 s):** everyone secretly picks **two different cards** from their hand. Then all pairs are
  shown at once. Timeout: your two lowest cards.
- **Keep (15 s):** looking at everyone's pair, everyone secretly keeps **one** of their two cards
  ("minus one"). Timeout: the lower card.
- **Result (5 s):** the kept cards are revealed. The **lowest card that nobody else kept** wins the round:
  its owner scores the **pot = the sum of all kept cards**. If no kept card is unique, nobody scores.
- Kept cards are spent; the other card of the pair goes back to the hand. So low cards are precious:
  spend them when they win a big pot.
- **Ranking:** most points; ties share a rank. Points by rank: 1000 for the first, 100 for the last,
  evenly between.
- **View:** your hand; everyone's score; during Keep, everyone's pair; after each round, everyone's kept
  card, the winner and the pot.
- **Moves:** `{type:"show", cards:[a,b]}`, `{type:"keep", card:a}`.
- **Bot:** shows mostly one low card and one other (sometimes two random ones, to bluff). To keep, it
  weighs each of its two cards: the chance it ends up the lowest unique card (others assumed to keep
  their lower card 60% of the time) times the pot, minus the value of spending a low card.
- Desktop: number keys 1–9 and 0 (for 10) pick cards, Enter confirms.
