# Design: Wicked Plan (v4, 2026-11-23)

A **web page of puzzles and party games** in the genre of The Devil's Plan. One page lists every game;
pick one and play. Solo puzzles are played alone; games for several players are played against bots.

## Decisions so far

- **Platform:** a single-page React + TypeScript web app for desktop (wider layout, keyboard shortcuts)
  and phone (portrait, one hand). No native app.
- **Language:** everything in the product and the docs is English.
- **No rooms and no server for now:** the page is a game picker. A room/server layer may come later;
  the rules keep secrets out of the views so a server could referee without changes.
- **Rules in Rust:** one `crates/rules` crate, run in the browser through WebAssembly.
- **Bots:** in games for several players the other seats are bots that move by themselves
  (seats: the game's minimum, or 4 when it allows more).
- **Original names and look:** the show's games are not copied one to one; we write our own versions.
- **All 51 catalogue games have a digital version** (done 2026-11-23).
- **Timers only where the game's rules give one** (decided 2026-11-24): a time limit, a race
  ("as fast as you can", "the fastest right answer wins") or time as the resource, as the game's
  line in `docs/games.md` says. Everywhere else you take your time: a phase ends when everyone has
  moved (bots move by themselves). Short result pauses stay, without a countdown bar. Spinning
  Puzzle Wheel keeps 12 s for the catalogue's 5, so a piece can be turned by touch.

## Visual identity (2026-11-23, from the Wicked Plan key art)

- **Emblem, the Observer:** the owner's own artwork, from the Wicked Plan poster (2026-11-23):
  `apps/web/src/assets/hood.webp`, a pointed hood seen from the front, a black void for a face drawn
  by glowing LED features (a cyan nested-diamond eye, a violet slash eye, a zigzag grin), shown large
  on the landing page, the How to play monitor and the results; and `face.webp`, its LED eyes and
  grin alone on transparent (the peeking face). The crown alone is the small mark (top bar, footer,
  favicon), drawn in SVG in `Emblem.tsx`.
- **Game icons** (the owner's art, 2026-11-23): one neon icon per game, `assets/icons/<game-id>.webp`
  (256 px, transparent), shown by `GameIcon.tsx`, loaded only when in view. It is small beside the
  name on the game's card (the card's picture is the still of the game) and in the sidebar's game
  list, beside the title in the game (`GameTop`), and on the How to play monitor, taking turns with
  the Observer (a flicker at each switch).
- **Palette:** near-black background, dark violet-grey surfaces, **cyan** for highlights and the main
  action, **violet** for fills and the second action, **silver** for the title and line art.
  Difficulty keeps green / yellow / red.
- **Type:** Cinzel (bundled, `@fontsource/cinzel`) for the title and headings; the system sans for
  everything else; tiny labels in letter-spaced capitals.
- **Shapes:** long hexagonal buttons (pointed ends); filter chips, step badges and difficulty pills
  are small hexagons too; thin divider lines with a small diamond in the middle, thin geometric line art. Glow only on the emblem and focus/hover; `prefers-reduced-motion`
  turns animation off.
- **Inside the games** (decided 2026-11-23): the games share the page's world, not each its own.
  - **Named game colours** are neon on near-black, from one set of tokens: `--g-red`, `--g-orange`,
    `--g-yellow` (also gold), `--g-green`, `--g-cyan`, `--g-blue`, `--g-purple`, `--g-pink`,
    `--g-white`, `--g-black`. A colour the rules name ("red") stays recognisably that colour, and a
    thing that *is* a colour (a red card, a yellow stack, a green square) is **filled** with it
    (softened a little toward the dark, e.g. `color-mix(in srgb, var(--g-red) 80%, #15121f)`), not
    dark glass with a coloured edge. Dark glass is for neutral things: number cards, playing cards.
  - **Seats** use one order everywhere: `--seat-0` is you (cyan), then violet, pink, yellow, green,
    orange, blue, white (`--seat-1`…`--seat-7`).
  - **Icons are faceted** (decided 2026-11-23, with the faceted animals of Animal Tile Auction
    as the reference): every pictogram a game draws (animals, card suits,
    jewels, monsters, pieces, objects) is a low-poly SVG in its colour, like a cut gem: a bold
    silhouette split into angular facets, lit from the top left (white overlays at 10–50%
    opacity on the upper-left facets, black at 12–35% on the lower-right), dark cut-outs in the
    page's near-black (`eye`), and one or two tiny white diamond glints. It must still read at
    about 25 px. Board squares that *are* a colour can be cut stones in CSS too (a flat table and
    four lit bevels; Hidden Rules Race), and pieces small cut gems. Grid lines, charts, numbers
    and text stay as they are.
  - **Surfaces:** no felt, wood, soil or sand; boards are `--board` (near-black) with thin silver
    `--board-line` lines. Cards and tiles are dark glass (`--glass`, edge `--glass-edge`) with light
    text and neon symbols, like the floating cards of the key art. Glow is soft and only on what
    matters (your pieces, a selection, a win).
- **Broadcast feel** (decided 2026-11-23): the games feel like a televised mind-game show where an
  unseen host watches and speaks to you over a live feed. Our own wording and look, no lines or
  images from films.
  - Opening a game switches it on like an old monitor (a short CRT flash, never blocking).
  - How to play is a **transmission from the Observer**: the hooded Observer on a scan-lined
    "monitor", a red LIVE dot (it blinks a few times, then stays) and a channel tag, always **CH 08**
    (a nod to another show; only the number, never a name; nothing counts up, the game
    waits). The Goal line types itself out, the bullets follow;
    a tap or any key shows everything at once. The monitor sits in the top-left corner and the
    text flows around it, to its right and below, so there is little to scroll. It opens by itself
    when a game starts, unless **Don't show again for this game** was ticked (per game, on this
    browser); the How to play button always opens it.
  - **The Observer watches**: on the landing page the hooded Observer's signal glitches every 7–16 s
    (one soft pulse and a cyan/violet split, under three flashes per second, WCAG 2.3.1). During a
    game only its **face** is there (`face.webp`), without the hood, behind the game like a
    background (never taking a tap), and mostly not at all. Only play time counts (not while the
    rules are open or after the end); after 10 minutes of play it all starts again.
    - **Once a game**, at a random moment 25 s–2.5 min in, large in the top-right corner (4 s): its
      piercing gaze comes in like a weak broadcast: far, small, blurred and looking away toward the
      game, it **turns to face you** and keeps coming closer as it sharpens, scan lines over it; the
      signal tears sideways once and blinks to a line once; it flickers and fades still staring.
    - **3–5 times**, at random moments in the first 4 minutes (at least 15 s apart), small,
      somewhere in the play area where bare background shows (2.6 s): it lifts its head, looks at
      you and lowers it again.
  - **Replays** (decided 2026-11-23): when anyone (you or a bot) makes a big move, the game freezes
    on that moment for about 3 s under a TV-style REPLAY banner: whose move it was and one line
    on what happened. The game clock stands still meanwhile (no ticks, bots wait); a tap or any key
    skips it. Each game names its own big moments in its rules (a capture, a sniped hand, a pot won
    with the lowest card, a perfect bid, a winning line...), rare enough to stay special, and its
    rules page lists them under "Replay moments". A replay never shows what that seat could not see.
  - With `prefers-reduced-motion` none of this moves: no flash, no typing, no glitches.
- **Landing:** the hooded Observer and the title on top (with a yellow "Under construction" pill
  while the page is shown publicly as work in progress), a short "How it works", then the
  catalogue with a search field and filters (type, category, difficulty, solo or with bots): from
  1024 px a sticky sidebar beside the cards, below that folded behind a Filters button that counts
  the active filters. No sign-in, lobby or leaderboard until rooms exist.

## Open questions (the owner decides)

- Should games keep a score history or best scores on the page?
- Rooms with friends: when, and with which server?

## Milestones

| # | Build | Done when |
|---|---|---|
| P1 | Rules in Rust + WebAssembly; the page with six solo puzzles | Every puzzle can be played through in the browser — done 2026-11-22 |
| P2 | Game picker page, bots for several seats, all 45 remaining games | Every catalogue game can be played through on desktop and phone, its tests (bot games) pass — done 2026-11-23 |
| P3 | Rooms with friends (server) | Two people play one game together |
