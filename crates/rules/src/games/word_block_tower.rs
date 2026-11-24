//! Word Block Tower (Team mini game, 1–4 players). Rules page: docs/games/word-block-tower.md
//!
//! A pile of letter blocks made from one secret topic word per player. In
//! turn, every player builds one topic word from the pile; the last word must
//! use every block left.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult};
use crate::rng::Rng;

pub const ROUNDS: usize = 3;
pub const ROUND_POINTS: i64 = 300;
pub const TAKE_BACK_COST: i64 = 30;
pub const MIN_ROUND_POINTS: i64 = 50;
/// Most blocks in a pile for several players.
const MAX_BLOCKS: usize = 20;

pub const TOPICS: [(&str, &[&str]); 6] = [
    (
        "Animals",
        &[
            "CAT", "DOG", "COW", "PIG", "HEN", "FOX", "OWL", "BEE", "ANT", "RAT", "BAT", "YAK", "EEL", "ELK", "APE", "EMU", "GOAT", "BEAR",
            "LION", "WOLF", "DEER", "FROG", "DUCK", "SEAL", "CRAB", "MOLE", "MULE", "HARE", "LAMB", "TOAD", "SWAN", "MOTH", "TIGER", "ZEBRA",
            "HORSE", "SHEEP", "MOUSE", "OTTER", "CAMEL", "EAGLE", "SHARK", "WHALE", "SNAKE", "LLAMA", "HIPPO", "PANDA", "RABBIT", "DONKEY",
            "MONKEY", "TURTLE", "PARROT", "BEAVER",
        ],
    ),
    (
        "Fruits",
        &[
            "FIG", "LIME", "PEAR", "PLUM", "KIWI", "DATE", "APPLE", "MANGO", "LEMON", "GRAPE", "PEACH", "MELON", "GUAVA", "OLIVE", "BERRY",
            "CHERRY", "BANANA", "ORANGE", "PAPAYA", "QUINCE", "TOMATO", "LYCHEE", "DAMSON", "APRICOT",
        ],
    ),
    (
        "Colours",
        &[
            "RED", "TAN", "BLUE", "PINK", "GOLD", "GREY", "TEAL", "NAVY", "RUBY", "ROSE", "LIME", "JADE", "WHITE", "BLACK", "GREEN", "BROWN",
            "CORAL", "IVORY", "OLIVE", "LILAC", "AMBER", "CREAM", "BEIGE", "MAUVE", "PEACH", "YELLOW", "ORANGE", "PURPLE", "SILVER", "INDIGO",
            "VIOLET", "MAROON", "SCARLET", "CRIMSON",
        ],
    ),
    (
        "Body",
        &[
            "ARM", "LEG", "EAR", "EYE", "TOE", "LIP", "HIP", "JAW", "RIB", "GUM", "HAND", "FOOT", "KNEE", "NOSE", "CHIN", "HEEL", "NECK",
            "SHIN", "BACK", "HEAD", "SKIN", "NAIL", "LUNG", "WRIST", "ELBOW", "THUMB", "ANKLE", "TOOTH", "CHEEK", "BRAIN", "HEART", "LIVER",
            "SPINE", "TONGUE", "FINGER", "STOMACH",
        ],
    ),
    (
        "Kitchen",
        &[
            "POT", "PAN", "CUP", "JUG", "MUG", "LID", "TIN", "JAR", "WOK", "BOWL", "FORK", "DISH", "OVEN", "SINK", "TRAY", "LADLE", "KNIFE",
            "SPOON", "PLATE", "TONGS", "SIEVE", "WHISK", "APRON", "GRATER", "KETTLE", "FRIDGE", "TOASTER", "SKILLET",
        ],
    ),
    (
        "Weather",
        &[
            "FOG", "SUN", "DEW", "ICE", "HAIL", "RAIN", "SNOW", "WIND", "MIST", "HEAT", "COLD", "GALE", "SMOG", "HAZE", "STORM", "CLOUD",
            "FROST", "SLEET", "FLOOD", "SUNNY", "BREEZE", "THUNDER", "DRIZZLE", "RAINBOW", "TORNADO",
        ],
    ),
];

type Letters = [u8; 26];

fn letters(word: &str) -> Letters {
    let mut l = [0; 26];
    for b in word.bytes() {
        l[(b - b'A') as usize] += 1;
    }
    l
}

fn fits(word: &Letters, pile: &Letters) -> bool {
    word.iter().zip(pile).all(|(w, p)| w <= p)
}

fn minus(pile: &Letters, word: &Letters) -> Letters {
    let mut out = *pile;
    for i in 0..26 {
        out[i] -= word[i];
    }
    out
}

/// Can `pile` be split exactly into `k` different words of `list`, none of them in `used`?
pub fn can_finish(pile: &Letters, k: usize, list: &[&str], used: &mut Vec<String>) -> bool {
    if k == 0 {
        return pile.iter().all(|&c| c == 0);
    }
    let left: usize = pile.iter().map(|&c| c as usize).sum();
    for w in list {
        if used.iter().any(|u| u == w) {
            continue;
        }
        let wl = letters(w);
        if !fits(&wl, pile) || (k == 1 && w.len() != left) {
            continue;
        }
        used.push(w.to_string());
        let ok = can_finish(&minus(pile, &wl), k - 1, list, used);
        used.pop();
        if ok {
            return true;
        }
    }
    false
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub letter: char,
    /// The seat whose word uses this block.
    pub used_by: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundResult {
    pub topic: &'static str,
    pub words: Vec<Option<String>>,
    pub take_backs: u32,
    pub points: i64,
}

#[derive(Clone, Debug)]
pub struct State {
    pub players: Vec<String>,
    pub topics: Vec<usize>,
    pub round: usize,
    /// The secret words the pile was made from.
    pub targets: Vec<String>,
    pub blocks: Vec<Block>,
    pub words: Vec<Option<String>>,
    /// Seats in building order this round, and how many have built.
    pub order: Vec<usize>,
    pub built: usize,
    pub take_backs: u32,
    pub results: Vec<RoundResult>,
    pub over: bool,
    seed: u64,
    /// Replay moments: a seven-letter word.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Move {
    Word { word: String },
    TakeBack,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub players: Vec<String>,
    pub you: Option<usize>,
    pub round: usize,
    pub rounds: usize,
    pub topic: &'static str,
    pub blocks: Vec<Block>,
    pub words: Vec<Option<String>>,
    pub order: Vec<usize>,
    pub turn: Option<usize>,
    pub your_turn: bool,
    pub last_word: bool,
    pub can_take_back: bool,
    pub take_backs: u32,
    pub results: Vec<RoundResult>,
    pub score: i64,
    pub over: bool,
}

impl State {
    fn seat(&self, player: &str) -> Option<usize> {
        self.players.iter().position(|p| p == player)
    }

    pub fn list(&self) -> &'static [&'static str] {
        TOPICS[self.topics[self.round.min(ROUNDS - 1)]].1
    }

    pub fn turn(&self) -> Option<usize> {
        (!self.over).then(|| self.order[self.built])
    }

    fn pile(&self) -> Letters {
        let mut l = [0; 26];
        for b in self.blocks.iter().filter(|b| b.used_by.is_none()) {
            l[(b.letter as u8 - b'A') as usize] += 1;
        }
        l
    }

    fn used_words(&self) -> Vec<String> {
        self.words.iter().flatten().cloned().collect()
    }

    pub fn score(&self) -> i64 {
        self.results.iter().map(|r| r.points).sum()
    }

    fn start_round(&mut self) {
        let n = self.players.len();
        let mut rng = Rng::new(self.seed.wrapping_add(self.round as u64 * 7_777_777));
        let list = self.list();
        self.targets = loop {
            let mut words: Vec<&str> = list.to_vec();
            rng.shuffle(&mut words);
            let pick: Vec<String> = if n == 1 {
                words.iter().filter(|w| w.len() >= 5).take(1).map(|w| w.to_string()).collect()
            } else {
                words.iter().take(n).map(|w| w.to_string()).collect()
            };
            if pick.len() == n && pick.iter().map(String::len).sum::<usize>() <= MAX_BLOCKS {
                break pick;
            }
        };
        let mut letters: Vec<char> = self.targets.iter().flat_map(|w| w.chars()).collect();
        rng.shuffle(&mut letters);
        self.blocks = letters.into_iter().map(|letter| Block { letter, used_by: None }).collect();
        self.words = vec![None; n];
        self.order = (0..n).map(|k| (k + self.round) % n).collect();
        self.built = 0;
        self.take_backs = 0;
    }

    fn check_word(&self, word: &str) -> Result<Letters, String> {
        let topic = TOPICS[self.topics[self.round]].0;
        if !self.list().contains(&word) {
            return Err(format!("{word} is not on our {topic} list"));
        }
        if self.used_words().iter().any(|w| w == word) {
            return Err(format!("{word} is already built"));
        }
        let wl = letters(word);
        let pile = self.pile();
        if !fits(&wl, &pile) {
            return Err("Not enough blocks for that word".into());
        }
        let last = self.built + 1 == self.order.len();
        if last && word.len() != pile.iter().map(|&c| c as usize).sum::<usize>() {
            return Err("The last word must use every block left".into());
        }
        Ok(wl)
    }

    fn build(&mut self, seat: usize, word: String) {
        // A replay moment (built words are public): the longest words there are.
        if word.len() == 7 {
            self.moments.push(&self.players[seat], format!("Seven letters in one word: {word}"));
        }
        for c in word.chars() {
            if let Some(b) = self.blocks.iter_mut().find(|b| b.used_by.is_none() && b.letter == c) {
                b.used_by = Some(seat);
            }
        }
        self.words[seat] = Some(word);
        self.built += 1;
        if self.built == self.order.len() {
            self.finish_round();
        }
    }

    fn take_back(&mut self) -> Result<(), String> {
        if self.built == 0 {
            return Err("Nothing to take back".into());
        }
        self.built -= 1;
        let seat = self.order[self.built];
        self.words[seat] = None;
        for b in self.blocks.iter_mut().filter(|b| b.used_by == Some(seat)) {
            b.used_by = None;
        }
        self.take_backs += 1;
        Ok(())
    }

    fn finish_round(&mut self) {
        let points = (ROUND_POINTS - TAKE_BACK_COST * i64::from(self.take_backs)).max(MIN_ROUND_POINTS);
        self.results.push(RoundResult {
            topic: TOPICS[self.topics[self.round]].0,
            words: self.words.clone(),
            take_backs: self.take_backs,
            points,
        });
        self.round += 1;
        if self.round >= ROUNDS {
            self.over = true;
        } else {
            self.start_round();
        }
    }

    /// The bot's move for the player whose turn it is.
    fn bot_choice(&self, seed: u64) -> Move {
        let mut rng = Rng::new(seed);
        let pile = self.pile();
        let left = self.order.len() - self.built;
        let mut used = self.used_words();
        let mut good: Vec<&str> = Vec::new();
        for w in self.list() {
            if used.iter().any(|u| u == w) || self.check_word(w).is_err() {
                continue;
            }
            used.push(w.to_string());
            if can_finish(&minus(&pile, &letters(w)), left - 1, self.list(), &mut used) {
                good.push(w);
            }
            used.pop();
        }
        if good.is_empty() {
            Move::TakeBack
        } else {
            Move::Word { word: rng.pick(&good).to_string() }
        }
    }
}

pub struct WordBlockTower;

impl Game for WordBlockTower {
    type State = State;
    type Move = Move;
    type View = View;

    fn info() -> GameInfo {
        GameInfo {
            id: "word-block-tower",
            kind: "mini-team",
            difficulty: "easy",
            tags: &["words", "alliance"],
            min_players: 1,
            max_players: 4,
            name: "Word Block Tower",
            short: "Use every letter block: one topic word per player, built in turn.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<State, String> {
        if !(1..=4).contains(&players.len()) {
            return Err("Word Block Tower is for 1–4 players".into());
        }
        let mut rng = Rng::new(seed);
        let mut topics: Vec<usize> = (0..TOPICS.len()).collect();
        rng.shuffle(&mut topics);
        topics.truncate(ROUNDS);
        let mut s = State {
            players: players.to_vec(),
            topics,
            round: 0,
            targets: Vec::new(),
            blocks: Vec::new(),
            words: Vec::new(),
            order: Vec::new(),
            built: 0,
            take_backs: 0,
            results: Vec::new(),
            over: false,
            seed,
            moments: Moments::default(),
        };
        s.start_round();
        Ok(s)
    }

    fn apply_move(s: &mut State, player: &str, mv: Move) -> Result<(), String> {
        if s.over {
            return Err("Game is over".into());
        }
        let seat = s.seat(player).ok_or("Not in this game")?;
        if s.turn() != Some(seat) {
            return Err("Not your turn".into());
        }
        match mv {
            Move::Word { word } => {
                let word = word.trim().to_ascii_uppercase();
                s.check_word(&word)?;
                s.build(seat, word);
                Ok(())
            }
            Move::TakeBack => s.take_back(),
        }
    }

    fn view_for(s: &State, player: &str) -> View {
        let you = s.seat(player);
        let turn = s.turn();
        View {
            players: s.players.clone(),
            you,
            round: (s.round + 1).min(ROUNDS),
            rounds: ROUNDS,
            topic: TOPICS[s.topics[s.round.min(ROUNDS - 1)]].0,
            blocks: s.blocks.clone(),
            words: s.words.clone(),
            order: s.order.clone(),
            turn,
            your_turn: turn.is_some() && turn == you,
            last_word: s.built + 1 == s.order.len(),
            can_take_back: s.built > 0,
            take_backs: s.take_backs,
            results: s.results.clone(),
            score: s.score(),
            over: s.over,
        }
    }

    fn is_over(s: &State) -> bool {
        s.over
    }

    fn results(s: &State) -> Vec<PlayerResult> {
        let pts = s.score();
        s.players.iter().map(|p| PlayerResult { player_id: p.clone(), rank: 1, points: pts }).collect()
    }

    fn to_move(s: &State) -> Vec<String> {
        s.turn().map(|k| vec![s.players[k].clone()]).unwrap_or_default()
    }

    fn bot_move(s: &State, _player: &str, seed: u64) -> Move {
        if s.over { Move::TakeBack } else { s.bot_choice(seed) }
    }

    fn moments(s: &State, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }
}

pub type Rules = WordBlockTower;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn team(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn lists_are_clean() {
        for (topic, list) in TOPICS {
            for w in list {
                assert!((3..=7).contains(&w.len()) && w.bytes().all(|b| b.is_ascii_uppercase()), "{topic}: {w}");
                assert_eq!(list.iter().filter(|x| *x == w).count(), 1, "{topic}: {w} twice");
            }
        }
    }

    #[test]
    fn the_pile_is_the_secret_words() {
        for seed in 0..50 {
            for n in 1..=4 {
                let s = WordBlockTower::setup(&team(n), seed).unwrap();
                let total: usize = s.targets.iter().map(String::len).sum();
                assert_eq!(s.blocks.len(), total);
                assert!(can_finish(&s.pile(), n, s.list(), &mut Vec::new()));
                assert_eq!(s.order[0], 0);
            }
        }
    }

    #[test]
    fn words_must_fit_the_topic_and_the_pile() {
        let mut s = WordBlockTower::setup(&team(2), 1).unwrap();
        let first = s.targets[0].clone();
        assert!(WordBlockTower::apply_move(&mut s, "p2", Move::Word { word: first.clone() }).is_err());
        assert!(WordBlockTower::apply_move(&mut s, "p1", Move::Word { word: "XYZZY".into() }).is_err());
        WordBlockTower::apply_move(&mut s, "p1", Move::Word { word: first.to_lowercase() }).unwrap();
        assert_eq!(s.blocks.iter().filter(|b| b.used_by == Some(0)).count(), first.len());
        // Take back, build again, then the last word must use every block.
        WordBlockTower::apply_move(&mut s, "p2", Move::TakeBack).unwrap();
        assert_eq!(s.take_backs, 1);
        assert_eq!(s.turn(), Some(0));
        WordBlockTower::apply_move(&mut s, "p1", Move::Word { word: first }).unwrap();
        let second = s.targets[1].clone();
        WordBlockTower::apply_move(&mut s, "p2", Move::Word { word: second }).unwrap();
        assert_eq!(s.round, 1);
        assert_eq!(s.results[0].points, ROUND_POINTS - TAKE_BACK_COST);
        // Round two starts one seat later.
        assert_eq!(s.turn(), Some(1));
    }

    #[test]
    fn the_last_word_must_empty_the_pile() {
        let mut s = WordBlockTower::setup(&team(1), 2).unwrap();
        let target = s.targets[0].clone();
        let short = s.list().iter().find(|w| w.len() < target.len() && fits(&letters(w), &s.pile()));
        if let Some(w) = short {
            assert!(WordBlockTower::apply_move(&mut s, "p1", Move::Word { word: w.to_string() }).is_err());
        }
        WordBlockTower::apply_move(&mut s, "p1", Move::Word { word: target }).unwrap();
        assert_eq!(s.round, 1);
    }

    #[test]
    fn bots_always_finish() {
        for seed in 0..40 {
            let mut s = WordBlockTower::setup(&team(4), seed).unwrap();
            let mut steps = 0;
            while !s.over {
                let p = WordBlockTower::to_move(&s)[0].clone();
                let mv = WordBlockTower::bot_move(&s, &p, seed + steps);
                WordBlockTower::apply_move(&mut s, &p, mv).unwrap();
                steps += 1;
            }
            assert_eq!(s.score(), 900, "seed {seed}");
        }
    }

    #[test]
    fn a_turn_waits_for_the_move() {
        let mut s = WordBlockTower::setup(&team(2), 3).unwrap();
        WordBlockTower::tick(&mut s, 3_600_000);
        assert_eq!((s.built, s.turn()), (0, Some(0)));
    }

    #[test]
    fn thousand_bot_games() {
        for seats in 1..=4 {
            bot_games_with::<WordBlockTower>(1000, seats, 3000);
        }
    }

    #[test]
    fn a_seven_letter_word_is_a_replay_moment() {
        let mut s = (0..)
            .map(|seed| WordBlockTower::setup(&team(1), seed).unwrap())
            .find(|s| s.targets[0].len() == 7)
            .unwrap();
        let word = s.targets[0].clone();
        WordBlockTower::apply_move(&mut s, "p1", Move::Word { word: word.clone() }).unwrap();
        let m = WordBlockTower::moments(&s, "p1");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.clone()), ("p1", format!("Seven letters in one word: {word}")));
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<WordBlockTower>(300, 4, 3_000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 5 <= stats.games * 4, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}
