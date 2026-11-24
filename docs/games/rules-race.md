# Rules Race

Main game, 4–8 players (the page seats you and 3 bots). Code: `crates/rules/src/games/rules_race.rs`.

## How to play

**Goal:** be the first to reach square 18, or be furthest ahead after 15 rounds.

- At the start, pick one condition and one effect, then tap **Write my secret rule**. Your rule fires whenever another player's roll meets the condition, and it stays secret until it first fires.
- On your turn, tap **Roll the die**. The die has 1, 2, 3, 3, prison and escape.
- Numbers move you forward. Prison locks your piece unless you hold an escape card; escape gives you a card.
- In prison you skip turns until you roll escape or have sat out two turns.
- Group rules, such as **Bump**, apply to everyone. After every 3rd round, vote to switch one on or off, or tap **Keep them all**.
- Tip: a well-chosen rule can push rivals back or send them to prison.

**On the show:** While planning, the 12 players split into two camps, and the bigger camp's plan was to stockpile escape cards (here you can hold only one at a time). That plan paid off best, and luck settled the final order. The last three places cost game tokens: Player A came 10th and gave back one, Player B came 11th and gave back three, and Player C came last, gave back five and was out of the game (here you race 3 bots and every place just scores points).

## Rules in detail

A race along a track of squares 0–18. Everyone writes one secret **personal rule** at the start; the
**group rules** can be changed by vote. The first piece on square 18 wins.

- **Replay moments:** a personal rule that pushes a piece back off square 18 ("Rule pushes {bot-2}
  back off the goal"); a swap of 6 squares or more ("Swap with {bot-2}: from 3 to 12"); a rule that
  carries its owner onto square 18 on another player's roll ("Reaches the goal on {bot-2}'s roll"). The
  moment is the rule's owner's. Public: a rule that fires is shown to everyone anyway.

## Setup

- All pieces start on square 0.
- **Write:** everyone builds a personal rule from one condition and one effect, in secret.
- A personal rule stays secret until it fires for the first time; then everyone sees it.

Conditions (always about **another** player's roll):

| Condition | Fires when the roller… |
|---|---|
| rolls a 1 | rolls the 1 side |
| rolls a 3 | rolls a 3 side |
| rolls prison | rolls the prison side |
| lands on a multiple of 4 | moves onto square 4, 8, 12 or 16 |
| passes me | moves from behind (or level with) my piece to ahead of it |

Effects (the rule's owner is "I", the roller is "they"):

| Effect | |
|---|---|
| I move forward 2 | |
| they move back 2 | not below 0 |
| they go to prison | an escape card saves them |
| I swap places with them | only if they are ahead of me |
| I get an escape card | at most one card each |

## The die

Six sides: **1, 2, 3, 3, prison, escape**.

- A number moves you forward.
- **Prison:** your piece goes to prison, unless you hold an escape card (it is used up instead).
- **Escape:** you get an escape card; if you already have one, you move 1.
- **In prison** you do not move. Escape gets you out and moves you 2. Otherwise you sit out the turn;
  after your second turn inside you are released (unless the group rule *Strict prison* is on).

## A turn

Seats take turns in order. After the roll and its move,
the personal rules of the other players are checked, in seat order starting after the roller; every
rule whose condition holds fires once. Effects do not set off further rules.

## Group rules

| Group rule | At the start |
|---|---|
| **Bump:** landing on a square with other pieces sends them back 2 (not on 0 or 18). | on |
| **Strict prison:** only an escape roll gets you out of prison. | off |
| **Exact finish:** overshooting 18 bounces you back by the extra steps. | off |
| **Big three:** a 3 moves 4. | off |

After every 3rd round there is a **vote**: everyone secretly picks one group rule to switch,
or *keep*. The option with the most votes wins; a tie changes nothing. Votes are shown afterwards.

## End and scoring

- The game ends at once when a piece stands on square 18 (after the rules of that roll), or after
  15 rounds.
- Rank by square (higher is better); equal squares share a rank.
- Points by rank: 1000 for first down to 100 for last, evenly spaced.

## Moves

- `{type:"write", condition, effect}`: condition `rollsOne | rollsThree | rollsPrison | landsOnFour | passesMe`,
  effect `meForward2 | themBack2 | themPrison | swapIfAhead | meEscapeCard`.
- `{type:"roll"}` on your turn.
- `{type:"vote", rule}`: `rule` is 0–3, or `null` to keep.

## View

Everyone sees the track, prison, escape cards, the group rules, the log, and personal rules that have
fired; a seat sees its own rule always.

## Bot

Writes a hurting rule (mostly *they move back 2* or *they go to prison*). Votes: the leader keeps
everything; others switch on *Exact finish* when the leader is within 4 of the goal, otherwise switch
*Bump* on if it is off, otherwise keep.

Desktop: Space or Enter rolls.
