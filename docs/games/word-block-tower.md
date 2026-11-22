# Word Block Tower

Team mini game, 1–4 players (on the page: you + 3 bot teammates). Code:
`crates/rules/src/games/word_block_tower.rs`.

- **3 rounds**, each with a **topic** (Animals, Fruits, Colours, Body, Kitchen, Weather; no topic twice) and a
  pile of **letter blocks**. The blocks are exactly the letters of one secret word per player from the topic.
- Players build in turn, one word each: on your turn spell a word **from the topic** with blocks still in the
  pile. The **last** player's word must use **every block that is left**. The round is won when the last
  word is built and no block is left.
- Words must be in the game's built-in topic list (English, 3–7 letters); one word only once per round. Any
  word that fits counts, not only the secret ones.
- **Take back:** instead of building, the player whose turn it is may take back the word built just before
  (its blocks return, that player builds again). Use it when the blocks left cannot finish the tower.
- **Turn order:** seat order, starting one seat later each round (so everyone gets to be first and last).
- **Turn timer:** 30 seconds; then a default move is made for you (the bot's choice).
- **Score (team), per round:** **300 − 30 per take-back − 60 per timed-out turn**, at least 50. Three rounds:
  up to 900. Every player gets the team's points and shares rank 1.
- **View:** topic, the blocks (and who used which), every built word, whose turn it is. The secret words are
  never sent.
- **Moves:** `{type:"word", word}`, `{type:"takeBack"}`.
- **Bot:** picks a topic word that fits the pile and still lets the remaining players finish (it searches the
  word list); if no such word exists it takes back the previous word.
- **Screen:** blocks as tiles; tap tiles to spell, ⌫ and Clear, Build. Desktop keys: type letters,
  Backspace, Enter builds.
