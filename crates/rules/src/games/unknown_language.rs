//! Unknown Language (exile, solo). Rules page: docs/games/unknown-language.md
//!
//! A picture script made from the seed: each word is a root glyph plus a mark
//! for thing / action / quality; the sentence order and where qualities stand
//! are seeded too. Learn it from examples, then read and write it (9
//! multiple-choice questions in three stages).

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, speed_points};
use crate::rng::Rng;

/// Thing, action, quality of every root.
pub const ROOTS: [(&str, &str, &str); 16] = [
    ("guard", "guards", "watchful"),
    ("key", "unlocks", "open"),
    ("fire", "burns", "hot"),
    ("water", "washes", "wet"),
    ("eye", "sees", "clear"),
    ("hand", "takes", "quick"),
    ("stone", "breaks", "hard"),
    ("lamp", "lights", "bright"),
    ("bread", "feeds", "full"),
    ("shadow", "hides", "dark"),
    ("door", "closes", "shut"),
    ("singer", "calls", "loud"),
    ("gold", "buys", "rich"),
    ("dog", "bites", "fierce"),
    ("rope", "ties", "tight"),
    ("ice", "freezes", "cold"),
];

pub const QUESTION_MS: i64 = 75_000;
pub const STAGES: usize = 3;
pub const PER_STAGE: usize = 3;
/// Examples shown in stage 1, and from stage 2 on.
pub const EXAMPLES: [usize; STAGES] = [4, 7, 7];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pos {
    Noun = 0,
    Verb = 1,
    Adj = 2,
}

pub type Word = (usize, Pos);

pub fn english_word((root, pos): Word) -> &'static str {
    let (n, v, a) = ROOTS[root];
    match pos {
        Pos::Noun => n,
        Pos::Verb => v,
        Pos::Adj => a,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Np {
    pub noun: usize,
    pub adj: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sentence {
    pub subj: Np,
    pub verb: usize,
    pub obj: Np,
}

impl Sentence {
    fn swapped(self) -> Sentence {
        Sentence { subj: self.obj, obj: self.subj, ..self }
    }
}

#[derive(Clone, Debug)]
pub struct Grammar {
    /// Slots in order: 0 subject, 1 action, 2 object.
    pub order: [u8; 3],
    pub adj_before: bool,
    /// Visual mark (0 bar under, 1 dot above, 2 ring left) of thing, action, quality.
    pub marks: [u8; 3],
}

impl Grammar {
    pub fn script_in(&self, order: [u8; 3], s: &Sentence) -> Vec<Word> {
        let np = |np: &Np| {
            let noun = (np.noun, Pos::Noun);
            match np.adj.map(|a| (a, Pos::Adj)) {
                Some(adj) if self.adj_before => vec![adj, noun],
                Some(adj) => vec![noun, adj],
                None => vec![noun],
            }
        };
        order
            .iter()
            .flat_map(|slot| match slot {
                0 => np(&s.subj),
                1 => vec![(s.verb, Pos::Verb)],
                _ => np(&s.obj),
            })
            .collect()
    }

    pub fn script(&self, s: &Sentence) -> Vec<Word> {
        self.script_in(self.order, s)
    }
}

/// "The hot dog bites the guard." `adj_text` can misread a quality (for wrong options).
fn english_with(s: &Sentence, adj_text: &dyn Fn(usize) -> &'static str) -> String {
    let np = |np: &Np| match np.adj {
        Some(a) => format!("{} {}", adj_text(a), ROOTS[np.noun].0),
        None => ROOTS[np.noun].0.to_string(),
    };
    format!("The {} {} the {}.", np(&s.subj), ROOTS[s.verb].1, np(&s.obj))
}

pub fn english(s: &Sentence) -> String {
    english_with(s, &|a| ROOTS[a].2)
}

/// 20 strokes between neighbouring points of a 3 × 3 grid.
fn all_strokes() -> Vec<[u8; 4]> {
    let mut out = Vec::new();
    for y in 0..3u8 {
        for x in 0..3u8 {
            if x < 2 {
                out.push([x, y, x + 1, y]);
            }
            if y < 2 {
                out.push([x, y, x, y + 1]);
            }
            if x < 2 && y < 2 {
                out.push([x, y, x + 1, y + 1]);
                out.push([x + 1, y, x, y + 1]);
            }
        }
    }
    out
}

/// One glyph of three strokes per root, all different.
pub fn make_glyphs(rng: &mut Rng) -> Vec<Vec<[u8; 4]>> {
    let strokes = all_strokes();
    let mut glyphs: Vec<Vec<[u8; 4]>> = Vec::new();
    while glyphs.len() < ROOTS.len() {
        let mut pick = strokes.clone();
        rng.shuffle(&mut pick);
        let mut g = pick[..3].to_vec();
        g.sort();
        if !glyphs.contains(&g) {
            glyphs.push(g);
        }
    }
    glyphs
}

#[derive(Clone, Debug)]
pub enum Question {
    Read { sentence: Sentence, options: Vec<String> },
    Write { sentence: Sentence, options: Vec<Vec<Word>> },
    Word { word: Word, options: Vec<String> },
}

/// The right option first, then up to three distinct wrong ones; shuffled.
/// Returns the options and where the right one went.
fn shuffle_options<T: PartialEq + Clone>(right: T, wrong: Vec<T>, rng: &mut Rng) -> (Vec<T>, usize) {
    let mut opts = vec![right.clone()];
    for w in wrong {
        if opts.len() < 4 && !opts.contains(&w) {
            opts.push(w);
        }
    }
    rng.shuffle(&mut opts);
    let at = opts.iter().position(|o| *o == right).expect("the right option is there");
    (opts, at)
}

#[derive(Clone, Debug)]
pub struct Course {
    pub grammar: Grammar,
    pub glyphs: Vec<Vec<[u8; 4]>>,
    pub examples: Vec<Sentence>,
    pub questions: Vec<(Question, usize)>,
}

pub fn make_course(rng: &mut Rng) -> Course {
    let mut order = [0u8, 1, 2];
    rng.shuffle(&mut order);
    let mut marks = [0u8, 1, 2];
    rng.shuffle(&mut marks);
    let grammar = Grammar { order, adj_before: rng.next_f64() < 0.5, marks };
    let glyphs = make_glyphs(rng);
    let mut roots: Vec<usize> = (0..ROOTS.len()).collect();
    rng.shuffle(&mut roots);
    let (n, v, a) = (&roots[0..4], &roots[4..6], &roots[6..8]);
    let np = |noun: usize, adj: Option<usize>| Np { noun, adj };
    let examples = vec![
        Sentence { subj: np(n[0], None), verb: v[0], obj: np(n[1], None) },
        Sentence { subj: np(n[2], None), verb: v[1], obj: np(n[3], None) },
        Sentence { subj: np(n[1], None), verb: v[1], obj: np(n[2], None) },
        Sentence { subj: np(n[3], None), verb: v[0], obj: np(n[0], None) },
        Sentence { subj: np(n[0], Some(a[0])), verb: v[1], obj: np(n[2], None) },
        Sentence { subj: np(n[3], None), verb: v[0], obj: np(n[1], Some(a[1])) },
        Sentence { subj: np(n[2], Some(a[1])), verb: v[0], obj: np(n[3], Some(a[0])) },
    ];

    // A new sentence from the known words (qualities from stage 2 on).
    let new_sentence = |rng: &mut Rng, adjs: &[usize], want_adj: bool| loop {
        let mut nouns = n.to_vec();
        rng.shuffle(&mut nouns);
        let mut pick_adj = || (!adjs.is_empty() && rng.next_f64() < 0.5).then(|| *rng.pick(adjs));
        let mut s = Sentence { subj: np(nouns[0], pick_adj()), verb: 0, obj: np(nouns[1], pick_adj()) };
        s.verb = *rng.pick(v);
        if want_adj && s.subj.adj.is_none() && s.obj.adj.is_none() {
            s.subj.adj = Some(*rng.pick(adjs));
        }
        if !examples.contains(&s) {
            break s;
        }
    };
    // Near misses of a sentence: swapped, another action, another thing, the quality moved.
    let near_misses = |s: &Sentence, rng: &mut Rng| {
        let mut out = vec![s.swapped()];
        let other_verb = if s.verb == v[0] { v[1] } else { v[0] };
        out.push(Sentence { verb: other_verb, ..*s });
        let free: Vec<usize> = n.iter().copied().filter(|&x| x != s.subj.noun && x != s.obj.noun).collect();
        let (mut subj_changed, mut obj_changed) = (*s, *s);
        subj_changed.subj.noun = *rng.pick(&free);
        obj_changed.obj.noun = *rng.pick(&free);
        if rng.next_f64() < 0.5 {
            out.extend([subj_changed, obj_changed]);
        } else {
            out.extend([obj_changed, subj_changed]);
        }
        if s.subj.adj.is_some() != s.obj.adj.is_some() {
            let moved = Sentence {
                subj: Np { adj: s.obj.adj, ..s.subj },
                obj: Np { adj: s.subj.adj, ..s.obj },
                ..*s
            };
            out.insert(1, moved);
        }
        // Spares, should some of the above be examples: two changes at once.
        out.push(Sentence { verb: other_verb, ..s.swapped() });
        out.push(Sentence { verb: other_verb, ..subj_changed });
        out.push(Sentence { verb: other_verb, ..obj_changed });
        // A wrong option never repeats an example word for word.
        out.retain(|m| !examples.contains(m));
        out
    };
    let read = |s: Sentence, rng: &mut Rng| {
        let wrong = near_misses(&s, rng).iter().map(english).collect();
        let (options, at) = shuffle_options(english(&s), wrong, rng);
        (Question::Read { sentence: s, options }, at)
    };
    let write = |s: Sentence, rng: &mut Rng| {
        let mut wrong: Vec<Vec<Word>> = Vec::new();
        // The right words in another order first, then the near misses.
        let mut other = grammar.order;
        while other == grammar.order {
            rng.shuffle(&mut other);
        }
        wrong.push(grammar.script_in(other, &s));
        wrong.extend(near_misses(&s, rng).iter().map(|m| grammar.script(m)));
        let (options, at) = shuffle_options(grammar.script(&s), wrong, rng);
        (Question::Write { sentence: s, options }, at)
    };

    let mut questions = Vec::new();
    // Stage 1: things and actions.
    for kind in [0, 1, 0] {
        let s = new_sentence(rng, &[], false);
        questions.push(if kind == 0 { read(s, rng) } else { write(s, rng) });
    }
    // Stage 2: with qualities.
    for kind in [1, 0, 1] {
        let s = new_sentence(rng, a, true);
        questions.push(if kind == 0 { read(s, rng) } else { write(s, rng) });
    }
    // Stage 3: known roots with a new mark. Two single words, then a sentence
    // with a thing's root used as a quality.
    let fresh = |root: usize| {
        if n.contains(&root) {
            [Pos::Verb, Pos::Adj]
        } else if v.contains(&root) {
            [Pos::Noun, Pos::Adj]
        } else {
            [Pos::Noun, Pos::Verb]
        }
    };
    let known = |root: usize| {
        if n.contains(&root) {
            Pos::Noun
        } else if v.contains(&root) {
            Pos::Verb
        } else {
            Pos::Adj
        }
    };
    let mut seen: Vec<usize> = roots[0..8].to_vec();
    rng.shuffle(&mut seen);
    for &root in &seen[..2] {
        let pos = *rng.pick(&fresh(root));
        let mut others: Vec<usize> = (0..ROOTS.len()).filter(|&r| r != root).collect();
        rng.shuffle(&mut others);
        let mut wrong = vec![english_word((root, known(root))).to_string()];
        wrong.extend(others.iter().map(|&r| english_word((r, pos)).to_string()));
        let (options, at) = shuffle_options(english_word((root, pos)).to_string(), wrong, rng);
        questions.push((Question::Word { word: (root, pos), options }, at));
    }
    let quality_root = *rng.pick(&n.iter().copied().filter(|r| !seen[..2].contains(r)).collect::<Vec<_>>());
    let mut s = loop {
        let s = new_sentence(rng, &[], false);
        if s.subj.noun != quality_root && s.obj.noun != quality_root {
            break s;
        }
    };
    if rng.next_f64() < 0.5 {
        s.subj.adj = Some(quality_root);
    } else {
        s.obj.adj = Some(quality_root);
    }
    let mut wrong = vec![english_with(&s, &|r| ROOTS[r].0)];
    wrong.extend(near_misses(&s, rng).iter().map(english));
    let (options, at) = shuffle_options(english(&s), wrong, rng);
    questions.push((Question::Read { sentence: s, options }, at));

    Course { grammar, glyphs, examples, questions }
}

// --- State, moves, view ----------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewWord {
    pub strokes: Vec<[u8; 4]>,
    pub mark: u8,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum QuestionView {
    Read { script: Vec<ViewWord>, options: Vec<String> },
    Write { english: String, options: Vec<Vec<ViewWord>> },
    Word { word: ViewWord, options: Vec<String> },
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleView {
    pub script: Vec<ViewWord>,
    pub english: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastView {
    pub question: QuestionView,
    pub given: Option<usize>,
    pub solution: usize,
    pub correct: bool,
    pub points: i64,
}

#[derive(Clone, Debug)]
pub struct LangState {
    pub player: String,
    pub course: Course,
    pub current: usize,
    pub score: i64,
    pub time_left_ms: i64,
    pub last: Option<LastView>,
    /// Right or wrong, per question answered.
    pub results: Vec<bool>,
    pub over: bool,
    /// Replay moments: all nine right; else stage 3 (the new marks) without a miss.
    pub moments: Moments,
}

impl LangState {
    fn word(&self, (root, pos): Word) -> ViewWord {
        ViewWord { strokes: self.course.glyphs[root].clone(), mark: self.course.grammar.marks[pos as usize] }
    }

    fn script(&self, words: &[Word]) -> Vec<ViewWord> {
        words.iter().map(|&w| self.word(w)).collect()
    }

    fn question_view(&self, q: &Question) -> QuestionView {
        match q {
            Question::Read { sentence, options } => {
                QuestionView::Read { script: self.script(&self.course.grammar.script(sentence)), options: options.clone() }
            }
            Question::Write { sentence, options } => {
                QuestionView::Write { english: english(sentence), options: options.iter().map(|o| self.script(o)).collect() }
            }
            Question::Word { word, options } => QuestionView::Word { word: self.word(*word), options: options.clone() },
        }
    }

    fn finish(&mut self, given: Option<usize>) {
        let (q, solution) = self.course.questions[self.current].clone();
        let correct = given == Some(solution);
        let points = if correct { speed_points(self.time_left_ms, QUESTION_MS) / 2 } else { 0 };
        self.score += points;
        self.results.push(correct);
        self.last = Some(LastView { question: self.question_view(&q), given, solution, correct, points });
        self.current += 1;
        self.over = self.current >= self.course.questions.len();
        self.time_left_ms = if self.over { 0 } else { QUESTION_MS };
        if self.over {
            let player = self.player.clone();
            if self.results.iter().all(|&r| r) {
                self.moments.push(&player, format!("Perfect run: all {} carvings read right", self.results.len()));
            } else if self.results[(STAGES - 1) * PER_STAGE..].iter().all(|&r| r) {
                self.moments.push(&player, "Stage 3 cracked without a miss");
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LangMove {
    Answer { option: usize },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangView {
    pub stage: usize,
    pub question: usize,
    pub questions: usize,
    pub examples: Vec<ExampleView>,
    pub current: Option<QuestionView>,
    pub last: Option<LastView>,
    pub results: Vec<bool>,
    pub score: i64,
    pub time_left_ms: i64,
    pub question_time_ms: i64,
    pub over: bool,
}

pub struct Rules;

impl Game for Rules {
    type State = LangState;
    type Move = LangMove;
    type View = LangView;

    fn info() -> GameInfo {
        GameInfo {
            id: "unknown-language",
            kind: "exile",
            difficulty: "hard",
            tags: &["words", "logic"],
            min_players: 1,
            max_players: 1,
            name: "Unknown Language",
            short: "Crack a picture script from a few translated carvings, then read and write it.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<LangState, String> {
        let [player] = players else { return Err("Unknown Language is a solo game".into()) };
        Ok(LangState {
            player: player.clone(),
            course: make_course(&mut Rng::new(seed)),
            current: 0,
            score: 0,
            time_left_ms: QUESTION_MS,
            last: None,
            results: Vec::new(),
            over: false,
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut LangState, player: &str, mv: LangMove) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        if player != s.player {
            return Err("Not your game".into());
        }
        let LangMove::Answer { option } = mv;
        if option >= 4 {
            return Err("Pick one of the four options".into());
        }
        s.finish(Some(option));
        Ok(())
    }

    fn tick(s: &mut LangState, ms: u64) {
        if s.over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms <= 0 {
            s.time_left_ms = 0;
            s.finish(None);
        }
    }

    fn view_for(s: &LangState, _player: &str) -> LangView {
        let stage = (s.current / PER_STAGE).min(STAGES - 1);
        LangView {
            stage: stage + 1,
            question: (s.current + 1).min(s.course.questions.len()),
            questions: s.course.questions.len(),
            examples: s.course.examples[..EXAMPLES[stage]]
                .iter()
                .map(|e| ExampleView { script: s.script(&s.course.grammar.script(e)), english: english(e) })
                .collect(),
            current: (!s.over).then(|| s.question_view(&s.course.questions[s.current].0)),
            last: s.last.clone(),
            results: s.results.clone(),
            score: s.score,
            time_left_ms: s.time_left_ms,
            question_time_ms: QUESTION_MS,
            over: s.over,
        }
    }

    fn is_over(s: &LangState) -> bool {
        s.over
    }

    fn results(s: &LangState) -> Vec<PlayerResult> {
        vec![PlayerResult { player_id: s.player.clone(), rank: 1, points: s.score }]
    }

    fn bot_move(s: &LangState, _player: &str, seed: u64) -> LangMove {
        let mut rng = Rng::new(seed);
        let solution = s.course.questions[s.current.min(s.course.questions.len() - 1)].1;
        LangMove::Answer { option: if rng.next_f64() < 0.75 { solution } else { rng.index(4) } }
    }

    fn moments(s: &LangState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub const READY: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games;

    const P: &str = "p1";

    #[test]
    fn courses_are_well_formed() {
        for seed in 0..500 {
            let c = make_course(&mut Rng::new(seed));
            assert_eq!(c.questions.len(), STAGES * PER_STAGE);
            let mut glyphs = c.glyphs.clone();
            glyphs.sort();
            glyphs.dedup();
            assert_eq!(glyphs.len(), ROOTS.len());
            for (q, at) in &c.questions {
                let n = match q {
                    Question::Read { options, .. } | Question::Word { options, .. } => {
                        let mut o = options.clone();
                        o.sort();
                        o.dedup();
                        o.len()
                    }
                    Question::Write { options, .. } => {
                        let mut o = options.clone();
                        o.sort_by_key(|w| format!("{w:?}"));
                        o.dedup();
                        o.len()
                    }
                };
                assert_eq!(n, 4, "seed {seed}: four different options");
                assert!(*at < 4);
            }
        }
    }

    #[test]
    fn questions_use_only_words_from_the_examples_until_stage_3() {
        for seed in 0..200 {
            let c = make_course(&mut Rng::new(seed));
            let seen: Vec<Word> = c.examples.iter().flat_map(|e| c.grammar.script(e)).collect();
            for (q, _) in &c.questions[..2 * PER_STAGE] {
                let (Question::Read { sentence, .. } | Question::Write { sentence, .. }) = q else { panic!("no words before stage 3") };
                assert!(c.grammar.script(sentence).iter().all(|w| seen.contains(w)), "seed {seed}");
            }
            // Stage 3: the root is known, the mark is new.
            for (q, at) in &c.questions[2 * PER_STAGE..2 * PER_STAGE + 2] {
                let Question::Word { word, options } = q else { panic!("word question") };
                assert!(!seen.contains(word) && seen.iter().any(|w| w.0 == word.0));
                assert_eq!(options[*at], english_word(*word));
            }
        }
    }

    #[test]
    fn script_follows_the_grammar() {
        let g = Grammar { order: [2, 1, 0], adj_before: false, marks: [0, 1, 2] };
        let s = Sentence { subj: Np { noun: 0, adj: Some(2) }, verb: 1, obj: Np { noun: 3, adj: None } };
        assert_eq!(g.script(&s), vec![(3, Pos::Noun), (1, Pos::Verb), (0, Pos::Noun), (2, Pos::Adj)]);
        assert_eq!(english(&s), "The hot guard unlocks the water.");
    }

    #[test]
    fn answers_score_and_time_runs_out() {
        let mut s = Rules::setup(&[P.into()], 3).unwrap();
        let right = s.course.questions[0].1;
        Rules::apply_move(&mut s, P, LangMove::Answer { option: right }).unwrap();
        assert_eq!(s.score, 100);
        assert!(Rules::apply_move(&mut s, P, LangMove::Answer { option: 4 }).is_err());
        Rules::tick(&mut s, QUESTION_MS as u64);
        assert_eq!(s.current, 2);
        assert!(!s.last.as_ref().unwrap().correct);
        let v = serde_json::to_value(Rules::view_for(&s, P)).unwrap();
        assert_eq!(v["examples"].as_array().unwrap().len(), 4);
        // The view never names the parts of speech.
        assert!(!v.to_string().contains("Noun"));
    }

    #[test]
    fn thousand_bot_games() {
        bot_games::<Rules>(1000, 2000);
    }

    /// Answers every question, right where `right[i]` says so.
    fn play(right: [bool; 9]) -> LangState {
        let mut s = Rules::setup(&[P.into()], 4).unwrap();
        for r in right {
            let solution = s.course.questions[s.current].1;
            Rules::apply_move(&mut s, P, LangMove::Answer { option: if r { solution } else { (solution + 1) % 4 } }).unwrap();
        }
        assert!(s.over);
        s
    }

    #[test]
    fn a_perfect_run_or_a_clean_stage_3_is_a_replay_moment() {
        let m = Rules::moments(&play([true; 9]), P);
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), (P, "Perfect run: all 9 carvings read right"));
        let m = Rules::moments(&play([false, true, true, true, true, true, true, true, true]), P);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].text, "Stage 3 cracked without a miss");
        assert!(Rules::moments(&play([true, true, true, true, true, true, true, true, false]), P).is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 1, 5_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 1, "too many in one game: {stats:?}");
    }
}
