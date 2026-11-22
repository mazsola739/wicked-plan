# Colour Count Bluff

Main game, 3–6 seats (the page: you + 3 bots). Code: `crates/rules/src/games/colour_count_bluff.rs`.

- A deck of 40 cards: **10 each of red, blue, green and yellow**. Everyone starts with **3 lives**.
- **Each round** the deck is shuffled and every player still in gets **4 hidden cards**; you see only yours.
- Turns go around the table from the round's starter (the loser of the last round, or the next player
  still in). On your turn, within **20 seconds**:
  - **Bet**: "there are at least *N* cards of colour *C* on the table". A bet must beat the last one:
    a higher *N*, or the same *N* with a colour further along the order red → blue → green → yellow.
  - **Call**: doubt the last bet. All cards are shown: if there are at least *N* of that colour, the
    caller loses a life; otherwise the bettor does. The round ends.
  The first player of a round must bet. Timeout: call, or (as first player) bet 1 of your most common colour.
- A player with no lives is out. The game ends when one player is left or after **12 rounds**.
- **Score:** 100 per life left, plus 1 per round survived. Points: 1000 for first down to 100 for last.
- Moves: `{type:"bet", count, colour}` (colour 0–3), `{type:"call"}`.
- View: your cards, everyone's lives and card count, the bet to beat, whose turn, the log, the last showdown.
- Bot: counts its own cards plus a quarter of the cards it cannot see; calls a bet well above that
  estimate, otherwise makes the safest raise.
- Desktop: R/B/G/Y choose a colour, + and − change the count, Enter bets, C calls.
