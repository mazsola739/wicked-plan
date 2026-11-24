# Halloween Monster

Main game, 4–8 players (the page seats you and 3 bots). Code: `crates/rules/src/games/monster_alliance.rs`.

## How to play

**Goal:** finish with the most points by helping to beat monsters and sharing in their rewards.

- You start with **20 points** and **8 secret weapon cards** of strength 1 to 6. Six monsters come, one each round.
- **Bid:** enter how many points you will pay for an extra weapon card, then tap **OK**. The highest bid wins it, but if the top bids are tied, nobody gets the card.
- **Alliance:** tap up to two partners, then tap **Ask them**, or tap **Fight alone**. An alliance forms only if they pick you too.
- **Attack:** tap the cards to throw, then tap **Attack**, or tap **Hold back**. Cards you play are used up.
- If your alliance's total power reaches the monster's strength, it beats the monster. The strongest alliance that beats it takes the reward.
- Inside the winning alliance, the reward is shared by how much power each member played.

**On the show:** A monster's points went to whoever landed the final blow, and an alliance pooled its points and split them evenly (here the strongest alliance takes the reward and shares it by power played). Three players finished with the highest scores and shared the 6 game tokens set aside for the top, 2 each.

## Rules in detail

Six monsters attack one by one. Bid points for extra weapons, form alliances of up to three, and
throw weapon cards at the monster. The strongest alliance that beats it shares the reward.

- **Replay moments:** a player fighting alone beats the monster with the single highest power and
  takes the whole reward ("Beat Rustjaw alone: all 27 points"). And an alliance that takes the
  reward of a monster of strength 16 or more alone (the single highest power, no tie), credited to
  the member who played the most ("Beat Hollow King,
  strength 17, with {bot-2}"). Public: the alliances and every played card are shown after the attack.

## Setup

- Everyone starts with **20 points** and **8 weapon cards** (strength 1–6 each, random). Hands are
  secret; how many cards each player holds is public.
- Six monsters, one per round. Monster *k* (1–6) has strength 6 + 2*k* + 0–4 (so 8–22) and a reward
  of one and a half times its strength (rounded down).

## A round

1. **Bid.** One extra weapon card (strength 4–7) is up for auction. Everyone bids points in
   secret (0 up to what they have). The highest bid wins the card and pays its bid; a tie for the top
   means nobody gets it.
2. **Alliance.** Everyone picks up to two partners in secret. Alliances form where the wish is
   mutual: first groups of three where all three picked each other, then pairs who picked each other.
   Everyone else fights alone. The alliances are then shown to all.
3. **Attack.** Everyone secretly plays any number of their weapon cards (also none). Played cards
   are used up. An alliance's power is the sum of its cards.
   - Alliances whose power reaches the monster's strength beat it; the **strongest** of them takes the
     reward (equal power: those alliances split it equally between all their members).
   - Inside the winning alliance the reward is shared by power played (rounded down; the rest goes to
     whoever played the most).
   - Every alliance's cards are shown after the attack.

## End and scoring

- After the sixth monster, rank by points (equal points share a rank).
- Points by rank: 1000 for first down to 100 for last, evenly spaced.

## Moves

- `{type:"bid", amount}`, `{type:"ally", partners:[seat, …]}` (at most two, not yourself),
  `{type:"attack", cards:[hand index, …]}`.

## Bot

- Bids roughly 1–1.5 points per point of strength, rarely more than a third of its points.
- Picks the two partners with the most cards, preferring those who picked it last round.
- Attacks with the fewest cards that cover its share of the monster (strength ÷ alliance size); if it
  cannot cover the share, it keeps its cards.

Desktop: digits 1–9 toggle hand cards, Enter confirms.
