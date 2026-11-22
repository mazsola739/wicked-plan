# Secret Number

Main game, 4–8 seats (the page: you + 3 bots). Code: `crates/rules/src/games/secret_number.rs`.

- Everyone gets a **different secret number from 1 to 100**.
- **5 rounds**, each in two secret, simultaneous steps:
  1. **Trade** (20 s): pick another player, or skip. A trade is a swap of clues: **you get a clue about
     their number and they get the same kind of clue about yours**. The table sees who traded with whom.
     Clue kinds: odd or even · higher than X or not · the tens digit · the last digit · divisible by 3 or not.
  2. **Guess** (25 s): guess one other player's exact number, or pass.
     - Right: you score **+50**, they lose **30** and their number is **exposed** (public; nobody can
       guess it again).
     - Wrong: you lose **5**, and you are told privately whether their number is **higher or lower**.
     The table sees who guessed whom and whether it was right.
- Timeouts: skip the trade, pass the guess.
- At the end every player whose number was never exposed scores **+40**.
- Results: total score, ranked; points 1000 for first down to 100 for last.
- Moves: `{type:"trade", seat}`, `{type:"skip"}`, `{type:"guess", seat, number}`, `{type:"pass"}`.
- View: your number, your clues per player and how many numbers each player could still have,
  exposed numbers, scores, who has moved, the public log.
- Bot: keeps the list of numbers each player can still have; trades with the player it knows least
  about, guesses when a list is short (halving it when it is medium), more boldly in the last round.
- Desktop: A–H pick a player, S skips / P passes, digits + Enter guess.
