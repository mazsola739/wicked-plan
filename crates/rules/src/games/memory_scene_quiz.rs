//! Memory Scene Quiz (Team mini game, 1–4 players). Rules page: docs/games/memory-scene-quiz.md
//!
//! Study a scene of coloured shapes, then answer 10 questions about it in
//! turn. One wrong answer fails the whole team.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::rng::Rng;

pub const SHAPES: [&str; 6] = ["circle", "square", "triangle", "star", "heart", "moon"];
pub const COLOURS: [&str; 5] = ["red", "blue", "green", "yellow", "purple"];
pub const COLS: usize = 4;
pub const ROWS: usize = 3;
pub const OBJECTS: usize = 8;
pub const QUESTIONS: usize = 10;
pub const POINTS: i64 = 100;
pub const BOT_RIGHT: f64 = 0.95;

const ROW_NAMES: [&str; ROWS] = ["top", "middle", "bottom"];
const COL_NAMES: [&str; COLS] = ["first", "second", "third", "fourth"];

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    pub shape: &'static str,
    pub colour: &'static str,
    pub count: u32,
    pub row: usize,
    pub col: usize,
}

impl Object {
    fn desc(&self) -> String {
        format!("{} {}", self.colour, self.shape)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub kind: &'static str,
    pub text: String,
    pub options: Vec<String>,
    #[serde(skip)]
    pub answer: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Study,
    Questions,
    Over,
}

#[derive(Clone, Debug)]
pub struct State {
    pub players: Vec<String>,
    pub objects: Vec<Object>,
    pub questions: Vec<Question>,
    pub phase: Phase,
    pub ready: Vec<bool>,
    /// The answers given so far.
    pub given: Vec<usize>,
    pub failed: bool,
    /// Replay moments: the tenth right answer that makes a perfect quiz.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Ready,
    Answer { choice: usize },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Answered {
    pub text: String,
    pub options: Vec<String>,
    pub seat: usize,
    pub given: usize,
    pub answer: usize,
    pub correct: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: Vec<String>,
    pub you: Option<usize>,
    pub phase: Phase,
    pub cols: usize,
    pub rows: usize,
    /// Only while studying and after the end.
    pub objects: Option<Vec<Object>>,
    pub ready: Vec<bool>,
    pub question: Option<Question>,
    pub number: usize,
    pub questions: usize,
    pub answerer: Option<usize>,
    pub your_turn: bool,
    pub history: Vec<Answered>,
    pub score: i64,
    pub failed: bool,
}

/// Eight objects in different cells, no two with the same colour and shape.
pub fn generate_scene(rng: &mut Rng) -> Vec<Object> {
    let mut cells: Vec<usize> = (0..COLS * ROWS).collect();
    rng.shuffle(&mut cells);
    let mut kinds: Vec<(usize, usize)> = (0..SHAPES.len()).flat_map(|s| (0..COLOURS.len()).map(move |c| (s, c))).collect();
    rng.shuffle(&mut kinds);
    let mut objects: Vec<Object> = (0..OBJECTS)
        .map(|i| Object {
            shape: SHAPES[kinds[i].0],
            colour: COLOURS[kinds[i].1],
            count: rng.int(1, 3) as u32,
            row: cells[i] / COLS,
            col: cells[i] % COLS,
        })
        .collect();
    objects.sort_by_key(|o| (o.row, o.col));
    objects
}

fn with_options(kind: &'static str, text: String, answer: String, mut pool: Vec<String>, rng: &mut Rng) -> Option<Question> {
    pool.retain(|p| *p != answer);
    pool.sort();
    pool.dedup();
    if pool.len() < 3 {
        return None;
    }
    rng.shuffle(&mut pool);
    let mut options: Vec<String> = pool.into_iter().take(3).collect();
    options.push(answer.clone());
    rng.shuffle(&mut options);
    let answer = options.iter().position(|o| *o == answer)?;
    Some(Question { kind, text, options, answer })
}

fn number_question(kind: &'static str, text: String, answer: u32, rng: &mut Rng) -> Question {
    let start = answer.saturating_sub(rng.int(0, 3) as u32);
    Question { kind, text, options: (start..start + 4).map(|n| n.to_string()).collect(), answer: (answer - start) as usize }
}

/// Every question the scene allows, before picking ten.
pub fn all_questions(objects: &[Object], rng: &mut Rng) -> Vec<Question> {
    let mut out = Vec::new();
    let shapes: Vec<String> = SHAPES.iter().map(|s| s.to_string()).collect();
    let colours: Vec<String> = COLOURS.iter().map(|s| s.to_string()).collect();
    let descs: Vec<String> = objects.iter().map(Object::desc).collect();
    let at = |row: usize, col: usize| objects.iter().find(|o| o.row == row && o.col == col);
    for o in objects {
        if objects.iter().filter(|p| p.shape == o.shape).count() == 1 {
            let text = format!("What colour was the {}?", o.shape);
            out.extend(with_options("colour", text, o.colour.to_string(), colours.clone(), rng));
        }
        let text = format!("Which shape was in the {} row, {} from the left?", ROW_NAMES[o.row], COL_NAMES[o.col]);
        out.extend(with_options("cell", text, o.shape.to_string(), shapes.clone(), rng));
        let text = format!("How many {} {}s were in their cell?", o.colour, o.shape);
        out.push(Question { kind: "count", text, options: (1..=4).map(|n| n.to_string()).collect(), answer: o.count as usize - 1 });
        if let Some(p) = (o.col + 1 < COLS).then(|| at(o.row, o.col + 1)).flatten() {
            let text = format!("What was just right of the {}?", o.desc());
            let pool = descs.iter().filter(|d| **d != o.desc()).cloned().collect();
            out.extend(with_options("right-of", text, p.desc(), pool, rng));
        }
    }
    for shape in SHAPES {
        let n = objects.iter().filter(|o| o.shape == shape).map(|o| o.count).sum();
        out.push(number_question("total", format!("How many {shape}s were there in all?"), n, rng));
    }
    for colour in COLOURS {
        let n = objects.iter().filter(|o| o.colour == colour).count() as u32;
        out.push(number_question("colour-count", format!("How many cells held something {colour}?"), n, rng));
    }
    for _ in 0..3 {
        let s = *rng.pick(&SHAPES);
        let c = *rng.pick(&COLOURS);
        if !objects.iter().any(|o| o.shape == s && o.colour == c) {
            out.extend(with_options("missing", "Which of these was NOT in the scene?".into(), format!("{c} {s}"), descs.clone(), rng));
        }
    }
    out
}

/// Ten questions with a mix of kinds (at most two of a kind while there is choice).
pub fn pick_questions(objects: &[Object], rng: &mut Rng) -> Vec<Question> {
    let mut all = all_questions(objects, rng);
    rng.shuffle(&mut all);
    let mut picked: Vec<Question> = Vec::new();
    for cap in 2..=QUESTIONS {
        for q in &all {
            if picked.len() == QUESTIONS {
                break;
            }
            let same = picked.iter().filter(|p| p.kind == q.kind).count();
            if same < cap && !picked.iter().any(|p| p.text == q.text) {
                picked.push(q.clone());
            }
        }
    }
    picked
}

impl State {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }

    pub fn answerer(&self) -> Option<usize> {
        (self.phase == Phase::Questions).then(|| self.given.len() % self.players.len())
    }

    pub fn right_answers(&self) -> usize {
        self.given.iter().zip(&self.questions).filter(|(g, q)| **g == q.answer).count()
    }

    fn start_questions(&mut self) {
        self.phase = Phase::Questions;
    }

    fn answer(&mut self, choice: usize) {
        let q = &self.questions[self.given.len()];
        let right = choice == q.answer;
        self.given.push(choice);
        if !right {
            self.failed = true;
        }
        if !right || self.given.len() == self.questions.len() {
            if right {
                // A replay moment: the last question answered right, a perfect quiz (answers are public).
                let by = self.players[(self.given.len() - 1) % self.players.len()].clone();
                let points = self.right_answers() as i64 * POINTS;
                self.moments.push(&by, format!("Question {} right: a perfect {points}", self.given.len()));
            }
            self.phase = Phase::Over;
        }
    }
}

pub struct MemorySceneQuiz;

impl Game for MemorySceneQuiz {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "memory-scene-quiz",
            kind: "mini-team",
            difficulty: "medium",
            tags: &["memory", "alliance"],
            min_players: 1,
            max_players: 4,
            name: "Memory Scene Quiz",
            short: "Study a scene, then answer 10 questions in turn. One wrong answer fails the team.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if !(1..=4).contains(&players.len()) {
            return Err("Memory Scene Quiz is for 1–4 players".into());
        }
        let mut rng = Rng::new(seed);
        let objects = generate_scene(&mut rng);
        let questions = pick_questions(&objects, &mut rng);
        Ok(State {
            players: players.to_vec(),
            objects,
            questions,
            phase: Phase::Study,
            ready: vec![false; players.len()],
            given: Vec::new(),
            failed: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        let seat = s.seat(player).ok_or("Not in this game")?;
        match (s.phase, mv) {
            (Phase::Study, Move::Ready) => {
                s.ready[seat] = true;
                if s.ready.iter().all(|&r| r) {
                    s.start_questions();
                }
            }
            (Phase::Questions, Move::Answer { choice }) => {
                if s.answerer() != Some(seat) {
                    return Err("Not your question".into());
                }
                if choice >= s.questions[s.given.len()].options.len() {
                    return Err("No such answer".into());
                }
                s.answer(choice);
            }
            (Phase::Over, _) => return Err("Game is over".into()),
            _ => return Err("Not now".into()),
        }
        Ok(())
    }

    fn view_for(s: &State, player: &str) -> View {
        let you = s.seat(player);
        let answerer = s.answerer();
        View {
            players: s.players.clone(),
            you,
            phase: s.phase,
            cols: COLS,
            rows: ROWS,
            objects: (s.phase != Phase::Questions).then(|| s.objects.clone()),
            ready: s.ready.clone(),
            question: (s.phase == Phase::Questions).then(|| s.questions[s.given.len()].clone()),
            number: (s.given.len() + 1).min(QUESTIONS),
            questions: s.questions.len(),
            answerer,
            your_turn: answerer.is_some() && answerer == you,
            history: s
                .given
                .iter()
                .zip(&s.questions)
                .enumerate()
                .map(|(i, (g, q))| Answered {
                    text: q.text.clone(),
                    options: q.options.clone(),
                    seat: i % s.players.len(),
                    given: *g,
                    answer: q.answer,
                    correct: *g == q.answer,
                })
                .collect(),
            score: s.right_answers() as i64 * POINTS,
            failed: s.failed,
        }
    }

    fn is_over(s: &State) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        let pts = s.right_answers() as i64 * POINTS;
        s.players.iter().map(|p| PlayerResult { player_id: p.clone(), rank: 1, points: pts }).collect()
    }

    fn to_move(s: &State) -> Vec<String> {
        match s.phase {
            Phase::Study => (0..s.players.len()).filter(|&k| !s.ready[k]).map(|k| s.players[k].clone()).collect(),
            Phase::Questions => s.answerer().map(|k| vec![s.players[k].clone()]).unwrap_or_default(),
            Phase::Over => Vec::new(),
        }
    }

    fn moments(s: &State, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &State, _player: &str, seed: u64) -> Move {
        if s.phase != Phase::Questions {
            return Move::Ready;
        }
        let mut rng = Rng::new(seed);
        let q = &s.questions[s.given.len()];
        if rng.next_f64() < BOT_RIGHT {
            Move::Answer { choice: q.answer }
        } else {
            Move::Answer { choice: (q.answer + rng.int(1, 3) as usize) % q.options.len() }
        }
    }
}

pub const READY: bool = true;
pub type Rules = MemorySceneQuiz;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn team(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    /// Works out a question's answer from the scene again, independently of the generator.
    fn solve(objects: &[Object], q: &Question) -> String {
        let words: Vec<&str> = q.text.trim_end_matches('?').split(' ').collect();
        let has = |s: &str, c: &str| objects.iter().find(|o| o.shape == s && o.colour == c);
        match q.kind {
            "colour" => objects.iter().find(|o| o.shape == words[4]).unwrap().colour.to_string(),
            "cell" => {
                let row = ROW_NAMES.iter().position(|r| *r == words[5]).unwrap();
                let col = COL_NAMES.iter().position(|c| *c == words[7]).unwrap();
                objects.iter().find(|o| o.row == row && o.col == col).unwrap().shape.to_string()
            }
            "count" => has(words[3].trim_end_matches('s'), words[2]).unwrap().count.to_string(),
            "total" => {
                let s = words[2].trim_end_matches('s');
                objects.iter().filter(|o| o.shape == s).map(|o| o.count).sum::<u32>().to_string()
            }
            "colour-count" => objects.iter().filter(|o| o.colour == words[5]).count().to_string(),
            "right-of" => {
                let o = has(words[7], words[6]).unwrap();
                objects.iter().find(|p| p.row == o.row && p.col == o.col + 1).unwrap().desc()
            }
            "missing" => {
                q.options.iter().find(|d| !objects.iter().any(|o| o.desc() == **d)).unwrap().clone()
            }
            k => panic!("unknown kind {k}"),
        }
    }

    #[test]
    fn every_question_has_one_right_answer_among_four() {
        for seed in 0..300 {
            let s = MemorySceneQuiz::setup(&team(1), seed).unwrap();
            assert_eq!(s.objects.len(), OBJECTS);
            assert_eq!(s.questions.len(), QUESTIONS, "seed {seed}");
            for q in &s.questions {
                assert_eq!(q.options.len(), 4);
                let mut o = q.options.clone();
                o.sort();
                o.dedup();
                assert_eq!(o.len(), 4, "{q:?}");
                assert_eq!(q.options[q.answer], solve(&s.objects, q), "seed {seed}: {q:?}");
            }
        }
    }

    #[test]
    fn questions_go_round_the_team_and_a_wrong_answer_ends_it() {
        let mut s = MemorySceneQuiz::setup(&team(3), 4).unwrap();
        for p in ["p1", "p2", "p3"] {
            MemorySceneQuiz::apply_move(&mut s, p, Move::Ready).unwrap();
        }
        assert_eq!(s.phase, Phase::Questions);
        assert!(MemorySceneQuiz::view_for(&s, "p1").objects.is_none(), "scene hidden");
        let right = |s: &State| s.questions[s.given.len()].answer;
        let choice = right(&s);
        MemorySceneQuiz::apply_move(&mut s, "p1", Move::Answer { choice }).unwrap();
        assert!(MemorySceneQuiz::apply_move(&mut s, "p1", Move::Answer { choice: 0 }).is_err());
        let choice = right(&s);
        MemorySceneQuiz::apply_move(&mut s, "p2", Move::Answer { choice }).unwrap();
        let wrong = (right(&s) + 1) % 4;
        MemorySceneQuiz::apply_move(&mut s, "p3", Move::Answer { choice: wrong }).unwrap();
        assert_eq!(s.phase, Phase::Over);
        assert!(s.failed);
        assert!(MemorySceneQuiz::results(&s).iter().all(|r| r.points == 2 * POINTS && r.rank == 1));
    }

    fn all_ready(s: &mut State) {
        for p in team(s.players.len()) {
            MemorySceneQuiz::apply_move(s, &p, Move::Ready).unwrap();
        }
    }

    #[test]
    fn study_and_questions_wait_for_the_team() {
        let mut s = MemorySceneQuiz::setup(&team(2), 9).unwrap();
        MemorySceneQuiz::tick(&mut s, 3_600_000);
        assert_eq!(s.phase, Phase::Study);
        all_ready(&mut s);
        MemorySceneQuiz::tick(&mut s, 3_600_000);
        assert_eq!((s.phase, s.given.len()), (Phase::Questions, 0));
        assert_eq!(MemorySceneQuiz::to_move(&s), vec!["p1".to_string()]);
    }

    #[test]
    fn all_ten_right_is_a_thousand() {
        let mut s = MemorySceneQuiz::setup(&team(1), 3).unwrap();
        MemorySceneQuiz::apply_move(&mut s, "p1", Move::Ready).unwrap();
        while s.phase == Phase::Questions {
            let choice = s.questions[s.given.len()].answer;
            MemorySceneQuiz::apply_move(&mut s, "p1", Move::Answer { choice }).unwrap();
        }
        assert_eq!(MemorySceneQuiz::results(&s)[0].points, 1000);
    }

    #[test]
    fn thousand_bot_games() {
        for seats in 1..=4 {
            bot_games_with::<MemorySceneQuiz>(1000, seats, 2000);
        }
    }

    #[test]
    fn a_perfect_quiz_is_a_replay_moment() {
        let mut s = MemorySceneQuiz::setup(&team(4), 3).unwrap();
        all_ready(&mut s);
        for i in 0..QUESTIONS {
            assert!(MemorySceneQuiz::moments(&s, "p1").is_empty());
            let choice = s.questions[i].answer;
            MemorySceneQuiz::apply_move(&mut s, &format!("p{}", i % 4 + 1), Move::Answer { choice }).unwrap();
        }
        // The tenth question went to the second seat.
        let m = MemorySceneQuiz::moments(&s, "p1");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p2", "Question 10 right: a perfect 1000"));
        // A failed quiz is no moment.
        let mut s = MemorySceneQuiz::setup(&team(4), 3).unwrap();
        all_ready(&mut s);
        let wrong = (s.questions[0].answer + 1) % 4;
        MemorySceneQuiz::apply_move(&mut s, "p1", Move::Answer { choice: wrong }).unwrap();
        assert_eq!(s.phase, Phase::Over);
        assert!(MemorySceneQuiz::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<MemorySceneQuiz>(300, 4, 2000);
        // Bots are right 95% of the time, so a perfect quiz comes in about 0.95^10 ≈ 60% of games.
        assert!(stats.with_any * 5 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 10 <= stats.games * 8, "too common: {stats:?}");
        assert!(stats.most <= 1, "too many in one game: {stats:?}");
    }
}
