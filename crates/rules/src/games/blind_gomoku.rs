//! Blind Gomoku (hidden game, solo against the house bot). Rules page: docs/games/blind-gomoku.md
//!
//! Five in a row on 9×9, but only the newest tile shows its colour; older tiles
//! lie face down. The bot is part of the rules (it moves on tick) and has an
//! imperfect memory of its own.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::rng::Rng;

pub const SIZE: usize = 9;
pub const CELLS: usize = SIZE * SIZE;
pub const PEEKS: u32 = 2;
pub const PEEK_MS: i64 = 3_000;
pub const BOT_THINK_MS: i64 = 800;
const DIRS: [(i32, i32); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Dark,
    Light,
}

#[derive(Clone, Debug)]
pub struct Stone {
    pub tile: Tile,
    /// Whether the bot still knows this tile's colour.
    pub bot_knows: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Outcome {
    Won,
    Lost,
    Draw,
}

#[derive(Clone, Debug)]
pub struct State {
    pub player: String,
    pub board: Vec<Option<Stone>>,
    pub last: Option<usize>,
    pub your_turn: bool,
    pub bot_wait_ms: i64,
    pub peeks_left: u32,
    pub peek_ms: i64,
    pub outcome: Option<Outcome>,
    pub win_line: Vec<usize>,
    pub rng: Rng,
    /// Replay moments: your winning line, with or without a peek.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Place { cell: usize },
    Peek,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub size: usize,
    /// null empty, "down", "dark" or "light".
    pub cells: Vec<Option<&'static str>>,
    pub last: Option<usize>,
    pub your_turn: bool,
    pub peeks_left: u32,
    pub peeking: bool,
    pub peek_left_ms: i64,
    pub tiles: usize,
    pub outcome: Option<Outcome>,
    pub win_line: Vec<usize>,
    pub score: i64,
    pub over: bool,
}

fn at(x: i32, y: i32) -> Option<usize> {
    (0..SIZE as i32).contains(&x).then_some(())?;
    (0..SIZE as i32).contains(&y).then_some(())?;
    Some(y as usize * SIZE + x as usize)
}

/// The longest run of `tile` through `cell` per direction, as cell lists.
fn runs(board: &[Option<Tile>], cell: usize, tile: Tile) -> Vec<Vec<usize>> {
    let (x, y) = ((cell % SIZE) as i32, (cell / SIZE) as i32);
    DIRS.iter()
        .map(|&(dx, dy)| {
            let mut line = vec![cell];
            for sign in [1, -1] {
                let mut k = 1;
                while let Some(c) = at(x + sign * k * dx, y + sign * k * dy) {
                    if board[c] != Some(tile) {
                        break;
                    }
                    line.push(c);
                    k += 1;
                }
            }
            line.sort();
            line
        })
        .collect()
}

fn longest_line(board: &[Option<Tile>], tile: Tile) -> usize {
    (0..CELLS).filter(|&c| board[c] == Some(tile)).flat_map(|c| runs(board, c, tile)).map(|l| l.len()).max().unwrap_or(0)
}

/// How good it is for `tile` to own `cell`: the lines it would make.
/// `board` uses `None` for empty; unknown tiles should be passed as blocked.
fn line_value(board: &[Option<Tile>], blocked: &[bool], cell: usize, tile: Tile) -> i64 {
    let (x, y) = ((cell % SIZE) as i32, (cell / SIZE) as i32);
    let mut total = 0;
    for &(dx, dy) in &DIRS {
        let mut count = 1;
        let mut open = 0;
        for sign in [1, -1] {
            let mut k = 1;
            loop {
                match at(x + sign * k * dx, y + sign * k * dy) {
                    Some(c) if !blocked[c] && board[c] == Some(tile) => {
                        count += 1;
                        k += 1;
                    }
                    Some(c) if !blocked[c] && board[c].is_none() => {
                        open += 1;
                        break;
                    }
                    _ => break,
                }
            }
        }
        total += match (count.min(5), open) {
            (5, _) => 1_000_000,
            (4, 2) => 50_000,
            (4, 1) => 5_000,
            (3, 2) => 4_000,
            (3, 1) => 300,
            (2, 2) => 200,
            (2, 1) => 30,
            (1, 2) => 8,
            (_, 0) => 0,
            _ => 2,
        };
    }
    total
}

/// The best square for `me` on a board as remembered: `memory[c]` is Some(tile)
/// for a known tile, `blocked[c]` for a taken square whose colour is forgotten.
fn best_cell(memory: &[Option<Tile>], blocked: &[bool], me: Tile, rng: &mut Rng) -> Option<usize> {
    let them = if me == Tile::Dark { Tile::Light } else { Tile::Dark };
    let centre = (SIZE / 2) as i64;
    (0..CELLS)
        .filter(|&c| memory[c].is_none() && !blocked[c])
        .map(|c| {
            let (x, y) = ((c % SIZE) as i64, (c / SIZE) as i64);
            let near_centre = 8 - (x - centre).abs().max((y - centre).abs());
            let v = line_value(memory, blocked, c, me) * 11 / 10 + line_value(memory, blocked, c, them) + near_centre + rng.int(0, 3);
            (v, c)
        })
        .max()
        .map(|(_, c)| c)
}

impl State {
    fn tiles(&self) -> Vec<Option<Tile>> {
        self.board.iter().map(|s| s.as_ref().map(|s| s.tile)).collect()
    }

    fn place(&mut self, cell: usize, tile: Tile) {
        let keep = if tile == Tile::Light { 0.9 } else { 0.8 };
        let bot_knows = self.rng.next_f64() < keep;
        self.board[cell] = Some(Stone { tile, bot_knows });
        self.last = Some(cell);
        let tiles = self.tiles();
        if let Some(line) = runs(&tiles, cell, tile).into_iter().find(|l| l.len() >= 5) {
            // A replay moment: your winning line (every tile is shown at the end anyway).
            if tile == Tile::Dark {
                let n = ["Five", "Six", "Seven", "Eight", "Nine"][line.len().min(9) - 5];
                let how = if self.peeks_left == PEEKS { "without a peek" } else { "on a blind board" };
                self.moments.push(&self.player, format!("{n} in a row {how}"));
            }
            self.win_line = line;
            self.outcome = Some(if tile == Tile::Dark { Outcome::Won } else { Outcome::Lost });
        } else if self.board.iter().all(Option::is_some) {
            self.outcome = Some(Outcome::Draw);
        }
    }

    fn bot_play(&mut self) {
        let last = self.last;
        let memory: Vec<Option<Tile>> =
            self.board.iter().enumerate().map(|(c, s)| s.as_ref().filter(|s| s.bot_knows || Some(c) == last).map(|s| s.tile)).collect();
        let blocked: Vec<bool> = (0..CELLS).map(|c| self.board[c].is_some() && memory[c].is_none()).collect();
        let mut rng = self.rng.clone();
        if let Some(c) = best_cell(&memory, &blocked, Tile::Light, &mut rng) {
            self.rng = rng;
            self.place(c, Tile::Light);
        }
        self.your_turn = true;
    }

    fn your_tiles(&self) -> usize {
        self.board.iter().flatten().filter(|s| s.tile == Tile::Dark).count()
    }

    pub fn score(&self) -> i64 {
        match self.outcome {
            Some(Outcome::Won) => (1000 - 15 * (self.your_tiles() as i64 - 5) - 100 * i64::from(PEEKS - self.peeks_left)).max(400),
            Some(Outcome::Draw) => 250,
            Some(Outcome::Lost) => 30 * longest_line(&self.tiles(), Tile::Dark) as i64,
            None => 0,
        }
    }
}

pub struct BlindGomoku;

impl Game for BlindGomoku {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "blind-gomoku",
            kind: "hidden",
            difficulty: "hard",
            tags: &["board", "memory"],
            min_players: 1,
            max_players: 1,
            name: "Blind Gomoku",
            short: "Five in a row against the bot, but older tiles turn face down: remember which are yours.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        let [player] = players else { return Err("Blind Gomoku is played against the house bot".into()) };
        Ok(State {
            player: player.clone(),
            board: vec![None; CELLS],
            last: None,
            your_turn: true,
            bot_wait_ms: 0,
            peeks_left: PEEKS,
            peek_ms: 0,
            outcome: None,
            win_line: Vec::new(),
            rng: Rng::new(seed),
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.outcome.is_some() {
            return Err("Game is over".into());
        }
        if player != s.player {
            return Err("Not your game".into());
        }
        match mv {
            Move::Place { cell } => {
                if !s.your_turn {
                    return Err("The bot is thinking".into());
                }
                if cell >= CELLS || s.board[cell].is_some() {
                    return Err("That square is taken".into());
                }
                s.place(cell, Tile::Dark);
                s.your_turn = false;
                s.bot_wait_ms = BOT_THINK_MS;
                Ok(())
            }
            Move::Peek => {
                if s.peeks_left == 0 || s.peek_ms > 0 {
                    return Err("No peek left".into());
                }
                s.peeks_left -= 1;
                s.peek_ms = PEEK_MS;
                Ok(())
            }
        }
    }

    fn tick(s: &mut State, ms: u64) {
        if s.outcome.is_some() {
            return;
        }
        s.peek_ms = (s.peek_ms - ms as i64).max(0);
        if !s.your_turn {
            s.bot_wait_ms -= ms as i64;
            if s.bot_wait_ms <= 0 {
                s.bot_play();
            }
        }
    }

    fn view_for(s: &State, _player: &str) -> View {
        let over = s.outcome.is_some();
        let open = over || s.peek_ms > 0;
        View {
            size: SIZE,
            cells: s
                .board
                .iter()
                .enumerate()
                .map(|(c, st)| {
                    st.as_ref().map(|st| match (open || Some(c) == s.last, st.tile) {
                        (false, _) => "down",
                        (true, Tile::Dark) => "dark",
                        (true, Tile::Light) => "light",
                    })
                })
                .collect(),
            last: s.last,
            your_turn: s.your_turn && !over,
            peeks_left: s.peeks_left,
            peeking: s.peek_ms > 0,
            peek_left_ms: s.peek_ms,
            tiles: s.board.iter().flatten().count(),
            outcome: s.outcome,
            win_line: s.win_line.clone(),
            score: s.score(),
            over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.outcome.is_some()
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        vec![PlayerResult { player_id: s.player.clone(), rank: 1, points: s.score() }]
    }

    fn to_move(s: &State) -> Vec<String> {
        if s.your_turn && s.outcome.is_none() { vec![s.player.clone()] } else { Vec::new() }
    }

    /// Plays the dark tiles with the same engine, remembering 85% of the tiles.
    fn bot_move(s: &State, _player: &str, seed: u64) -> Move {
        let mut rng = Rng::new(seed);
        let last = s.last;
        let memory: Vec<Option<Tile>> =
            s.board.iter().enumerate().map(|(c, st)| st.as_ref().filter(|_| Some(c) == last || rng.next_f64() < 0.85).map(|st| st.tile)).collect();
        let blocked: Vec<bool> = (0..CELLS).map(|c| s.board[c].is_some() && memory[c].is_none()).collect();
        match best_cell(&memory, &blocked, Tile::Dark, &mut rng) {
            Some(cell) => Move::Place { cell },
            None => Move::Place { cell: s.board.iter().position(Option::is_none).unwrap_or(0) },
        }
    }

    fn moments(s: &State, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub type Rules = BlindGomoku;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    const P: &str = "p1";

    fn play(s: &mut State, cell: usize) {
        BlindGomoku::apply_move(s, P, Move::Place { cell }).unwrap();
        BlindGomoku::tick(s, BOT_THINK_MS as u64);
    }

    #[test]
    fn older_tiles_lie_face_down() {
        let mut s = BlindGomoku::setup(&[P.into()], 1).unwrap();
        play(&mut s, 40);
        let v = BlindGomoku::view_for(&s, P);
        assert_eq!(v.cells[40], Some("down"));
        let last = v.last.unwrap();
        assert_eq!(v.cells[last], Some("light"));
        assert_eq!(v.cells.iter().flatten().count(), 2);
        assert!(v.your_turn);
    }

    #[test]
    fn a_peek_shows_every_colour_for_three_seconds() {
        let mut s = BlindGomoku::setup(&[P.into()], 2).unwrap();
        play(&mut s, 40);
        BlindGomoku::apply_move(&mut s, P, Move::Peek).unwrap();
        assert_eq!(BlindGomoku::view_for(&s, P).cells[40], Some("dark"));
        assert!(BlindGomoku::apply_move(&mut s, P, Move::Peek).is_err());
        BlindGomoku::tick(&mut s, PEEK_MS as u64);
        assert_eq!(BlindGomoku::view_for(&s, P).cells[40], Some("down"));
        BlindGomoku::apply_move(&mut s, P, Move::Peek).unwrap();
        BlindGomoku::tick(&mut s, PEEK_MS as u64);
        assert!(BlindGomoku::apply_move(&mut s, P, Move::Peek).is_err());
    }

    #[test]
    fn taken_squares_and_the_bots_turn_are_refused() {
        let mut s = BlindGomoku::setup(&[P.into()], 3).unwrap();
        BlindGomoku::apply_move(&mut s, P, Move::Place { cell: 10 }).unwrap();
        assert!(BlindGomoku::apply_move(&mut s, P, Move::Place { cell: 11 }).is_err());
        BlindGomoku::tick(&mut s, BOT_THINK_MS as u64);
        assert!(BlindGomoku::apply_move(&mut s, P, Move::Place { cell: 10 }).is_err());
        assert!(BlindGomoku::apply_move(&mut s, P, Move::Place { cell: CELLS }).is_err());
    }

    #[test]
    fn five_in_a_row_wins_and_scores() {
        let mut s = BlindGomoku::setup(&[P.into()], 4).unwrap();
        // Force the bot's tiles out of the way: fill the row by hand.
        for x in 0..5 {
            s.board[SIZE * 8 + x] = Some(Stone { tile: Tile::Light, bot_knows: true });
        }
        s.board[SIZE * 8] = None;
        for x in 0..4 {
            s.board[x] = Some(Stone { tile: Tile::Dark, bot_knows: true });
        }
        BlindGomoku::apply_move(&mut s, P, Move::Place { cell: 4 }).unwrap();
        assert_eq!(s.outcome, Some(Outcome::Won));
        assert_eq!(s.win_line, vec![0, 1, 2, 3, 4]);
        assert_eq!(s.score(), 1000);
        assert_eq!(BlindGomoku::view_for(&s, P).cells[0], Some("dark"));
        let m = BlindGomoku::moments(&s, P);
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), (P, "Five in a row without a peek"));
    }

    #[test]
    fn a_win_after_a_peek_and_a_loss_are_told_apart() {
        let mut s = BlindGomoku::setup(&[P.into()], 4).unwrap();
        BlindGomoku::apply_move(&mut s, P, Move::Peek).unwrap();
        for x in [0, 1, 2, 4, 5] {
            s.board[x] = Some(Stone { tile: Tile::Dark, bot_knows: true });
        }
        BlindGomoku::apply_move(&mut s, P, Move::Place { cell: 3 }).unwrap();
        let m = BlindGomoku::moments(&s, P);
        assert_eq!((m.len(), m[0].text.as_str()), (1, "Six in a row on a blind board"));
        // The house bot's five is no moment of yours.
        let mut s = BlindGomoku::setup(&[P.into()], 4).unwrap();
        for x in 0..4 {
            s.board[SIZE * 8 + x] = Some(Stone { tile: Tile::Light, bot_knows: true });
        }
        s.place(SIZE * 8 + 4, Tile::Light);
        assert_eq!(s.outcome, Some(Outcome::Lost));
        assert!(BlindGomoku::moments(&s, P).is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<BlindGomoku>(300, 1, 2000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.most <= 1, "one win at most: {stats:?}");
    }

    #[test]
    fn the_bot_blocks_an_open_four() {
        let mut s = BlindGomoku::setup(&[P.into()], 5).unwrap();
        for x in 1..4 {
            s.board[SIZE * 4 + x] = Some(Stone { tile: Tile::Dark, bot_knows: true });
        }
        BlindGomoku::apply_move(&mut s, P, Move::Place { cell: SIZE * 4 + 4 }).unwrap();
        BlindGomoku::tick(&mut s, BOT_THINK_MS as u64);
        let blocked = [SIZE * 4, SIZE * 4 + 5].iter().any(|&c| s.board[c].as_ref().is_some_and(|t| t.tile == Tile::Light));
        assert!(blocked);
    }

    #[test]
    fn bot_wins_some_games_against_the_house() {
        let mut won = 0;
        for seed in 1..=40 {
            let mut s = BlindGomoku::setup(&[P.into()], seed).unwrap();
            let mut n = 0;
            while s.outcome.is_none() {
                let mv = BlindGomoku::bot_move(&s, P, seed * 977 + n);
                BlindGomoku::apply_move(&mut s, P, mv).unwrap();
                BlindGomoku::tick(&mut s, BOT_THINK_MS as u64);
                n += 1;
            }
            won += usize::from(s.outcome == Some(Outcome::Won));
        }
        assert!((4..=36).contains(&won), "won {won} of 40");
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<BlindGomoku>(1000, 2000);
    }
}
