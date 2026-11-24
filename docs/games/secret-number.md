# Secret Number

Main game, 4–8 seats (the page: you + 3 bots). Code: `crates/rules/src/games/secret_number.rs`.

## How to play

**Goal:** guess the other players' secret numbers while keeping yours hidden.

- Everyone has a different secret number from 1 to 100. There are 5 rounds.
- **Trade:** tap a player to swap clues, or tap **Skip the trade**. You learn something about their number, and they learn the same kind of thing about yours.
- **Guess:** tap a player and type their number on the keypad, or tap **Pass**.
- A right guess scores +50, costs them 30, and shows their number to everyone.
- A wrong guess costs you 5, but tells you whether their number is higher or lower.
- At the end, everyone whose number was never shown scores +40. The highest total wins.

**On the show:** A large alliance saw Player A as a threat and plotted to knock Player A out, with help from Player B, who had allied with Player A. The plan worked and Player A was eliminated (here nobody is knocked out: a right guess costs its target 30 points). Player C, another of Player A's allies, also went out because of Player B's betrayal.

## Rules in detail

- Everyone gets a **different secret number from 1 to 100**.
- **5 rounds**, each in two secret, simultaneous steps:
  1. **Trade**: pick another player, or skip. A trade is a swap of clues: **you get a clue about
     their number and they get the same kind of clue about yours**. The table sees who traded with whom.
     Clue kinds: odd or even · higher than X or not · the tens digit · the last digit · divisible by 3 or not.
  2. **Guess**: guess one other player's exact number, or pass.
     - Right: you score **+50**, they lose **30** and their number is **exposed** (public; nobody can
       guess it again).
     - Wrong: you lose **5**, and you are told privately whether their number is **higher or lower**.
     The table sees who guessed whom and whether it was right.
- At the end every player whose number was never exposed scores **+40**.
- Results: total score, ranked; points 1000 for first down to 100 for last.
- Moves: `{type:"trade", seat}`, `{type:"skip"}`, `{type:"guess", seat, number}`, `{type:"pass"}`.
- View: your number, your clues per player and how many numbers each player could still have,
  exposed numbers, scores, who has moved, the public log.
- Bot: keeps the list of numbers each player can still have; trades with the player it knows least
  about, guesses when a list is short (halving it when it is medium), more boldly in the last round.
- Desktop: A–H pick a player, S skips / P passes, digits + Enter guess.
- **Replay moments:** a number cracked in round 1 or 2 ("{bot-2}'s number 47 cracked in round 1"),
  by the first seat that guessed it right; later, a long shot: a right guess while 10 or more numbers
  were still possible as far as the guesser knew ("Long shot: {bot-2}'s 47 out of 23 possible"). The
  crack is public (a right guess exposes the number to everyone); the long shot is shown only to the
  guesser, because how much it knew comes from its private clues.
