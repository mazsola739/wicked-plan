# Crooked Cops

Main game, 4–8 players (the page seats you and 3 bots). Code: `crates/rules/src/games/thieves_and_cops.rs`.

## How to play

**Goal:** as a thief, collect hidden pieces without getting caught; as police, catch the thieves. One officer is secretly corrupt and helps the thieves.

- Everyone knows who the thieves are. Thieves and the corrupt officer know where the pieces are and always see the thieves.
- Each of the **12 rounds**, everyone moves at once: tap a lit station to move one stop, or **Stay**.
- An honest officer catches a thief by ending on the same station, or by passing them on the same stretch of line.
- Thieves take pieces by stepping on them. Honest police see the thieves only after rounds 2, 4, 6, 8 and 10, or where a piece is taken.
- The thieves' side wins by keeping **5 pieces**; otherwise the police win.
- At the end the police vote on who is corrupt. Unmasking them costs the corrupt officer 300 and gives each honest officer 100.

**On the show:** Two thieves, secretly helped by two corrupt officers, collected the 12 pieces they needed by round 11 of 15, and neither thief was ever arrested (here, at a table of four, there is one thief and one corrupt officer, and the thieves' side needs 5 pieces within 12 rounds on a smaller map). In the final vote neither corrupt officer was unmasked, and as no thief had been arrested either, all four players on the thieves' side won 3 game tokens each (here a win is worth 500 points).

## Rules in detail

Thieves sneak around a metro map collecting hidden pieces; the police hunt them. One police officer
is secretly **corrupt** and works for the thieves.

- **Replay moments:** an honest officer catching a thief ("{bot-2} caught at Central: 2 pieces
  lost"), and a thief taking the piece that wins it for the thieves in round 11 or 12 ("Winning piece
  taken at Market in round 11"). Public: every catch is announced, and where a piece was taken (and so
  by which thief) is shown to all.

## Map and roles

- 17 stations on five lines (a ring line and four crossing lines). Moving one stop along a line is one move.
- 7 **pieces** lie hidden on 7 stations (never a starting station).
- Roles: 1 thief (4–5 players), 2 thieves (6–7) or 3 thieves (8). Everyone else is police; **one** of them is corrupt.
  Who is a thief is public; who is corrupt is secret.
- Thieves and the corrupt officer know each other, see every thief all the time and know where the
  pieces are. Honest police see the thieves only when they are spotted.
- Thieves start on random ring stations; the police start in the middle (Central, Library, Stadium, …).

## Rounds (12)

- Everyone chooses at once: stay, or move one stop.
- All move together. A thief is **caught** if an honest officer ends on the same station, or they pass
  each other on the same stretch of line. The corrupt officer never catches anyone. A caught thief is
  out and the pieces they took are lost.
- A thief on a station with a piece takes it; everyone sees *where* a piece was taken (and so where
  that thief stands).
- **Sightings:** after rounds 2, 4, 6, 8 and 10 every free thief's station is shown to all.
- The game ends when the thieves have kept **5 pieces** together, when every thief is caught, or after round 12.

## Unmask

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
