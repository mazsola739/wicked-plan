# Hidden Rules Race

Main game, 4–8 players (the page seats you and 3 bots). Code: `crates/rules/src/games/hidden_rules_race.rs`.

## How to play

**Goal:** get your piece closest to the goal in the centre of the board.

- The 7×7 board has squares in five colours. Each colour has its own secret movement rule: step one, step two, backwards, turn right, or turn left.
- Each round, tap a direction (up, right, down or left) in secret within 20 seconds. Then all pieces move at once. If you don't choose, your piece stays.
- The rule used is the one of the square your piece starts on. A move off the board does not happen.
- Watch the last round's log: every piece's colour, direction and result is a clue. Tap a colour to note your guess.
- After rounds 6, 12 and 18, one rule is revealed to everyone.
- The race ends when a piece reaches the goal, or after 30 rounds. The closer to the goal, the better your place.

## Rules in detail

Everyone races a cube piece over a 7×7 board of coloured squares to the **goal in the centre**. How
a piece moves depends on the colour of the square it stands on, and those **movement rules are secret**:
you work them out from what happens to everyone's pieces.

## Board and rules

- 7×7 squares in five colours (red, blue, green, yellow, purple), random each game; the centre square
  is the goal.
- Starts: the four corners first, then the middle of each edge (up to 8 pieces). Pieces may share squares.
- Every colour has one secret movement rule, each rule on exactly one colour:

| Rule | You choose a direction (up, right, down, left) and the piece… |
|---|---|
| Step one | moves 1 square that way |
| Step two | moves 2 squares that way |
| Backwards | moves 1 square the opposite way |
| Turn right | moves 1 square, turned a quarter right (up becomes right) |
| Turn left | moves 1 square, turned a quarter left (up becomes left) |

- The rule used is the one of the square the piece **starts** on. A move that would leave the board
  does not happen: the piece stays.

## Rounds

- Up to **30 rounds**. Each round everyone picks a direction in secret (**20 s**; no choice = the
  piece stays). Then all pieces move at once.
- After each round everyone sees every piece's colour, chosen direction and result (the log).
- **Clues:** after rounds 6, 12 and 18 one rule is revealed to all (a colour not yet revealed).
- The race ends after the round in which a piece reaches the goal, or after round 30.
- At the end all the rules are shown (**15 s**, or tap *See results*).

## Scoring

- Rank by distance to the goal (steps along rows and columns); pieces on the goal share first place.
- Points by rank: 1000 for first down to 100 for last, evenly spaced.

## Moves

- `{type:"go", dir}`: `dir` is `up | right | down | left`.
- `{type:"close"}`: ends the rule reveal at the end.

## View

The board, all pieces, who has chosen (not what), the full public history, the revealed rules; all
rules after the race.

## Bot

Keeps, for every colour, the rules that fit everything seen so far (a rule pinned to one colour is
ruled out for the others), then picks the direction with the shortest expected distance to the goal.

Desktop: arrow keys choose a direction.
