# Big or Small

Final duel, 2 players. Code: `crates/rules/src/games/big_or_small.rs`, screen: `apps/web/src/games/BigOrSmall.tsx`.

## How to play

**Goal:** finish with more chips than your opponent.

- You both hold the cards 1–9 and start with 10 chips. There are 9 rounds.
- Each round the dealer turns up a card. Then you both choose in secret, within 45 seconds.
- Tap one of your cards to play face down. Each card can be played only once.
- Bet on your **opponent's** card against the dealer's: tap **Smaller**, **Equal ×2** or **Bigger**.
- Pick a stake of 1–3 chips, then tap **Play**.
- When both have chosen, the cards are shown. A right bet wins its stake (equal pays double); a wrong bet loses it.
- If time runs out, a choice is made for you.
- Tip: count the cards your opponent has already played; the last rounds can be worked out.

## Rules in detail

- Each player holds the cards **1–9** (both hands are the same and known to both). Each starts with **10 chips**.
- **9 rounds.** Each round the dealer turns up a card from its own shuffled 1–9 deck.
- Both players then decide at the same time, in secret:
  - which of their cards to play face down (each card is played once, so the last rounds can be worked out
    by counting the cards already shown), and
  - a bet on the **opponent's** card compared with the dealer's: **bigger**, **smaller** or **equal**,
    with a stake of **1–3 chips** (0 only when you have no chips left).
- When both have chosen, the cards are shown. A right bet wins its stake (**equal pays double**), a wrong
  bet loses it.
- Turn timer: **45 s** per round; whoever has not chosen gets an automatic choice (the bot's).
- End: after round 9. Points: **100 + 20 × chips**. Most chips wins; equal chips share the rank.
- Moves: `{type:"play", card, guess:"bigger"|"smaller"|"equal", stake}`.
- View (seat "you"): round, dealer card, your hand, the opponent's remaining hand, both chip counts, who has
  chosen, the timer, and every finished round (dealer card, both cards, bets and chip changes). The
  opponent's choice stays secret until both have chosen.
- Bot: counts the opponent's remaining cards and bets on the outcome with the best expected value, staking
  more when it is sure. It plays a card whose relation to the dealer's card is the rarest in its own hand,
  so the opponent's likeliest guess is wrong.
- **Replay moments:** a right "equal" call with chips at stake ("Called {bot-1}'s 7 exactly (+6)"),
  a right 3-chip bet while the opponent's 3-chip bet fails ("Six-chip swing against {bot-1}"), and
  going from behind to ahead in round 9 ("Lead stolen in the last round (12–9)"). Public: both
  cards, bets and chips are shown once both have chosen.
