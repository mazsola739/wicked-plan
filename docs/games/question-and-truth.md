# Question and Truth

Final duel, 2 players. Code: `crates/rules/src/games/question_and_truth.rs`, screen: `apps/web/src/games/QuestionAndTruth.tsx`.

- Each player gets a secret row of the cards **1–8** in a random order (positions 1–8, left to right).
  You see your own row; the goal is to work out the opponent's.
- Each player has **10 tokens**. Players take turns (the first seat starts). On your turn either **ask one
  question** about the opponent's row, paying its cost, or make your **final guess**.
- Questions (always answered truthfully by the rules):

  | Question | Cost | Answer |
  |---|---|---|
  | Which card is at position P? | 3 | a card |
  | Where is card N? | 3 | a position |
  | What is the sum of the cards at positions P and Q? | 2 | a number |
  | Is card A left of card B? | 1 | yes / no |
  | Is the card at position P bigger than the card at position Q? | 1 | yes / no |
  | Is card N in the left half (positions 1–4)? | 1 | yes / no |

- Both players see every question and answer, also the ones about their own row.
- **Final guess:** the whole row at once. After guessing you stop; the opponent goes on alone until they
  guess. With no tokens left, the only move is the guess.
- Turn timer: **60 s**; when it runs out the bot plays your turn (a question or the guess).
- End: when both have guessed. Points: **100 per position right**, **+100** for a fully right row, and
  **10 per token left** at the time of the guess (about 0–1000). More points wins.
- Moves: `{type:"ask", question:{kind:"cardAt", pos}}`, `{kind:"whereIs", card}`, `{kind:"sum", a, b}`
  (positions), `{kind:"leftOf", a, b}` (cards), `{kind:"bigger", a, b}` (positions), `{kind:"leftHalf", card}`;
  `{type:"guess", order:[8 cards]}`. Positions and cards are 1–8.
- View: your row, both token counts, whose turn, both question logs with answers, who has guessed, the
  timer; after the end the opponent's row and both guesses.
- Bot: keeps every row (of the 40,320) that fits the answers so far, asks the question with the most
  expected information per token, and guesses as soon as one row is left (or the tokens run out, then the
  row that agrees most with the remaining ones).
