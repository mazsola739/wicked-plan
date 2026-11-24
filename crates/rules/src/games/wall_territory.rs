//! Wall Go (prison, 2–3 players). Rules page: docs/games/wall-territory.md
//!
//! 7×7 squares, two pieces each. A turn moves one piece 0–2 squares, then
//! builds a wall on a side of its square. The game ends when no region (squares
//! still connected through open sides) holds pieces of two players; each player
//! scores the area of the regions that hold only their pieces.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const N: usize = 7;
const CELLS: usize = N * N;
pub const STEPS: usize = 2;

const STARTS: [[(usize, usize); 2]; 3] = [[(1, 1), (5, 5)], [(5, 1), (1, 5)], [(3, 0), (3, 6)]];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dir {
    N,
    E,
    S,
    W,
}

const DIRS: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];

/// Walls and pieces: everything the rules and the bot look at.
#[derive(Clone, Debug)]
pub struct Board {
    /// A wall on the east side of each square.
    pub east: [bool; CELLS],
    /// A wall on the south side of each square.
    pub south: [bool; CELLS],
    /// pieces[seat] = the squares of that seat's two pieces.
    pub pieces: Vec<[usize; 2]>,
}

impl Board {
    /// The neighbour of `c` across side `d`, if that side is inside the board.
    fn across(c: usize, d: Dir) -> Option<usize> {
        let (x, y) = (c % N, c / N);
        match d {
            Dir::N => (y > 0).then(|| c - N),
            Dir::S => (y + 1 < N).then(|| c + N),
            Dir::W => (x > 0).then(|| c - 1),
            Dir::E => (x + 1 < N).then(|| c + 1),
        }
    }

    fn walled(&self, c: usize, d: Dir) -> bool {
        match d {
            Dir::E => self.east[c],
            Dir::S => self.south[c],
            Dir::W => self.east[c - 1],
            Dir::N => self.south[c - N],
        }
    }

    /// The neighbour across an open side.
    fn open(&self, c: usize, d: Dir) -> Option<usize> {
        Board::across(c, d).filter(|_| !self.walled(c, d))
    }

    pub fn can_wall(&self, c: usize, d: Dir) -> bool {
        self.open(c, d).is_some()
    }

    fn build(&mut self, c: usize, d: Dir) {
        match d {
            Dir::E => self.east[c] = true,
            Dir::S => self.south[c] = true,
            Dir::W => self.east[c - 1] = true,
            Dir::N => self.south[c - N] = true,
        }
    }

    fn occupied(&self, c: usize) -> bool {
        self.pieces.iter().any(|p| p.contains(&c))
    }

    /// Squares the piece on `from` can end on (its own square included).
    pub fn reach(&self, from: usize) -> Vec<usize> {
        let mut out = vec![from];
        let mut frontier = vec![from];
        for _ in 0..STEPS {
            let mut next = Vec::new();
            for &c in &frontier {
                for d in DIRS {
                    if let Some(n) = self.open(c, d)
                        && !self.occupied(n)
                        && !out.contains(&n)
                    {
                        out.push(n);
                        next.push(n);
                    }
                }
            }
            frontier = next;
        }
        out
    }

    /// Region label of every square, and the number of regions.
    fn regions(&self) -> ([u8; CELLS], usize) {
        let mut label = [u8::MAX; CELLS];
        let mut count = 0;
        let mut stack = Vec::with_capacity(CELLS);
        for start in 0..CELLS {
            if label[start] != u8::MAX {
                continue;
            }
            label[start] = count as u8;
            stack.push(start);
            while let Some(c) = stack.pop() {
                for d in DIRS {
                    if let Some(n) = self.open(c, d)
                        && label[n] == u8::MAX
                    {
                        label[n] = count as u8;
                        stack.push(n);
                    }
                }
            }
            count += 1;
        }
        (label, count)
    }

    /// For each square: the only seat with pieces in its region, if exactly one has.
    pub fn owners(&self) -> [Option<usize>; CELLS] {
        let (label, count) = self.regions();
        // Bit mask of the seats present in each region.
        let mut seats = vec![0u32; count];
        for (s, ps) in self.pieces.iter().enumerate() {
            for &c in ps {
                seats[label[c] as usize] |= 1 << s;
            }
        }
        std::array::from_fn(|c| {
            let m = seats[label[c] as usize];
            (m.count_ones() == 1).then(|| m.trailing_zeros() as usize)
        })
    }

    /// Secured area of each seat.
    pub fn areas(&self) -> Vec<usize> {
        let mut out = vec![0; self.pieces.len()];
        for s in self.owners().into_iter().flatten() {
            out[s] += 1;
        }
        out
    }

    /// Whether each seat is sealed in: all its pieces in regions without other seats.
    pub fn sealed(&self) -> Vec<bool> {
        let owners = self.owners();
        self.pieces.iter().map(|ps| ps.iter().all(|&c| owners[c].is_some())).collect()
    }

    /// Open neighbours of every square (`NONE` past the last), for the bot's fast floods.
    fn links(&self) -> [[u8; 4]; CELLS] {
        let mut out = [[NONE; 4]; CELLS];
        for (c, row) in out.iter_mut().enumerate() {
            let mut k = 0;
            for d in DIRS {
                if let Some(n) = self.open(c, d) {
                    row[k] = n as u8;
                    k += 1;
                }
            }
        }
        out
    }

    /// Territory estimate per seat: squares whose nearest piece (walking round walls) is
    /// strictly that seat's.
    #[cfg(test)]
    fn nearest(&self) -> Vec<i64> {
        self.estimate().0[..self.pieces.len()].to_vec()
    }

    /// For the bot, in one pass: the nearest-piece estimate and the secured area of each seat.
    fn estimate(&self) -> ([i64; 3], [i64; 3]) {
        let links = self.links();
        let mut dist = [u8::MAX; CELLS];
        let mut who = [u8::MAX; CELLS]; // seat, or TIE
        let mut queue = [0u8; CELLS];
        let (mut head, mut tail) = (0, 0);
        for (s, ps) in self.pieces.iter().enumerate() {
            for &c in ps {
                dist[c] = 0;
                who[c] = s as u8;
                queue[tail] = c as u8;
                tail += 1;
            }
        }
        while head < tail {
            let c = queue[head] as usize;
            head += 1;
            for &n in &links[c] {
                if n == NONE {
                    break;
                }
                let n = n as usize;
                if dist[n] == u8::MAX {
                    dist[n] = dist[c] + 1;
                    who[n] = who[c];
                    queue[tail] = n as u8;
                    tail += 1;
                } else if dist[n] == dist[c] + 1 && who[n] != who[c] {
                    who[n] = TIE;
                }
            }
        }
        let mut near = [0; 3];
        for w in who {
            if w < 3 {
                near[w as usize] += 1;
            }
        }
        // Regions: flood from every unlabelled square, noting which seats are inside.
        let mut seat_at = [0u8; CELLS];
        for (s, ps) in self.pieces.iter().enumerate() {
            for &c in ps {
                seat_at[c] |= 1 << s;
            }
        }
        let mut seen = [false; CELLS];
        let mut areas = [0; 3];
        for start in 0..CELLS {
            if seen[start] {
                continue;
            }
            seen[start] = true;
            let (mut head, mut tail) = (0, 1);
            queue[0] = start as u8;
            let mut mask = 0u8;
            while head < tail {
                let c = queue[head] as usize;
                head += 1;
                mask |= seat_at[c];
                for &n in &links[c] {
                    if n == NONE {
                        break;
                    }
                    if !seen[n as usize] {
                        seen[n as usize] = true;
                        queue[tail] = n;
                        tail += 1;
                    }
                }
            }
            if mask.count_ones() == 1 {
                areas[mask.trailing_zeros() as usize] += tail as i64;
            }
        }
        (near, areas)
    }
}

const NONE: u8 = u8::MAX;
const TIE: u8 = 254;

/// One whole turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Turn {
    pub piece: usize,
    pub to: usize,
    pub wall: Dir,
}

pub fn turns(b: &Board, seat: usize) -> Vec<Turn> {
    let mut out = Vec::new();
    for piece in 0..2 {
        for to in b.reach(b.pieces[seat][piece]) {
            out.extend(DIRS.into_iter().filter(|&d| b.can_wall(to, d)).map(|wall| Turn { piece, to, wall }));
        }
    }
    out
}

fn play(b: &mut Board, seat: usize, t: Turn) {
    b.pieces[seat][t.piece] = t.to;
    b.build(t.to, t.wall);
}

/// The bot's turn: the best territory estimate for `seat` against the strongest rival.
pub fn best_turn(b: &Board, seat: usize, rng: &mut Rng) -> Option<Turn> {
    let mut options = turns(b, seat);
    rng.shuffle(&mut options);
    let mut best: Option<(Turn, i64)> = None;
    let mut next = b.clone();
    for t in options {
        let old = next.pieces[seat][t.piece];
        play(&mut next, seat, t);
        let (near, areas) = next.estimate();
        let value = |s: usize| 2 * near[s] + areas[s];
        let rival = (0..b.pieces.len()).filter(|&s| s != seat).map(value).max().unwrap_or(0);
        let v = value(seat) - rival;
        if best.is_none_or(|(_, bv)| v > bv) {
            best = Some((t, v));
        }
        // Undo.
        next.pieces[seat][t.piece] = old;
        next.east = b.east;
        next.south = b.south;
    }
    best.map(|(t, _)| t)
}

#[derive(Clone, Debug)]
pub struct WallState {
    players: Vec<String>,
    board: Board,
    turn: usize,
    sealed: Vec<bool>,
    last: Option<(usize, Turn)>,
    turn_no: u32,
    over: bool,
    /// Replay moments: one wall sealing off 15 squares or more; a rival shut in with 4 or fewer.
    pub moments: Moments,
}

/// A wall that adds at least this much secured area in one turn is a replay moment.
const BIG_SEAL: usize = 15;
/// A rival sealed in with at most this much area is a replay moment.
const SMALL_POCKET: usize = 4;

impl WallState {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }
    fn end_turn(&mut self) {
        self.sealed = self.board.sealed();
        self.turn_no += 1;
        if self.sealed.iter().all(|&s| s) {
            self.over = true;
            return;
        }
        let n = self.players.len();
        for k in 1..=n {
            let next = (self.turn + k) % n;
            if !self.sealed[next] {
                self.turn = next;
                return;
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WallMove {
    Turn { piece: usize, x: usize, y: usize, wall: Dir },
    Pass,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceView {
    pub seat: usize,
    pub index: usize,
    pub x: usize,
    pub y: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatView {
    pub id: String,
    pub area: usize,
    pub sealed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WallView {
    pub size: usize,
    pub me: usize,
    pub seats: Vec<SeatView>,
    pub pieces: Vec<PieceView>,
    pub east: Vec<bool>,
    pub south: Vec<bool>,
    /// The seat whose sealed region each square is in, or −1.
    pub owner: Vec<i32>,
    pub turn: usize,
    pub your_turn: bool,
    /// Squares (y × size + x) each of your pieces can end on, on your turn.
    pub reach: Vec<Vec<usize>>,
    /// Last turn: seat, the square the piece ended on, the wall side.
    pub last: Option<(usize, usize, Dir)>,
    pub turn_no: u32,
    pub over: bool,
}

pub struct WallTerritory;

impl Game for WallTerritory {
    type State = WallState;
    type Move = WallMove;
    type View = WallView;

    fn info() -> GameInfo {
        GameInfo {
            id: "wall-territory",
            kind: "prison",
            difficulty: "hard",
            tags: &["strategy-territory"],
            min_players: 2,
            max_players: 3,
            name: "Wall Go",
            short: "Move a piece, build a wall: fence off the biggest area of the 7×7 board for yourself.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<WallState, String> {
        if !(2..=3).contains(&players.len()) {
            return Err("Wall Go is for 2 or 3 players".into());
        }
        let mut sets: Vec<usize> = (0..players.len()).collect();
        Rng::new(seed).shuffle(&mut sets);
        let pieces = sets.iter().map(|&k| STARTS[k].map(|(x, y)| y * N + x)).collect();
        let board = Board { east: [false; CELLS], south: [false; CELLS], pieces };
        Ok(WallState {
            players: players.to_vec(),
            sealed: vec![false; players.len()],
            board,
            turn: 0,
            last: None,
            turn_no: 1,
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut WallState, player: &str, mv: WallMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.seat(player).ok_or("Not in this game")?;
        if seat != s.turn {
            return Err("Not your turn".into());
        }
        match mv {
            WallMove::Pass => {
                if !turns(&s.board, seat).is_empty() {
                    return Err("You can still move".into());
                }
            }
            WallMove::Turn { piece, x, y, wall } => {
                if piece > 1 || x >= N || y >= N {
                    return Err("No such piece or square".into());
                }
                let to = y * N + x;
                if !s.board.reach(s.board.pieces[seat][piece]).contains(&to) {
                    return Err("That piece cannot get there".into());
                }
                if !s.board.can_wall(to, wall) {
                    return Err("No wall can go on that side".into());
                }
                let t = Turn { piece, to, wall };
                let before = s.board.areas();
                play(&mut s.board, seat, t);
                s.last = Some((seat, t));
                // Replay moments (the board is public): a big area sealed off for the mover, or a
                // rival shut in with a small pocket.
                let after = s.board.areas();
                let sealed = s.board.sealed();
                if after[seat] >= before[seat] + BIG_SEAL {
                    s.moments.push(&s.players[seat].clone(), format!("Wall seals off {} squares", after[seat] - before[seat]));
                }
                for k in (0..s.players.len()).filter(|&k| k != seat && !s.sealed[k] && sealed[k] && after[k] <= SMALL_POCKET) {
                    let squares = if after[k] == 1 { "1 square".to_string() } else { format!("{} squares", after[k]) };
                    s.moments.push(&s.players[seat].clone(), format!("{{{}}} shut in with only {squares}", s.players[k]));
                }
            }
        }
        s.end_turn();
        Ok(())
    }

    fn view_for(s: &WallState, player: &str) -> WallView {
        let me = s.seat(player).unwrap_or(0);
        let areas = s.board.areas();
        let your_turn = !s.over && s.turn == me;
        WallView {
            size: N,
            me,
            seats: s
                .players
                .iter()
                .enumerate()
                .map(|(i, id)| SeatView { id: id.clone(), area: areas[i], sealed: s.sealed[i] })
                .collect(),
            pieces: s
                .board
                .pieces
                .iter()
                .enumerate()
                .flat_map(|(seat, ps)| ps.iter().enumerate().map(move |(index, &c)| PieceView { seat, index, x: c % N, y: c / N }))
                .collect(),
            east: s.board.east.to_vec(),
            south: s.board.south.to_vec(),
            owner: s.board.owners().iter().map(|o| o.map_or(-1, |s| s as i32)).collect(),
            turn: s.turn,
            your_turn,
            reach: if your_turn { s.board.pieces[me].iter().map(|&c| s.board.reach(c)).collect() } else { Vec::new() },
            last: s.last.map(|(seat, t)| (seat, t.to, t.wall)),
            turn_no: s.turn_no,
            over: s.over,
        }
    }

    fn is_over(s: &WallState) -> bool {
        s.over
    }

    fn results(s: &WallState) -> Vec<PlayerResult> {
        let areas = s.board.areas();
        let scores: Vec<(String, i64)> = s.players.iter().zip(areas).map(|(p, a)| (p.clone(), 100 + 20 * a as i64)).collect();
        rank_by_score(&scores)
    }

    fn to_move(s: &WallState) -> Vec<String> {
        if s.over { Vec::new() } else { vec![s.players[s.turn].clone()] }
    }

    fn bot_move(s: &WallState, player: &str, seed: u64) -> WallMove {
        let seat = s.seat(player).unwrap_or(0);
        match best_turn(&s.board, seat, &mut Rng::new(seed)) {
            Some(t) => WallMove::Turn { piece: t.piece, x: t.to % N, y: t.to / N, wall: t.wall },
            None => WallMove::Pass,
        }
    }

    fn moments(s: &WallState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub type Rules = WallTerritory;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn two() -> WallState {
        let mut s = WallTerritory::setup(&["a".into(), "b".into()], 1).unwrap();
        s.board.pieces = vec![[8, 40], [12, 36]];
        s
    }

    fn turn(piece: usize, x: usize, y: usize, wall: Dir) -> WallMove {
        WallMove::Turn { piece, x, y, wall }
    }

    #[test]
    fn moves_go_up_to_two_squares_and_not_through_walls() {
        let mut s = two();
        // a's piece 0 at (1,1).
        let r = s.board.reach(8);
        assert_eq!(r.len(), 1 + 4 + 6);
        assert!(WallTerritory::apply_move(&mut s, "a", turn(0, 4, 1, Dir::N)).is_err());
        assert!(WallTerritory::apply_move(&mut s, "a", turn(0, 1, 0, Dir::N)).is_err(), "board edge");
        WallTerritory::apply_move(&mut s, "a", turn(0, 1, 1, Dir::E)).unwrap();
        assert!(s.board.east[8]);
        assert_eq!(s.turn, 1);
        assert!(!s.board.reach(8).contains(&9));
        assert!(!s.board.can_wall(9, Dir::W), "wall already there");
    }

    #[test]
    fn a_closed_box_seals_and_scores() {
        let mut b = Board { east: [false; CELLS], south: [false; CELLS], pieces: vec![[0, 1], [48, 47]] };
        // Wall off the top-left 2×2 square: east of (1,0),(1,1), south of (0,1),(1,1).
        b.east[1] = true;
        b.east[8] = true;
        b.south[7] = true;
        b.south[8] = true;
        assert_eq!(b.sealed(), vec![true, true]);
        assert_eq!(b.areas(), vec![4, 45]);
    }

    #[test]
    fn sealed_players_are_skipped_and_the_end_comes() {
        let mut s = WallTerritory::setup(&["a".into(), "b".into(), "c".into()], 1).unwrap();
        s.board.pieces = vec![[0, 1], [48, 47], [24, 25]];
        for c in [1, 8] {
            s.board.east[c] = true;
        }
        for c in [7, 8] {
            s.board.south[c] = true;
        }
        s.turn = 2;
        s.end_turn();
        assert!(s.sealed[0] && !s.sealed[1]);
        assert_eq!(s.turn, 1);
        s.end_turn();
        assert_eq!(s.turn, 2);
    }

    #[test]
    fn the_estimate_splits_the_board() {
        let b = Board { east: [false; CELLS], south: [false; CELLS], pieces: vec![[0, 1], [48, 47]] };
        let near = b.nearest();
        assert_eq!(near[0], near[1]);
        assert!(near[0] > 15);
    }

    #[test]
    fn the_bot_closes_the_gap_to_its_side() {
        // A wall between columns 2 and 3 with one gap in row 3; a's pieces are on the left.
        let mut b = Board { east: [false; CELLS], south: [false; CELLS], pieces: vec![[3 * N + 1, 0], [6, 48]] };
        for y in (0..N).filter(|&y| y != 3) {
            b.east[y * N + 2] = true;
        }
        let t = best_turn(&b, 0, &mut Rng::new(1)).unwrap();
        play(&mut b, 0, t);
        // It closes the gap; the piece that went through keeps fighting for the right side.
        assert_eq!(b.areas()[0], 21, "{t:?}");
    }

    #[test]
    fn the_turn_waits_for_the_mover() {
        let mut s = two();
        WallTerritory::tick(&mut s, 600_000);
        assert_eq!((s.turn, s.turn_no), (0, 1));
        assert!(s.last.is_none());
    }

    // 300 games each instead of 1,000: the bot tries ~100 turns with two floods each, which
    // is slow in the unoptimised test build (1,000 two-player games took about 20 s).
    #[test]
    fn bot_games_for_two() {
        bot_games_with::<WallTerritory>(300, 2, 1000);
    }

    #[test]
    fn bot_games_for_three() {
        bot_games_with::<WallTerritory>(300, 3, 1500);
    }

    #[test]
    fn a_big_seal_is_a_replay_moment() {
        // A wall between columns 2 and 3 with one gap in row 3; a's pieces are on the left.
        let mut s = two();
        s.board.pieces = vec![[3 * N + 1, 0], [6, 48]];
        for y in (0..N).filter(|&y| y != 3) {
            s.board.east[y * N + 2] = true;
        }
        WallTerritory::apply_move(&mut s, "a", turn(0, 2, 3, Dir::E)).unwrap();
        let m = WallTerritory::moments(&s, "b");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("a", "Wall seals off 21 squares"));
    }

    #[test]
    fn shutting_a_rival_in_a_small_pocket_is_a_replay_moment() {
        let ids: Vec<String> = (1..=3).map(|i| format!("p{i}")).collect();
        let mut s = WallTerritory::setup(&ids, 1).unwrap();
        // p3 sits in the top-left corner, walled on two sides; p1 closes the third.
        s.board.pieces = vec![[2 * N + 1, 30], [48, 47], [0, 1]];
        s.board.east[1] = true;
        s.board.south[0] = true;
        s.turn = 0;
        WallTerritory::apply_move(&mut s, "p1", turn(0, 1, 1, Dir::N)).unwrap();
        let m = WallTerritory::moments(&s, "p2");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "{p3} shut in with only 2 squares"));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<WallTerritory>(300, 3, 1_500);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}
