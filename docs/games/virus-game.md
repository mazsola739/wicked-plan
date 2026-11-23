# Virus Game

Main game, 6–10 players (the page seats you and 5 bots). Code: `crates/rules/src/games/virus_game.rs`.

## How to play

**Goal:** if you are in the city, find and quarantine the terrorists; if you are a terrorist, infect at least half of the others.

- You get a public number and a secret role: terrorist, officer, journalist, researcher or citizen. Only terrorists spread the virus.
- **Contact (25 s):** pick one player to meet. The officer learns if they are a terrorist, the journalist sees whom they met, and a researcher vaccinates them.
- **Test:** you learn if you are infected, and everyone sees how many new infections there were.
- **Report (20 s):** **Accuse** one player in public, or **Stay quiet**.
- **Vote (20 s):** vote someone into **Quarantine**, or **Nobody**. It takes at least 2 votes and no tie, and their side is then shown.
- There are **4 rounds**. The winning side gets 500 each, the losers 200; you get +150 for your secret mission.

## Rules in detail

Hidden roles. The **terrorists** spread a virus; the **city** (citizens, officer, journalist,
researchers) tries to find and quarantine them before too many people are infected.

## Roles

Every player gets a public number (#1, #2, …, shuffled) and a secret role:

| Players | Terrorists | Officer | Journalist | Researchers | Citizens |
|---|---|---|---|---|---|
| 6–7 | 2 | 1 | 1 | 1 | the rest |
| 8 | 2 | 1 | 1 | 2 | the rest |
| 9–10 | 3 | 1 | 1 | 2 | the rest |

- Terrorists know each other. They are carriers: never ill, and they infect.
- Everyone else knows only their own role.

## A round (4 rounds)

1. **Contact (25 s).** Every player who is not in quarantine meets one other such player, in secret.
   No choice in time: a random player. What a meeting does:
   - A meeting with a terrorist (either way round) infects the other player.
   - **Researcher:** the person they meet is vaccinated: cured and immune for the rest of the game
     (this is applied first, so it also stops an infection this round).
   - **Officer:** learns whether the person they meet is a terrorist.
   - **Journalist:** learns whom the person they meet met themselves, and whether that person
     was newly infected this round.
   - Infected citizens do not pass the virus on; only terrorists do.
2. **Test.** Everyone privately learns whether they are infected now; everyone sees how many people
   were newly infected this round.
3. **Report (20 s).** Everyone may publicly accuse one player ("#4 is a terrorist") or stay quiet.
   All reports appear together, signed with the reporter's number.
4. **Vote (20 s).** Everyone (quarantined players too) votes for one player to quarantine, or for
   nobody. The player with the most votes, at least 2 and no tie, goes into quarantine and their side
   (terrorist or not) is shown to all. Quarantined players make no more contacts and cannot be met.

The game ends after round 4, or as soon as every terrorist is in quarantine.

## Winning and scoring

- **Terrorists win** if, at the end, at least half of the non-terrorists are infected; otherwise the
  **city wins**.
- Points: 500 for each player of the winning side, 200 for the losers; +150 for your secret mission;
  +100 for a city player who is healthy at the end.
- Missions (one each, secret):
  - City: *stay healthy*, *meet the same player in two rounds*, *vote for a terrorist who goes into
    quarantine*, *never be accused*.
  - Terrorists: *infect the officer*, *never be quarantined*.
- At the end every role is shown (15 s, or tap *See results*).

## Moves

- `{type:"contact", target}`, `{type:"report", target}` (`target` may be `null`),
  `{type:"vote", target}` (`null` = nobody), `{type:"close"}`. `target` is a seat index.

## View

Your number, role, mission, last test result and your private notes (officer and journalist findings;
terrorists see their partners). Public: the numbers, who is in quarantine and their revealed side,
all reports, all votes, new infections per round.

## Bot

- Contacts: terrorists meet players they have not yet infected; the officer checks unchecked players;
  the journalist watches unwatched ones; researchers vaccinate unvaccinated ones; citizens avoid
  players under suspicion.
- Reports: the officer accuses a terrorist it found; the journalist accuses a player whose contact was
  newly infected; terrorists often accuse a city player (preferring whoever accused them).
- Votes: each accusation counts by the accuser's record (a reporter whose accusation was proven wrong
  is no longer trusted, a proven one counts double); vote for the most suspected player, or nobody.
  The officer votes for a terrorist it knows; terrorists vote for a suspected city player.
