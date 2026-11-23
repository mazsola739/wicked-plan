# Russian Roulette

Prison game, 3–6 seats (on the page: you + 3 bots). Code: `crates/rules/src/games/russian_roulette.rs`.
No gore: the drum holds **paint charges**; a hit costs a life, and a player with no lives is out.

## How to play

**Goal:** keep your lives longer than everyone else at the table.

- A drum has 6 chambers, 2 of them charged with paint. You start with 2 lives and 16 credits; only you see your credits.
- On your turn (20 s) you may spend credits first, then you must tap **Pull**. A charged chamber costs you a life.
- **Peek (3)** shows you alone whether the next chamber is charged.
- **Spin (2)** turns the drum to a random chamber; everyone sees it.
- **Tamper (4)** secretly moves the drum one chamber on without firing.
- When time runs out, you pull. When both charges are spent, the drum is reloaded.
- The game ends when one player has lives left, or after 6 pulls each. Most lives wins; equal lives go to more credits left.

## Rules in detail

- **The drum** has **6 chambers**, **2 of them charged**, in secret places. It turns one chamber after
  every pull. When both charges are spent it is **reloaded** (new secret places; everyone sees that).
- Everyone starts with **2 lives** and **16 credits**. Credits are secret: only you see yours.
- **Turns** go round the table (players who are out are skipped). On your turn (**20 s**) you may spend
  credits on any number of actions, then you must **pull**:
  - **Peek (3 credits):** see privately whether the chamber about to fire is charged.
  - **Spin (2 credits):** spin the drum to a random chamber. Everyone sees the spin.
  - **Tamper (4 credits):** secretly turn the drum one chamber on without firing. Nobody else sees it.
  - **Pull:** fire the current chamber. Charged: you are **hit** and lose a life (the charge is spent).
    Empty: click, you are safe. Either way the drum turns one chamber. Timeout: pull.
- After each pull the table sees the outcome for **2.5 s**.
- The game ends when **only one player has lives left**, or after **6 pulls per player** (24 at a table
  of four).
- **Ranking:** most lives left; equal lives: more credits left. Points by rank: 1000 for the first,
  100 for the last, evenly between.
- **View:** your lives, credits and peek result; everyone's lives; charges left in the drum and pulls
  since the last spin or reload (so you can work out the odds); a log of the spins, pulls and hits.
  Peeks and tampering never show up.
- **Moves:** `{type:"peek"}`, `{type:"spin"}`, `{type:"tamper"}`, `{type:"pull"}`.
- **Bot:** works out the odds from what the table has seen (charges left ÷ chambers not yet fired since
  the last spin). When the odds are bad and it can pay, it peeks; if the peek shows a charge it tampers
  (or spins when short of credits); with very bad odds and no peek money it spins. It keeps a few
  credits in reserve early on.
- Desktop: P peek, S spin, T tamper, Space or Enter pull.
