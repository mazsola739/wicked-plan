# Design: Wicked Plan (v4, 2026-11-23)

A **web page of puzzles and party games** in the genre of The Devil's Plan. One page lists every game;
pick one and play. Solo puzzles are played alone; games for several players are played against bots.

## Decisions so far

- **Platform:** a single-page React + TypeScript web app for desktop (wider layout, keyboard shortcuts)
  and phone (portrait, one hand). No native app.
- **Language:** everything in the product and the docs is English.
  - **Hungarian** (decided 2026-11-26): the page has an EN / HU switch in the top bar and starts in
    the browser's language (the choice is kept). In Hungarian everything is translated except
    the names: the game names stay the show's English names, with the Hungarian one under them in
    brackets, and the title stays "Wicked Plan" with "(Gonosz terv)" under it. The repo, code, docs
    and commits stay English. How: `apps/web/src/i18n.ts` (`t(english)`: the English text is the
    key, a missing translation shows the English); `locales/hu/common.ts` for the page,
    `locales/hu/<game-id>.ts` per game (Hungarian name and short, screen strings, patterns for the
    lines the rules write) and `locales/hu/howto/<game-id>.md` (the Hungarian How to play).
- **No rooms and no server for now:** the page is a game picker. A room/server layer may come later;
  the rules keep secrets out of the views so a server could referee without changes.
- **Rules in Rust:** one `crates/rules` crate, run in the browser through WebAssembly.
- **Bots:** in games for several players the other seats are bots that move by themselves
  (seats: the game's minimum, or 4 when it allows more).
- **The show's names, our own versions** (decided 2026-11-24): each game carries the show's official
  English name, as Netflix uses it (source: the English Wikipedia article on the show); the digital
  rules, texts, art and look are our own. The How to play of the complex games for several players
  ends with one round from the show, the contestants called Player A, B … (no real names).
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
  grin alone on transparent (the peeking face). The grin is also the small brand mark (top bar,
  footer; `Grin` in `Emblem.tsx`, the owner's `logo.webp`) and the favicon (`public/favicon.png`,
  transparent; the phone home-screen icon sits on a dark rounded square).
- **On the show** (2026-11-26): each game's card shows, small and quiet in its bottom-right corner,
  where the game was on the show: season and day (`S01 · D04`), `FINAL`, `EXILE n` or `HIDDEN`,
  from the English Wikipedia article (it gives days, not episodes); `apps/web/src/aired.ts`. The
  catalogue filters by it too, with two dropdowns side by side at the top of the sidebar: Season
  (any, 1–3) and Day (any, 1–6; finals, exile and hidden games are types). The two are the page's own
  dropdowns (`Dropdown.tsx`, the listbox pattern): a hexagon trigger with the chosen season's or
  day's colour as a dot (and, once one is chosen, the trigger looks like a chosen chip in that
  colour), and a dark list with a dot per row and a tick on the chosen one. Under them the Category chips: no "All" chip, a chip is off until picked,
  the pressed chip again turns it off ("Clear all" by the count resets everything). Players and
  Difficulty sit over the cards instead, as two rows of hexagon chips like the filters: All games ·
  Solo · Multiplayer (not solo: a game for several players), and under them smaller ones, Any level · Easy … Expert, each in its colour. The count of games shown sits
  under the "Games" title, with a quiet "× Clear all" beside it while anything (search, a filter,
  a tab) narrows the list, the one way back to all the games, and under it the types are jump buttons, each in its colour, that scroll
  to their group of cards; a type with no game left after the filters is grey and disabled. The
  sidebar's game list is A to Z.
  Metal Cast Puzzle, Rising Water Well and Unknown Language are not in the article and have none.
- **Filter icons** (the owner's art, 2026-11-26): neon icons for the seven types, five levels,
  seven categories and the two player modes (solo, multiplayer),
  `assets/ui-icons/<kind|level|category|mode>-<id>.webp` (128 px, transparent), named by the ids
  the code uses. Each card shows them as **badges**: hexagons like icon roundels, one size on every card, sized from the card's width so the most a card has (six) fill one row
  on the line between its picture and its text, half over the picture, at the left; only the edge carries
  the group's colour (the inside is neutral dark), and over the card the edge lights up
  bright with a glow around the hexagon (the inside stays dark) and the icon goes from 80% to full at 110% brightness: solo or multiplayer, the difficulty, the type, then the categories; the badge is the card's only difficulty
  mark (the name is the tooltip and what a screen reader says). The filter chips
  carry the same icon in front of the label (the type jumps, the levels and the categories; not the
  neutral "all"/"any" ones), so the filters are the key to the cards' badges; a chip's icon is at 80% at rest and full,
  a touch brighter (110%), when the chip lights up or is chosen.
- **Game icons** (the owner's art, 2026-11-23; two sets since 2026-11-26): two icons per game,
  256 px, transparent, shown by `GameIcon.tsx`, loaded only when in view. The metal set
  (`assets/icons/<game-id>.webp`) is small in the sidebar's game list, beside the title in the game
  (`GameTop`). The How to play shows no metal icon; its big title is the game's name.
  The game's card has no icon: its picture is the still of the game. The neon set
  (`assets/icons-neon/<game-id>.webp`) is on the How to play monitor, taking turns with the
  Observer (a flicker at each switch).
- **Palette** (the owner's colour system, 2026-11-26; the tokens are in `:root` of `styles.css`):
  near-black background (#08090F), graphite surfaces (#141520, raised #0E0F18), neutral edges
  (#343447); the mascot's **cyan** (#20E9FF) for focus and the main action and **violet** (#A855F7)
  for emphasis; **silver** for the title. Each kind of filter has its own colours, never one shared
  set: difficulty (easy #44D6A0, normal #609EFF, medium #EAC568, hard #F07891, expert #BE73FA), season (1 antique gold #D4AA62, 2 jade
  #52C6BA, 3 orchid #D77AB8), day (one indigo-to-violet scale, #60749F … #C079FF), category (logic
  #27D9ED, strategy #9B85F5, bluff #E27CAB, social #55B6A1, auction #DCA45C, memory #6B9DE5, speed
  #ED9566), players (solo #73B8E8, multiplayer #51D1C0) and the muted game types (main #50BEE9, mini
  #60C5A3, team #819DEB, prison #D18E7C, hidden #B184E2, exile #D881C1, final duels #B8B8CA). The
  neutral resets (All games, Any level, Any season, Any day) are cyan or plain. Small text keeps
  4.5:1 contrast: a too-dark accent is brightened for text (the darkest days on the card).
- **Filter buttons:** one size for every hexagon control (38 px high, one label size: `--hex-h`,
  `--hex-font`; the type jumps and the player and level rows also one width, `--hex-w`, three a
  row on a phone) and one behaviour for all: a dark fill with a hint of the accent, an edge half-way
  to it and the label in it; hover lights every hexagon button up like a lamp from its bottom edge (a bright
  line in its accent and a glow rising from it), with the full accent edge and a lighter label; chosen a stronger tint,
  a 2px edge and a bright label; pressed a moment darker; disabled grey (#11121A, #292A39, #66677D),
  not just faded; keyboard focus a bright #8DEEFF edge. No permanent glow. Hover, keyboard focus and an open dropdown
  share one glass look on every control (chips, type jumps, the player and level rows, dropdown
  triggers and rows). The
  dropdown list is an angular panel with hexagon rows. A game card on hover lifts a little with a
  quiet cyan-to-violet edge.
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
  - **Icons are faceted** (decided 2026-11-23, with the faceted animals of Zoo
    as the reference): every pictogram a game draws (animals, card suits,
    jewels, monsters, pieces, objects) is a low-poly SVG in its colour, like a cut gem: a bold
    silhouette split into angular facets, lit from the top left (white overlays at 10–50%
    opacity on the upper-left facets, black at 12–35% on the lower-right), dark cut-outs in the
    page's near-black (`eye`), and one or two tiny white diamond glints. It must still read at
    about 25 px. Board squares that *are* a colour can be cut stones in CSS too (a flat table and
    four lit bevels; Unknown), and pieces small cut gems. Grid lines, charts, numbers
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
