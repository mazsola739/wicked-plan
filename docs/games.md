# Game catalogue

Every puzzle and game planned for Wicked Plan. Sources: `Wicked Plan 1 - Design Plan.pdf` (Game Catalogue,
41 games) and the NamuWiki episode pages of all three seasons (season 3 up to day 4 and Exile match 5;
checked 2026-11-22). The genre and mechanics follow The Devil's Plan, but **names, art and texts are
our own** (working names, open to change); classic board games (mill, gomoku, knight's tour) keep their names.

Types: **Main** (everyone, the day's main game), **Mini team**, **Mini solo**, **Prison** (the losers' game),
**Hidden** (found and played alone), **Final duel** (one against one). Difficulty and minimum players are estimates; the difficulty is also in each game's `GameInfo` and colours its card on the page (green easy, yellow medium, red hard).

Status: ✅ the rules are in `crates/rules/src/games/<id>.rs` and the page plays it (all 51 since 2026-11-23).
The exact rules of each digital version are on its page in `docs/games/<id>.md`; they adapt the short lines below.
Every finished game gets a rules page in `docs/games/<id>.md`.

## Main games

| Game | Difficulty | Min. | In short |
|---|---|---|---|
| ✅ Rules Race | medium | 4 | Board race where each player writes a personal rule; the die has prison and escape sides. |
| ✅ Animal Tile Auction | medium | 4 | Bid chips for animal tiles on a 5×5 grid that fit your secret pattern. |
| ✅ Equation Hi-Lo | medium | 3 | Build an equation from number and symbol cards, then bet on being closest to 1 or to 20. |
| ✅ Colour Count Bluff | medium | 3 | Bet on how many cards of a colour are on the table; raise or call the bluff. |
| ✅ Hidden Rules Race | hard | 8 | Race a cube to the centre square while working out the secret movement rules. |
| ✅ Virus Game | hard | 6 | Hidden roles: terrorists spread a virus; citizens, officer, journalist and researchers stop it. |
| ✅ Either Or | hard | 6 | Ten two-way choices for points; four surprise mini games turn up along the way. |
| ✅ Thieves and Cops | hard | 6 | Thieves collect hidden pieces on a metro map, cop teams hunt them, with secret corrupt cops. |
| ✅ Secret Number | hard | 4 | Everyone has a secret number from 1 to 100; trade clues, guess the others, protect your own. |
| ✅ Monster Alliance | hard | 4 | Form alliances, bid points, attack monsters with weapon cards and share the score. |
| ✅ Mystery Stock Market | hard | 4 | Trade made-up stocks over 24 rounds and work out the hidden rules that move the prices. |
| ✅ Polyomino Territory | hard | 3 | Grow your territory with polyomino pieces on a 30×30 grid; key squares give special moves. |
| ✅ Treasure Island | hard | 3 | Bid for arrows that open clue boxes on an 81-square island and find the treasure. |
| ✅ Colour Mancala | hard | 3 | Round mancala; your score is the lowest of your three colour totals. |
| ✅ Jewel Black Market | medium | 4 | Everyone holds one hidden card (jewel or bomb); in two swap rounds trade with others, the buyer's 3 cards or the discard. Score when the buyer has your jewel; a bomb costs points. 3 rounds. |
| ✅ Auction Bingo | medium | 4 | A 3×3 bingo board of circles, triangles and stars. Over 7 rounds pick a shape in secret, then bid coins for it; who won stays secret. |

## Mini games, solo

| Game | Difficulty | Min. | In short |
|---|---|---|---|
| ✅ Count the Cubes | easy | 1 | Count the cubes in a 3D drawing; the fastest right answer wins. |
| ✅ Count the Rectangles | easy | 1 | Count every rectangle in a figure, as fast as you can. |
| ✅ Lay the Grass | medium | 2 | Short Polyomino Territory: place pieces, the largest area wins. |
| ✅ Memory Poker | medium | 2 | Memorise 16 face-down cards, then make the best hand with the shared cards. |

## Mini games, team

| Game | Difficulty | Min. | In short |
|---|---|---|---|
| ✅ Word Block Tower | easy | 2 | Use every letter block to spell one word each in a given topic. |
| ✅ Duplicate Picture Buzzer | easy | 2 | Watch fast photo slides and buzz the moment one repeats. |
| ✅ Spinning Puzzle Wheel | medium | 4 | Rotate past shape puzzles; everyone gets 5 seconds to place a piece. |
| ✅ Memory Scene Quiz | medium | 2 | Study a scene, then 10 questions in a row; one wrong answer fails the team. |
| ✅ Weight Balance Puzzle | hard | 2 | In separate rooms, use a shared virtual scale to find the weights of five blocks. |
| ✅ Hidden Fourth Player | hard | 3 | Three in a row against an unknown fourth player you have to block together. |

## Prison games

| Game | Difficulty | Min. | In short |
|---|---|---|---|
| ✅ Time Auction | easy | 2 | Hold a button to bid time from a limited bank; the longest press wins the round. |
| ✅ Minus One | easy | 3 | Show two cards, keep one; the lowest unique card wins the round. |
| ✅ Metal Cast Puzzle | easy | 1 | A solo metal puzzle in prison, for an extra piece. |
| ✅ Sniper Hold'em | medium | 2 | Poker where you can "snipe" a hand to knock it out. |
| ✅ Dice Prediction | medium | 2 | Build a three-dice hand with hidden dice, then predict your own score. |
| ✅ Equation Target | medium | 2 | Pick 3 of 10 numbers and combine them to hit the target number. |
| ✅ Heavy 13 | hard | 3 | Number cards, rainbow chips and passes stack coloured rocks; an overloaded stack costs points. |
| ✅ Russian Roulette | hard | 3 | Spend 16 credits on clues, barrel spins and secret tampering to dodge the bullet. |
| ✅ Wall Territory | hard | 2 | Move pieces and build walls on a go board to enclose the largest area. (Season 3 version: two pieces that move differently, one wall break.) |
| ✅ Piggy Bank | medium | 3 | Drop coins into a piggy bank and pass it on; whoever gets it says how many coins of a colour are inside. Others may call doubt: it is opened, the wrong side loses a point. |

## Hidden games

| Game | Difficulty | Min. | In short |
|---|---|---|---|
| ✅ Blind Gomoku | hard | 1 | Five in a row with face-down tiles; remember which tiles are yours. |
| ✅ Knight's Tour | hard | 1 | Move a knight over 6×6, 7×7 and 8×8 boards without repeating a square, in one hour. (Ours: 5×5, 6×6, 7×7 in 15 minutes.) |
| ✅ Rising Water Well | hard | 1 | Decode picture clues to find a key before the well fills with water in 10 minutes. |

## Exile (the way back for eliminated players)

Eliminated players go to a hidden place. They can come back in two ways: crack the place's secrets and pass
the trial, or win the exile matches until day 6. The source names the matches but not their rules yet, so
these rows are sketches from the names.

| Game | Difficulty | Min. | In short |
|---|---|---|---|
| ✅ Unknown Language | hard | 1 | Decode a made-up picture script (root + part-of-speech mark) and read the place's rules and how to open the trial. |
| ✅ Magic Square Trial | hard | 1 | A 7×7 magic square, 1–49, with the 1 given; one hour. (Ours: 3×3, 4×4, 5×5 with a few numbers given.) |
| ✅ Piece Puzzle | medium | 1 | A tangram-like piece puzzle, as a race. |
| ✅ Find the Differences | easy | 1 | Spot the differences between two pictures, as a race. |
| ✅ Secret Code | medium | 1 | Code breaking, as a race. |
| ✅ Fence Building | medium | 1 | A fence-building logic puzzle, as a race. |
| ✅ Symmetry Puzzle | medium | 1 | A symmetry puzzle, as a race. |

## Final duels

| Game | Difficulty | Min. | In short |
|---|---|---|---|
| ✅ Big or Small | easy | 2 | Bet whether the opponent's hidden card is bigger or smaller than the dealer's. |
| ✅ Nine Men's Morris | medium | 2 | The mill game: make lines of three to take the opponent's pieces. |
| ✅ Hexagon | medium | 2 | 19 numbers on a hexagon are shown for 30 seconds, then turned over; each round name three neighbouring cells in a line that make the target. Memory and mental arithmetic. |
| ✅ Tiger and Goat | hard | 2 | Play two boards at once: tigers on one, goats on the other. |
| ✅ Question and Truth | hard | 2 | Spend tokens on questions to work out the opponent's secret order of 8 cards. |

## Categories

Every game has 1 to 3 categories (in its `GameInfo` as `tags`); the page filters by them and the search finds them.

| Category | Means | Games |
|---|---|---|
| **Cards** (`cards`) | played with cards | Big or Small, Colour Count Bluff, Equation Hi-Lo, Heavy 13, Jewel Black Market, Memory Poker, Minus One, Monster Alliance, Sniper Hold'em |
| **Bluff** (`bluff`) | lying, bluffing or reading bluffs is central | Colour Count Bluff, Equation Hi-Lo, Minus One, Piggy Bank, Sniper Hold'em, Virus Game |
| **Secret role** (`secret-role`) | hidden identities, roles or powers | Rules Race, Thieves and Cops, Virus Game |
| **Bidding** (`bidding`) | auctions and bids | Animal Tile Auction, Auction Bingo, Monster Alliance, Time Auction, Treasure Island |
| **Memory** (`memory`) | remembering what you saw | Blind Gomoku, Duplicate Picture Buzzer, Hexagon, Jewel Black Market, Memory Poker, Memory Scene Quiz |
| **Logic** (`logic`) | a puzzle or deduction with one right answer | Dice Prediction, Fence Building, Hidden Rules Race, Knight's Tour, Magic Square, Metal Cast Puzzle, Mystery Stock Market, Piece Puzzle, Question and Truth, Rising Water Well, Secret Code, Secret Number, Spinning Puzzle Wheel, Symmetry Puzzle, Treasure Island, Unknown Language, Weight Balance Puzzle |
| **Numbers** (`numbers`) | arithmetic, counting, number sense | Big or Small, Colour Mancala, Count the Cubes, Count the Rectangles, Equation Hi-Lo, Equation Target, Heavy 13, Hexagon, Magic Square, Mystery Stock Market, Piggy Bank, Rising Water Well, Russian Roulette, Secret Number, Weight Balance Puzzle |
| **Board** (`board`) | placing or moving pieces on a board or map | Animal Tile Auction, Blind Gomoku, Colour Mancala, Hidden Fourth Player, Hidden Rules Race, Knight's Tour, Lay the Grass, Nine Men's Morris, Polyomino Territory, Thieves and Cops, Tiger and Goat, Wall Territory |
| **Dice** (`dice`) | dice are central | Dice Prediction, Rules Race, Secret Code |
| **Words** (`words`) | words, letters or language | Either Or, Unknown Language, Word Block Tower |
| **Speed** (`speed`) | speed or reflexes decide | Count the Cubes, Count the Rectangles, Duplicate Picture Buzzer, Fence Building, Find the Differences, Piece Puzzle, Symmetry Puzzle |
| **Alliance** (`alliance`) | teams, cooperation, negotiation | Either Or, Hidden Fourth Player, Memory Scene Quiz, Monster Alliance, Spinning Puzzle Wheel, Thieves and Cops, Virus Game, Weight Balance Puzzle, Word Block Tower |

## Still missing

- Season 3, days 5–7: not out yet (the NamuWiki pages are empty).
- The exact rules of the exile matches.
- Dice Poker: the third game of the season 1 final; never played, rules unknown.

NamuWiki refuses automated fetching; it can be read in the built-in browser.
