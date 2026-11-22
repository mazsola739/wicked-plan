# Thieves and Cops

Main game, 4–8 players (the page seats you and 3 bots). Code: `crates/rules/src/games/thieves_and_cops.rs`.

Thieves sneak around a metro map collecting hidden pieces; the police hunt them. One police officer
is secretly **corrupt** and works for the thieves.

## Map and roles

- 17 stations on five lines (a ring line and four crossing lines). Moving one stop along a line is one move.
- 7 **pieces** lie hidden on 7 stations (never a starting station).
- Roles: 1 thief (4–5 players), 2 thieves (6–7) or 3 thieves (8). Everyone else is police; **one** of them is corrupt.
  Who is a thief is public; who is corrupt is secret.
- Thieves and the corrupt officer know each other, see every thief all the time and know where the
  pieces are. Honest police see the thieves only when they are spotted.
- Thieves start on random ring stations; the police start in the middle (Central, Library, Stadium, …).

## Rounds (12)

- Everyone chooses at once (**20 s**): stay, or move one stop. No choice in time: stay.
- All move together. A thief is **caught** if an honest officer ends on the same station, or they pass
  each other on the same stretch of line. The corrupt officer never catches anyone. A caught thief is
  out and the pieces they took are lost.
- A thief on a station with a piece takes it; everyone sees *where* a piece was taken (and so where
  that thief stands).
- **Sightings:** after rounds 2, 4, 6, 8 and 10 every free thief's station is shown to all.
- The game ends when the thieves have kept **5 pieces** together, when every thief is caught, or after round 12.

## Unmask (20 s)

At the end every police officer (the corrupt one too) names one other officer as corrupt. If the
corrupt officer gets the most votes (no tie), they are **unmasked**. Then everything is shown
(**15 s**, or tap *See results*).

## Scoring

- The thieves' side (thieves and the corrupt officer) **wins** with 5 kept pieces; otherwise the police win.
- Winning side 500 each, losing side 200 each.
- Thief: +50 for every piece kept. Honest officer: +200 for each catch.
- Unmasked: the corrupt officer loses 300 and every honest officer gets +100.

## Moves

- `{type:"go", to}` (`to` = the station you stand on to stay, or a neighbour), `{type:"accuse", target}`
  (seat index of another officer), `{type:"close"}`.

## Bot

In bot-only games the thieves win about 40–65% of the time, depending on the number of players.

- Thief: heads for the nearest piece, avoiding stations where an honest officer stands or could step.
- Honest officer: works out where each thief could be (from the last sighting and the rounds since)
  and walks toward the nearest such station.
- Corrupt officer: wanders, staying off the thieves' stations.
- Unmask: honest officers name the officer who was, on average, farthest from the thieves at the
  sightings; the corrupt officer names a random honest one.
