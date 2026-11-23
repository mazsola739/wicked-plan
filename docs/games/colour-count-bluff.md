# Colour Count Bluff

Main game, 3–6 seats (the page: you + 3 bots). Code: `crates/rules/src/games/colour_count_bluff.rs`.

## How to play

**Goal:** keep your lives longer than everyone else.

- Everyone starts with 3 lives. Each round everyone gets 4 hidden cards in red, blue, green and yellow; you see only yours.
- On your turn (20 seconds), either bet or call.
- **Bet:** claim "at least N cards of a colour" across everyone's cards. Pick the colour, set N with − and +, then tap **Bet**. You must beat the last bet: a higher N, or the same N with a colour later in red, blue, green, yellow.
- **Call:** tap **Call** to doubt the last bet. All cards are shown. If the bet holds, you lose a life; if not, the bettor does.
- The first player of a round must bet. On a timeout you call (or, as first player, bet 1 of your most common colour).
- With no lives left you are out. The game ends when one player is left or after 12 rounds.

## Rules in detail

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
- **Replay moments:** at a call, a bet at least 2 above the table's average (cards in play ÷ 4)
  that holds exactly, for the bettor ("Exactly 6 red: {bot-2}'s call fails"); or a bet 3 or more
  above what is there, for the caller ("{bot-1}'s 6 green caught: only 3 there"). Public: every
  hand is shown at the call.
