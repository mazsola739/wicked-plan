//! Fence Building (exile, a race for 1–4). Rules page: docs/games/fence-building.md
//!
//! One closed fence along the grid lines; a number in a plot is how many of
//! its sides are fenced. The fence is the border of a fenced-in area, so the
//! generator and the solver work on plots (inside / outside), not on edges.
//! Seat 0 solves; the other seats are bots that finish at seeded times.

use serde::{Deserialize, Serialize};

use super::piece_puzzle::{Race, RaceView};
use crate::game::{Game, GameInfo, PlayerResult};
use crate::rng::Rng;

pub const SIZES: [usize; 3] = [5, 6, 7];
/// Share of plots that keep their number before the uniqueness pass adds some back (percent).
pub const SHOWN: [usize; 3] = [45, 38, 34];
/// Work limit of one solver run while making a field.
const SOLVER_BUDGET: u64 = 200_000;
pub const ROUND_MS: [i64; 3] = [2 * 60_000, 3 * 60_000, 5 * 60_000];

/// Number of edges: horizontal ones first ((n+1)·n), then vertical ones (n·(n+1)).
pub fn edge_count(n: usize) -> usize {
    2 * n * (n + 1)
}

/// The four sides of a plot: top, bottom, left, right.
pub fn sides(n: usize, cell: usize) -> [usize; 4] {
    let (r, c) = (cell / n, cell % n);
    let h = (n + 1) * n;
    [r * n + c, (r + 1) * n + c, h + r * (n + 1) + c, h + r * (n + 1) + c + 1]
}

/// The two posts (vertices, (n+1)² of them) at the ends of an edge.
fn posts(n: usize, edge: usize) -> (usize, usize) {
    let h = (n + 1) * n;
    if edge < h {
        let (r, c) = (edge / n, edge % n);
        (r * (n + 1) + c, r * (n + 1) + c + 1)
    } else {
        let (r, c) = ((edge - h) / (n + 1), (edge - h) % (n + 1));
        (r * (n + 1) + c, (r + 1) * (n + 1) + c)
    }
}

fn neighbours(n: usize, cell: usize) -> [Option<usize>; 4] {
    let (r, c) = (cell / n, cell % n);
    [
        (r > 0).then(|| cell - n),
        (r + 1 < n).then(|| cell + n),
        (c > 0).then(|| cell - 1),
        (c + 1 < n).then(|| cell + 1),
    ]
}

/// The fence around an area: an edge is fenced when the plots on its two sides differ.
pub fn fence_of(n: usize, inside: &[bool]) -> Vec<bool> {
    let mut edges = vec![false; edge_count(n)];
    for cell in (0..n * n).filter(|&c| inside[c]) {
        for e in sides(n, cell) {
            edges[e] = !edges[e];
        }
    }
    edges
}

/// Clue of every plot: how many of its sides are fenced.
pub fn clues_of(n: usize, inside: &[bool]) -> Vec<u8> {
    let fence = fence_of(n, inside);
    (0..n * n).map(|c| sides(n, c).iter().filter(|&&e| fence[e]).count() as u8).collect()
}

/// A valid area: connected, no holes (every outside plot reaches the border),
/// and no two plots touching only at a corner in a checkerboard (the fence would touch itself).
pub fn is_valid_area(n: usize, inside: &[bool]) -> bool {
    let Some(start) = inside.iter().position(|&x| x) else { return false };
    let reach = |from: Vec<usize>, want: bool| {
        let mut seen = vec![false; n * n];
        let mut stack = from;
        for &s in &stack {
            seen[s] = true;
        }
        while let Some(c) = stack.pop() {
            for nb in neighbours(n, c).into_iter().flatten() {
                if inside[nb] == want && !seen[nb] {
                    seen[nb] = true;
                    stack.push(nb);
                }
            }
        }
        seen
    };
    let ins = reach(vec![start], true);
    let border: Vec<usize> = (0..n * n).filter(|&c| !inside[c] && (c < n || c >= n * n - n || c % n == 0 || c % n == n - 1)).collect();
    let outs = reach(border, false);
    let checker = (0..n - 1).any(|r| {
        (0..n - 1).any(|c| {
            let (a, b, d, e) = (inside[r * n + c], inside[r * n + c + 1], inside[r * n + n + c], inside[r * n + n + c + 1]);
            a == e && b == d && a != b
        })
    });
    (0..n * n).all(|c| if inside[c] { ins[c] } else { outs[c] }) && !checker
}

/// Plot values while solving: -1 open, 0 outside, 1 inside.
type Cells = Vec<i8>;

/// Fills in what the numbers and the "no touching corners" rule force.
/// False on a contradiction.
fn propagate(n: usize, clues: &[Option<u8>], cells: &mut Cells) -> bool {
    loop {
        let mut changed = false;
        for x in 0..n * n {
            let Some(k) = clues[x] else { continue };
            let nbs = neighbours(n, x);
            let vals = nbs.map(|o| o.map_or(0, |c| cells[c]));
            let v = cells[x];
            let open = vals.iter().filter(|&&w| w < 0).count() as u8;
            if v >= 0 {
                let diff = vals.iter().filter(|&&w| w >= 0 && w != v).count() as u8;
                if diff > k || diff + open < k {
                    return false;
                }
                if open > 0 && (diff == k || diff + open == k) {
                    let fill = if diff == k { v } else { 1 - v };
                    for o in nbs.into_iter().flatten() {
                        if cells[o] < 0 {
                            cells[o] = fill;
                            changed = true;
                        }
                    }
                }
            } else if open == 0 {
                let ones = vals.iter().filter(|&&w| w == 1).count() as u8;
                match (ones == k, 4 - ones == k) {
                    (false, false) => return false,
                    (true, false) => cells[x] = 0,
                    (false, true) => cells[x] = 1,
                    (true, true) => continue,
                }
                changed = true;
            }
        }
        // Corners: a b / d e must never be a checkerboard (a = e, b = d, a ≠ b).
        for r in 0..n - 1 {
            for c in 0..n - 1 {
                let idx = [r * n + c, r * n + c + 1, (r + 1) * n + c, (r + 1) * n + c + 1];
                let v = idx.map(|i| cells[i]);
                // (open, its diagonal partner, the other diagonal)
                for (p, q, s, t) in [(0, 3, 1, 2), (3, 0, 1, 2), (1, 2, 0, 3), (2, 1, 0, 3)] {
                    if v[p] < 0 && v[q] >= 0 && v[s] >= 0 && v[s] == v[t] && v[q] != v[s] {
                        cells[idx[p]] = v[s];
                        changed = true;
                    }
                }
                if v.iter().all(|&w| w >= 0) && v[0] == v[3] && v[1] == v[2] && v[0] != v[1] {
                    return false;
                }
            }
        }
        if !changed {
            return true;
        }
    }
}

/// Rows 0..=r are filled. Err when they are already wrong (a hole, or an
/// inside part cut off next to another); Ok(true) when the area is closed
/// off, so every later plot must be outside.
fn rows_done(n: usize, cells: &Cells, r: usize) -> Result<bool, ()> {
    let upto = (r + 1) * n;
    let mut seen = vec![false; upto];
    let (mut closed_inside, mut inside_parts) = (false, 0);
    for start in 0..upto {
        if seen[start] {
            continue;
        }
        let v = cells[start];
        seen[start] = true;
        let mut stack = vec![start];
        let (mut open, mut border) = (false, false);
        while let Some(x) = stack.pop() {
            open |= x / n == r;
            border |= x < n || x % n == 0 || x % n == n - 1;
            for nb in neighbours(n, x).into_iter().flatten() {
                if nb < upto && !seen[nb] && cells[nb] == v {
                    seen[nb] = true;
                    stack.push(nb);
                }
            }
        }
        if v == 1 {
            inside_parts += 1;
            closed_inside |= !open;
        } else if !open && !border {
            return Err(());
        }
    }
    if closed_inside && inside_parts > 1 {
        return Err(());
    }
    Ok(closed_inside)
}

/// The areas that fit the clues, up to `limit` of them. None when the work
/// limit ran out first.
pub fn solve(n: usize, clues: &[Option<u8>], limit: usize, budget: &mut u64) -> Option<Vec<Vec<bool>>> {
    struct S<'a> {
        n: usize,
        clues: &'a [Option<u8>],
        found: Vec<Vec<bool>>,
        limit: usize,
        budget: &'a mut u64,
    }
    impl S<'_> {
        /// False when the search must stop (enough found or out of work).
        fn go(&mut self, mut cells: Cells) -> bool {
            if *self.budget == 0 {
                return false;
            }
            *self.budget -= 1;
            let n = self.n;
            if !propagate(n, self.clues, &mut cells) {
                return true;
            }
            let mut next = cells.iter().position(|&v| v < 0).unwrap_or(n * n);
            if next >= n {
                match rows_done(n, &cells, next / n - 1) {
                    Err(()) => return true,
                    Ok(true) => {
                        // Closed off: the rest is outside.
                        let from = next / n * n;
                        if cells[from..].contains(&1) {
                            return true;
                        }
                        cells[from..].iter_mut().for_each(|v| *v = 0);
                        if !propagate(n, self.clues, &mut cells) {
                            return true;
                        }
                        next = n * n;
                    }
                    Ok(false) => {}
                }
            }
            if next == n * n {
                let inside: Vec<bool> = cells.iter().map(|&v| v == 1).collect();
                if is_valid_area(n, &inside) {
                    self.found.push(inside);
                }
                return self.found.len() < self.limit;
            }
            for v in [0, 1] {
                let mut branch = cells.clone();
                branch[next] = v;
                if !self.go(branch) {
                    return false;
                }
            }
            true
        }
    }
    let mut s = S { n, clues, found: Vec::new(), limit, budget };
    let finished = s.go(vec![-1; n * n]) || s.found.len() >= limit;
    finished.then_some(s.found)
}

#[derive(Clone, Debug)]
pub struct Field {
    pub size: usize,
    pub inside: Vec<bool>,
    pub clues: Vec<Option<u8>>,
}

pub fn generate(round: usize, rng: &mut Rng) -> Field {
    let n = SIZES[round];
    loop {
        // Grow a wiggly area: of a few candidate plots, add the one with the fewest inside neighbours.
        let goal = n * n * rng.int(45, 60) as usize / 100;
        let mut inside = vec![false; n * n];
        inside[rng.index(n * n)] = true;
        let mut size = 1;
        let mut misses = 0;
        while size < goal && misses < 200 {
            let frontier: Vec<usize> =
                (0..n * n).filter(|&c| !inside[c] && neighbours(n, c).into_iter().flatten().any(|nb| inside[nb])).collect();
            let inside_nbs = |c: usize| neighbours(n, c).into_iter().flatten().filter(|&nb| inside[nb]).count();
            let cell = (0..3).map(|_| *rng.pick(&frontier)).min_by_key(|&c| inside_nbs(c)).expect("three picks");
            inside[cell] = true;
            if is_valid_area(n, &inside) {
                size += 1;
                misses = 0;
            } else {
                inside[cell] = false;
                misses += 1;
            }
        }
        if size * 100 < n * n * 35 {
            continue;
        }
        // Hide numbers at random down to the base share, then show numbers again
        // until the solver finds a single fence: each time one where two fences differ.
        let all = clues_of(n, &inside);
        let mut clues: Vec<Option<u8>> = all.iter().map(|&k| Some(k)).collect();
        let mut order: Vec<usize> = (0..n * n).collect();
        rng.shuffle(&mut order);
        for &cell in &order[n * n * SHOWN[round] / 100..] {
            clues[cell] = None;
        }
        loop {
            let mut budget = SOLVER_BUDGET;
            let found = solve(n, &clues, 2, &mut budget);
            let hidden: Vec<usize> = (0..n * n).filter(|&c| clues[c].is_none()).collect();
            let telling: Vec<usize> = match &found {
                Some(f) if f.len() == 1 => break,
                Some(f) if f.len() == 2 => {
                    let other = clues_of(n, if f[0] == inside { &f[1] } else { &f[0] });
                    hidden.iter().copied().filter(|&c| all[c] != other[c]).collect()
                }
                _ => Vec::new(),
            };
            let pool = if telling.is_empty() { &hidden } else { &telling };
            let cell = *rng.pick(pool);
            clues[cell] = Some(all[cell]);
        }
        return Field { size: n, inside, clues };
    }
}

/// One row-major letter per plot: upper case inside, lower case outside;
/// 'a'…'e' a shown number 0…4, 'x' a hidden one.
pub fn encode(f: &Field) -> String {
    (0..f.size * f.size)
        .map(|c| {
            let ch = f.clues[c].map_or('x', |k| (b'a' + k) as char);
            if f.inside[c] { ch.to_ascii_uppercase() } else { ch }
        })
        .collect()
}

pub fn decode(n: usize, s: &str) -> Field {
    let bytes = s.as_bytes();
    Field {
        size: n,
        inside: bytes.iter().map(|b| b.is_ascii_uppercase()).collect(),
        clues: bytes.iter().map(|b| b.to_ascii_lowercase()).map(|b| (b != b'x').then(|| b - b'a')).collect(),
    }
}

/// The field turned `t % 4` quarter turns, then mirrored when `t >= 4`.
/// Numbers belong to their plot, so every turn or mirror is a valid field too.
pub fn transform(f: &Field, t: usize) -> Field {
    let n = f.size;
    let source = |cell: usize| {
        let (mut r, mut c) = (cell / n, cell % n);
        if t >= 4 {
            c = n - 1 - c;
        }
        for _ in 0..t % 4 {
            (r, c) = (n - 1 - c, r);
        }
        r * n + c
    };
    Field {
        size: n,
        inside: (0..n * n).map(|c| f.inside[source(c)]).collect(),
        clues: (0..n * n).map(|c| f.clues[source(c)]).collect(),
    }
}

/// A field for the round: one from the bank (made by `generate`, see the
/// `print_bank` test), turned or mirrored. The bank keeps the game and its
/// tests fast: proving a single fence takes the solver a while.
pub fn pick(round: usize, rng: &mut Rng) -> Field {
    let bank = BANK[round];
    transform(&decode(SIZES[round], bank[rng.index(bank.len())]), rng.index(8))
}

/// Is this set of fenced edges one closed loop that fits every number?
pub fn is_solved(n: usize, clues: &[Option<u8>], fenced: &[bool]) -> bool {
    let clues_fit = (0..n * n).all(|c| clues[c].is_none_or(|k| sides(n, c).iter().filter(|&&e| fenced[e]).count() == k as usize));
    let mut degree = vec![0; (n + 1) * (n + 1)];
    let lines: Vec<usize> = (0..fenced.len()).filter(|&e| fenced[e]).collect();
    for &e in &lines {
        let (a, b) = posts(n, e);
        degree[a] += 1;
        degree[b] += 1;
    }
    if !clues_fit || lines.is_empty() || degree.iter().any(|&d| d != 0 && d != 2) {
        return false;
    }
    // Walk the loop from the first edge; it must use every fenced edge.
    let (start, mut at) = posts(n, lines[0]);
    let (mut prev, mut walked) = (lines[0], 1);
    while at != start {
        let Some(&next) = lines.iter().find(|&&e| e != prev && (posts(n, e).0 == at || posts(n, e).1 == at)) else { return false };
        let (a, b) = posts(n, next);
        at = if a == at { b } else { a };
        prev = next;
        walked += 1;
    }
    walked == lines.len()
}

#[derive(Clone, Debug)]
pub struct FenceState {
    pub race: Race,
    pub field: Field,
    /// 0 empty, 1 fence, 2 cross.
    pub edges: Vec<u8>,
    seed: u64,
}

impl FenceState {
    fn load(&mut self) {
        if !self.race.over {
            self.field = pick(self.race.round, &mut Rng::new(self.seed.wrapping_add(self.race.round as u64 * 1_000_003)));
            self.edges = vec![0; edge_count(self.field.size)];
        }
    }

    fn solved(&self) -> bool {
        let fenced: Vec<bool> = self.edges.iter().map(|&m| m == 1).collect();
        is_solved(self.field.size, &self.field.clues, &fenced)
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FenceMove {
    Edge { edge: usize, mark: u8 },
    Clear,
    GiveUp,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FenceView {
    #[serde(flatten)]
    pub race: RaceView,
    pub size: usize,
    pub clues: Vec<Option<u8>>,
    pub edges: Vec<u8>,
    pub over: bool,
}

pub struct Rules;

impl Game for Rules {
    type State = FenceState;
    type Move = FenceMove;
    type View = FenceView;

    fn info() -> GameInfo {
        GameInfo {
            id: "fence-building",
            kind: "exile",
            min_players: 1,
            max_players: 4,
            name: "Fence Building",
            short: "Build one closed fence; each number says how many sides of its plot are fenced. A race.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<FenceState, String> {
        let race = Race::new(players, seed, &ROUND_MS)?;
        let mut s = FenceState { race, field: Field { size: 0, inside: Vec::new(), clues: Vec::new() }, edges: Vec::new(), seed };
        s.load();
        Ok(s)
    }

    fn apply_move(s: &mut FenceState, player: &str, mv: FenceMove) -> Result<(), String> {
        s.race.check_move(player)?;
        match mv {
            FenceMove::Edge { edge, mark } => {
                if edge >= s.edges.len() || mark > 2 {
                    return Err("Illegal move".into());
                }
                s.edges[edge] = mark;
                if s.solved() {
                    s.race.solver_done(true);
                    s.load();
                }
            }
            FenceMove::Clear => s.edges.iter_mut().for_each(|m| *m = 0),
            FenceMove::GiveUp => {
                s.race.solver_done(false);
                s.load();
            }
        }
        Ok(())
    }

    fn tick(s: &mut FenceState, ms: u64) {
        if s.race.tick(ms) {
            s.load();
        }
    }

    fn view_for(s: &FenceState, _player: &str) -> FenceView {
        FenceView {
            race: s.race.view(),
            size: s.field.size,
            clues: s.field.clues.clone(),
            edges: s.edges.clone(),
            over: s.race.over,
        }
    }

    fn is_over(s: &FenceState) -> bool {
        s.race.over
    }

    fn results(s: &FenceState) -> Vec<PlayerResult> {
        s.race.results()
    }

    fn to_move(s: &FenceState) -> Vec<String> {
        s.race.to_move()
    }

    fn bot_move(s: &FenceState, _player: &str, seed: u64) -> FenceMove {
        let mut rng = Rng::new(seed);
        let fence = fence_of(s.field.size, &s.field.inside);
        let wrong: Vec<usize> = (0..s.edges.len()).filter(|&e| (s.edges[e] == 1) != fence[e]).collect();
        if wrong.is_empty() {
            return FenceMove::GiveUp;
        }
        let edge = *rng.pick(&wrong);
        if rng.next_f64() < 0.1 {
            // A slip: a fence on a random edge.
            return FenceMove::Edge { edge: rng.index(s.edges.len()), mark: 1 };
        }
        FenceMove::Edge { edge, mark: u8::from(fence[edge]) }
    }
}

pub const READY: bool = true;

/// The field bank, printed by the `print_bank` test.
pub const BANK: [&[&str]; 3] = [
    &[
        "XCCCcXxdBXXxDBdbxxCXxaxxb",
        "xxbbbabXXXbbcxXCXXXBCXXdX",
        "XxCCXBXBxXDdCcXbxCcDxxDxb",
        "bbxbxXXdXxxXXBcCCxBDDcbXx",
        "xXCBDCXdXxXdxCxXDcCbXcxXx",
        "DxDxDXXXBCXccDxXxxbxXbaxa",
        "xXxabDBbbXcCxxCxXXBCDCdXx",
        "abXxxacXdDxXXBXXBxDxxDxba",
        "xcDxaCBXDxBBXdxXxXCXbxXdD",
        "xbbDbxDxXcXXCBXcDcxXxbbXX",
        "XxCXbBBBdxXXXXbXxCBbbacXx",
        "axXxaxbCxxxxBDxDXBdxcCBCX",
        "XdXCcBXAXXCdXxXXbbabbxxaa",
        "aaxCXbxCBdXXXBXbxXdXxaxbX",
        "DxcXDBXBXcXdXcxDcXbaxxDxx",
        "DCXcxxdXCDDCXxxxxCxaxxDba",
        "abXcXaxCdXbDBXBxxbxCxxxDC",
        "CCcxDCAXXXcCxxCxDcCXxxbDc",
        "axaxxcDcDbCCcXxXxcXxCXXXD",
        "bXBCxxcBBxDdXCxBXXcxCXcaa",
        "XcbxxBXCXDCdcxbXBCxxxCBXD",
        "aaaaxbxxxDXCxXBXBXBXXdDxD",
        "cXbbaCXxDcCACXXxXxDcDXxbx",
        "cXdXxDAXBCcXBdXbXBbbaxXba",
        "xaxaxXxxbDCxbxXXCCXXDccCX",
        "XBBDbxXXcacXdxaCXBCbXdCXx",
        "xaaxaXbxaxXxbxDXxCBXXCBCx",
        "DdCXXXXXcbCxCcxCxXXbDbxxa",
        "xDBXxxcCxaabXxxbcXBDDXBXx",
        "xbDxaxcXcxxXXCCDXXdCcXxbD",
        "CXXDxCcxcxCdCDbXXXdxDxXDx",
        "CCXCXCdcxxBCXDxXxxbxDbxxa",
        "XxDxbCXXdXxdXCXbXBdCxcDxX",
        "axxbabbxXcCCcXXBXdXdXXXXX",
        "CCcaxCXCxbxXBXXCXdCXDcabx",
        "aacXbxcXXcDCABXxxXdXaxDxD",
        "XXCXCXdccDCCCbbbxXxxxxDxx",
        "CXXxaBBBXxCdxcaXCxDxxCXBD",
        "xcDxaXBBCxxDdXCabxdXaxbXX",
        "axXXxcXXcbXXcbDCdxcXCXXCC",
        "xbXXCxbcXBbXcdXxXXBBbDxXX",
        "bbXDxXxCxxXXBcaxXXCcDCdXX",
        "CBXXxXXdXbCxxDbBDxxxDxxax",
        "xxxCXxbDAXxbxXdcDxBCXBXXX",
        "xcCBDxCXXxCXXxbDdXXDxacXx",
        "DCCcbxxBXXXBXXcXXxbaXcaaa",
        "bxbDxDxxCxBBCXDCXxDcxDbxx",
        "xxbXCaxcXBxxCBBDBBxXxCXcX",
    ],
    &[
        "CXXCXXCxCdCBCxCbxXCxXbxbXxcxaxCCDbxx",
        "abxxxxxCXxbxcXdxDbCXXXXcCXBABCcXxXxX",
        "XXXCcxDxCXBDxadXXxxxXBxaabxCxxbXCXxa",
        "xbcXdXXXXBXXCAXxdCxCXxXBaxbaxDaxaxxb",
        "DxbxaaXBDxxxXXxxDxXxacXCBCxXBBXXXXxX",
        "xaxxbbbbcXBXXXBBXBbxXxxDxXBXcxDXdCXD",
        "XCccCXcCXCXxxbxdBCbcXBXCDCBXBcxxxxDx",
        "bxxxcDXCcXBCcCxdXdbXCXXCxbbbdCxxxbDX",
        "CXxxxxBXDxDxCdcxXxXBBXXCxXXxCCacXbxb",
        "xxbXCCxbxcxXDXxXXBxXCBdDacxXxxxDCBDx",
        "DxbCXbXCdXXbBXCXxxXxccaxXXCXxaCCdDxx",
        "axxbXXcXxxdXXXCdCBXdXBBBxaxXdXaxDXcX",
        "bxbXCCXCxccXXBxDdCXBXBXXbcCcxXaxXcXX",
        "xxxabXaaxacXbxxbXBDdXxxXXXXxCBCBXXXC",
        "XCCxaxxdXxbxXXXCCDccCcxxXxCxDxXCBCBD",
        "DXcaxxxXXxabXBBxxXxxXDcXXdCdcXCCXXCX",
        "DXBCCXxXCxcXXBxabCxCbxxXxXbaxbbDxxxx",
        "CXXXbxXdDxbaXcxxDxCXDcXxcXdxBDxXCXCx",
        "CXCCCCCBxcdXdXxXCBCXcCxCCCxDxDbxaxax",
        "XXBCXcDdXxCCbcXbdCxDXxCXaxbxXCxxabXc",
        "bxxbXbDcDcCcXxXcBXXdXxBXXXBBXBxDxXxD",
        "bDXXXCxxbxXBbDxbdXxXXBCXbDdXxXxbxDcD",
        "DXXXcaxXdXDxacxXxxxDxXcbDXXACCcDdXxX",
        "xbDbxxxcXxxxDXADcDxCBdcCxdXXXXbXBCxD",
        "aaxxxaxaaxxxXbxxxDXdCCxXBXXXBCDdCBXx",
        "XdDxXbXBXBXxcCXXADxxCdCdxbCcCXxbXxxb",
        "DXBCXxcXCdXbxCxaxaXBxaxxxCxbxxXBXXDb",
        "xcCXXcCCBxXDDdXxCxbxccBCbXXBACxxcCXx",
        "XBXXCcxDdXAXxcbdXxcCXXBbXXdBXxbbxCBX",
        "XdCXxbXXBxcXCxBCBCCcXdCxCcXcXxXcDxDb",
        "XCcxXXcCXXXcbdXdxaXBXXbxdXdXxxXCxDbx",
        "baxaxxXdDbaxCXBxxxcdBXxaCBXXXcXXdXXX",
        "xCCXXbbCdccxxXCXDxcCdxcbXXXXCDcDxDxx",
        "XCxCBDdXXBXxCBXdXXCcbxcDCxxxabXbaaxx",
        "axxxCDbXCXXdxbcdXXxbXCXxxabxBCxxxXBC",
        "abDbxXaxXxCCxXBXXcDBdXdbxXcBXDxbxXxb",
        "XbxXcxXxbXCbCCbdCbcCcXXcxCdBXCXBXXxD",
        "xxcDcbaxDXBDbxcxXxXxDxBDXXXBXxDcxXca",
        "xxxcDxxxDXBcaxbcXXxbaxxCcDdDxXXXCBXC",
        "abXXCbxxxdXcaxDCXXxxccxXDXXXBBcXccXC",
        "bCXXBCcXdXBXDBxDdXxCxxxXxXxxxDaxaxxb",
        "CCXXXxXxCxcaDxBXXxbcXdDxcXCbxaXXxxaa",
        "DCXXBCxxxdXXbDXXXdxcDdXCxbccXCxDXCXx",
        "xxDxCXxCBxBXXAXBAXcCxCXdxDbxXXabxxxD",
        "DXXBCxxXxXAXXBbdXcxXcDBbaxxxDbxxxaxx",
        "xDBCXCaxXcxCbDXbcXbbcxXXDdXcxXCXBCCX",
        "CCXCCDCdxxxxXBCXDbxCxbbaxXCcbaaxXXXx",
        "axXXBDbXCdXxbDxxBXaxcCXBbCXBxCxDxDxX",
    ],
    &[
        "DcXDbcXXdXxcCCBCXXXXxXxcXdXxXDcDxxaXxxxbxaCXXXXXb",
        "xxXbxaxabXbxaxxxCxxxxxXXCcCCDAXXxBBxXXXBBXDXXdDdX",
        "XCdDdCXBBXBXXxCxbdXXCXxxXXdXbaxcxxCabDBXXBxaxXdCX",
        "babDcXCXxcXxXBXXCXxBXxDxXXBXabxxcdCxabXCBBxxbDxXX",
        "xabDCxXxaxxBXCxcXxCxxDXXXACDxDdBXdxxxxXBXDabXCxXc",
        "xaaxbcDxxbxXXXbXCCXXcxxxbxCcbDbxCBCcCdXXdCDBCCxxX",
        "xcCCcxDDXAXXdXxxCBACBxxxxXxXbXxCXcXbXXXdbXabcXDbx",
        "xxcCXxaxXCABDxbxdCdxbCXXAXXXCcdXxbxXxCBCbxXcXxDbx",
        "DXXxDxbdXBdBCXXAXXBxbdBXdCDbDXXxxxxxdBXXxxDXCxbax",
        "bbXcXxaXdXdXXcXCXBXXCXcdXxxXxxDXDbxaxxXxaxxxDCxxa",
        "XCxXXBXXABBXBXxXBxDcxacCcxaxxcBXbaxDXBxxaaxcDxaxx",
        "XXXXXbadXcxbaxDXxXcbbcXCXXCDxCxcCdxDXXxCBDxbbxxXx",
        "XbxXBCcXxCXXAXBXXdcDxCxXCxxxXbxXXbxCcCXxaaXxCXbaa",
        "XcDdXCDCxXBBxxCdXXAXDBCBxCxxCxCxBCDXcXxXcxxbDxbxa",
        "CCdCCXxXXCBxxaxCxXXbaXBcBBbxcCcXXcxbXbcXXbxbxxDxx",
        "xxxxCXDxxxcCxxxxDXXxDabcxXCBbXCccxCxXBXBXXxDccXdX",
        "abxDxbXxDCXxbXabxXXcXxxXXxbXXBBXCxCcCcdXXBbXcXXBX",
        "bDXCBCCxccdCdXbDXXXxxbxcdBCDCBXXXcxXXcdXxxDxxXCxa",
        "DdXXCbaBXXxXxxXXCcDxxCxbbbxxXXdDxxxBAXBxxxXCdXDxx",
        "XCXDxxxxxCxbbaDBXBBCxxXdXXCxCBxCxcxXBxXXCXcDxDxbx",
        "XBXxDxxcCxxBXDcBCCXdxXBdxCXDxBDxdXxbCdxDXxxXXxxxa",
        "xxaaaxDcDxxxxXDXxDcbXcCXBXdXxcxxBXBDCXBXXCbcXXdXc",
        "DCBXXXXxdCxCABDXCbxXBxXxxxdXXXDxxDCbxbxaxxaxaaxxx",
        "XCXdXXDxxBXBdbDBBxXXxcXXcxXxaxXDcXbxxCcxbaxDCbaxa",
        "XdXCxaaCCXADxxxxBXcxaXBXxaxxxBXXxxxCBdBCbaCXxXXxa",
        "XXXXXXbCdcxXcaBXDcXxxXxxbXBCXCXbcBBCdXbxXCXxbaaxx",
        "bxaaxbDCXxxDcXXBcbXcXCBXcXxXxxXxCxCDdCxBCCXXXXXcx",
        "xxcDxCDxcXBcXxXCXXxXbccdBCBxDXXXdCCxdXdxxCXCBXDcD",
        "xcDxcCCbCXxXXCxXBbxCxxxCcDACbcCbdXXXXBxXAXxcCXXCx",
        "xxbaaxaxXCxxbxacCbxDxxxXxbXbDXXCxXccXAACXXbXBXxCC",
        "DcDXcxbXxcXCCDCcxXxxxBCXXDxaXxXccxbCxCdXXDXxCXXxb",
        "aabxbXXxxXCcxXxacXXxCaxxdCdCcXdXBXCXBXXxXxbcDcbXb",
        "DbcXxaaCxXCbxxXcCdxxaXxXCXXcXXXdXXXdXdbxcxDXXCCXb",
        "DcXCdCCXxBBXBCCXBxcxcxBXBBBXXBxCBBXBXxbbxxDcxxaxa",
        "DXXxCXDxdXdXdccCXXXXXDCxDxXCxbabcXdaxxxXXXbXXXCdX",
        "xxxxxcDxbDcxXXxxCCxdXDxcCcXBCXxCxxCcXXAXXBbDxCXdX",
        "DXCdXcacxBXBXxXdXxbxxXBXXcxxdXdXXbxCXxxcaxXXXXDba",
        "xDXdXdDxcXXXCXabcdCxDbXXBXccbBXBXXDxBBxcDxXBXXbxx",
        "xDxbXdDxCcxXXXDXCcxCdxdBXBXCCXXdXxXDxDxXcbxxbxCXx",
        "xDXXxDxxxxBCAXabDBxXxbxxCxbaXbcXDbxCxXXdbaCCXXXXx",
        "xcDcCXDbXXxXcbxxCcCxaxCBxBDxXXXXBcxbdXBBdXxDXxXCX",
        "CXxCBCXBXXXXxcXCdBBxDbxxCXXCxaxxXXxxxxxdXDxaxbDCx",
        "CXCXCXDCBxxccxdXCBCCXDXxCcxbdXcXbbxXXcCxXccDxCCBD",
        "bcXxxaaCXXDxxxDdXdbxxxxBBXxxXBXBXbxCBxcxbxxCXCCDx",
        "cXbxaxxDBxXcbbxBBACCXXBXXxxbxdXdcXbxCBBCBxXCdXxCX",
        "XCXDxaxCdcxxxbBXCCXXCXXxxCdXCdxCXbxXXxCBbxXcacXxx",
        "XCDxaxxXxcxxxaXxDxxaxCdXdDxxXBBXCxbxXxxccDDXXCCXX",
        "XXCxxxDcdBXxCXCCBBXXdCdbxxBDXDxXCXdCcxDdXXXxxxbCX",
    ],
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::{bot_games, bot_games_with};

    #[test]
    fn generated_fields_have_a_single_fence() {
        for seed in 0..4 {
            for round in 0..3 {
                let f = generate(round, &mut Rng::new(seed));
                let n = f.size;
                assert!(is_valid_area(n, &f.inside));
                assert!(is_solved(n, &f.clues, &fence_of(n, &f.inside)), "seed {seed} round {round}");
                let mut budget = u64::MAX;
                assert_eq!(solve(n, &f.clues, 2, &mut budget).unwrap(), vec![f.inside.clone()]);
            }
        }
    }

    #[test]
    fn bank_fields_have_a_single_fence_in_every_turn() {
        for (round, bank) in BANK.iter().enumerate() {
            assert!(bank.len() >= 40, "round {round}");
            for (i, code) in bank.iter().enumerate() {
                let n = SIZES[round];
                let base = decode(n, code);
                assert_eq!(encode(&base), *code);
                let f = transform(&base, i % 8);
                assert!(is_valid_area(n, &f.inside));
                assert!(is_solved(n, &f.clues, &fence_of(n, &f.inside)));
                let mut budget = u64::MAX;
                assert_eq!(solve(n, &f.clues, 2, &mut budget).unwrap().len(), 1, "round {round} field {i}");
            }
        }
    }

    #[test]
    fn transforms_are_the_eight_symmetries() {
        let f = decode(5, BANK[0][0]);
        let all: Vec<String> = (0..8).map(|t| encode(&transform(&f, t))).collect();
        assert_eq!(encode(&transform(&f, 0)), BANK[0][0]);
        // Four quarter turns come back to the start.
        let mut g = f.clone();
        for _ in 0..4 {
            g = transform(&g, 1);
        }
        assert_eq!(encode(&g), BANK[0][0]);
        assert!(all.iter().all(|code| code.len() == 25));
    }

    /// Prints a new bank: `cargo test --release -p wp-rules print_bank -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn print_bank() {
        println!("pub const BANK: [&[&str]; 3] = [");
        for round in 0..3 {
            println!("    &[");
            for seed in 0..48 {
                println!("        \"{}\",", encode(&generate(round, &mut Rng::new(seed * 31 + round as u64))));
            }
            println!("    ],");
        }
        println!("];");
    }

    #[test]
    fn loop_checks() {
        // 2×2 field, the area is the top-left plot: a square fence.
        let n = 2;
        let inside = [true, false, false, false];
        let fence = fence_of(n, &inside);
        let clues = vec![Some(4), None, None, Some(0)];
        assert!(is_solved(n, &clues, &fence));
        assert!(!is_solved(n, &[Some(3), None, None, None], &fence));
        // Two separate squares are not one loop.
        let two = fence_of(n, &[true, false, false, true]);
        assert!(!is_solved(n, &[None; 4], &two));
        assert!(!is_valid_area(n, &[true, false, false, true]));
        // A dangling edge.
        let mut open = fence.clone();
        open[sides(n, 3)[3]] = true;
        assert!(!is_solved(n, &[None; 4], &open));
    }

    #[test]
    fn fencing_the_solution_ends_the_round() {
        let mut s = Rules::setup(&["p1".to_string()], 4).unwrap();
        let fence = fence_of(s.field.size, &s.field.inside);
        assert!(Rules::apply_move(&mut s, "p1", FenceMove::Edge { edge: 999, mark: 1 }).is_err());
        let mut lines = (0..fence.len()).filter(|&e| fence[e]);
        let first = lines.next().unwrap();
        Rules::apply_move(&mut s, "p1", FenceMove::Edge { edge: first, mark: 2 }).unwrap();
        for e in lines {
            Rules::apply_move(&mut s, "p1", FenceMove::Edge { edge: e, mark: 1 }).unwrap();
        }
        assert_eq!(s.race.round, 0);
        Rules::apply_move(&mut s, "p1", FenceMove::Edge { edge: first, mark: 1 }).unwrap();
        assert_eq!(s.race.round, 1);
        assert_eq!(s.race.last.as_ref().unwrap()[0].place, Some(1));
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<Rules>(1000, 5000);
        bot_games_with::<Rules>(1000, 4, 5000);
    }
}
