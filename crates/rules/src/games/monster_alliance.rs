//! Monster Alliance (main game). Rules page: docs/games/monster-alliance.md
//!
//! Six monsters, one per round: bid points for an extra weapon, form
//! alliances of up to three by mutual choice, then play weapon cards in
//! secret. The strongest alliance that beats the monster shares its reward.

use serde::{Deserialize, Serialize};

use crate::game::{Game, GameInfo, Moment, Moments, PlayerResult, rank_by_score};
use crate::rng::Rng;

pub const ROUNDS: usize = 6;
pub const START_POINTS: i64 = 20;
pub const HAND: usize = 8;
pub const BID_MS: i64 = 15_000;
pub const ALLY_MS: i64 = 20_000;
pub const ATTACK_MS: i64 = 20_000;
const NAMES: [&str; 8] = ["Gloomtusk", "Rustjaw", "Mirebeast", "Thornback", "Hollow King", "Ashwing", "Sludgemaw", "Ironhide"];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Monster {
    pub name: &'static str,
    pub strength: i64,
    pub reward: i64,
    /// The extra weapon card auctioned this round.
    pub bonus: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Bid,
    Ally,
    Attack,
    Over,
}

#[derive(Clone, Debug)]
pub struct Fighter {
    pub id: String,
    pub points: i64,
    pub hand: Vec<i64>,
    pub bid: Option<i64>,
    pub ally: Option<Vec<usize>>,
    pub attack: Option<Vec<usize>>,
    /// Partners picked in the last alliance phase.
    pub picked: Vec<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outcome {
    pub members: Vec<usize>,
    pub played: Vec<Vec<i64>>,
    pub power: i64,
    pub won: bool,
    pub shares: Vec<i64>,
}

#[derive(Clone, Debug)]
pub struct MaState {
    pub fighters: Vec<Fighter>,
    pub monsters: Vec<Monster>,
    pub round: usize,
    pub phase: Phase,
    pub time_left_ms: i64,
    pub groups: Vec<Vec<usize>>,
    pub last: Vec<Outcome>,
    pub log: Vec<String>,
    /// Replay moments: a monster beaten alone, the whole reward to one player.
    pub moments: Moments,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MaMove {
    Bid { amount: i64 },
    Ally { partners: Vec<usize> },
    Attack { cards: Vec<usize> },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FighterView {
    pub index: usize,
    pub id: String,
    pub points: i64,
    pub cards: usize,
    pub done: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaView {
    pub phase: &'static str,
    pub round: usize,
    pub rounds: usize,
    pub time_left_ms: i64,
    pub phase_time_ms: i64,
    pub me: usize,
    pub monster: Option<Monster>,
    pub fighters: Vec<FighterView>,
    pub hand: Vec<i64>,
    pub acted: bool,
    pub groups: Vec<Vec<usize>>,
    pub last: Vec<Outcome>,
    pub log: Vec<String>,
    pub score: i64,
}

/// Points by rank: 1000 for first down to 100 for last.
fn rank_points(scores: &[(String, i64)]) -> Vec<PlayerResult> {
    let n = scores.len() as i64;
    rank_by_score(scores)
        .into_iter()
        .map(|mut r| {
            r.points = if n > 1 { 100 + 900 * (n - i64::from(r.rank)) / (n - 1) } else { 1000 };
            r
        })
        .collect()
}

/// Mutual choices: groups of three first, then pairs, then everyone alone.
pub fn form_groups(choices: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = choices.len();
    let mutual = |a: usize, b: usize| choices[a].contains(&b) && choices[b].contains(&a);
    let mut used = vec![false; n];
    let mut groups = Vec::new();
    for a in 0..n {
        for b in a + 1..n {
            for c in b + 1..n {
                if !used[a] && !used[b] && !used[c] && mutual(a, b) && mutual(a, c) && mutual(b, c) {
                    used[a] = true;
                    used[b] = true;
                    used[c] = true;
                    groups.push(vec![a, b, c]);
                }
            }
        }
    }
    for a in 0..n {
        for b in a + 1..n {
            if !used[a] && !used[b] && mutual(a, b) {
                used[a] = true;
                used[b] = true;
                groups.push(vec![a, b]);
            }
        }
    }
    for (a, u) in used.iter().enumerate() {
        if !u {
            groups.push(vec![a]);
        }
    }
    groups
}

/// The fewest, smallest cards (hand indices) adding up to at least `need`.
pub fn cover(hand: &[i64], need: i64) -> Vec<usize> {
    if need <= 0 || hand.iter().sum::<i64>() < need {
        return Vec::new();
    }
    let mut order: Vec<usize> = (0..hand.len()).collect();
    order.sort_by_key(|&i| -hand[i]);
    let mut picked = Vec::new();
    let mut sum = 0;
    for &i in &order {
        if sum >= need {
            break;
        }
        picked.push(i);
        sum += hand[i];
    }
    // Swap the last card for the smallest one that still covers the need.
    if let Some(&last) = picked.last() {
        let rest = sum - hand[last];
        if let Some(&better) = order.iter().rev().find(|&&i| !picked.contains(&i) && rest + hand[i] >= need && hand[i] < hand[last]) {
            picked.pop();
            picked.push(better);
        }
    }
    picked
}

impl MaState {
    fn say(&mut self, line: String) {
        self.log.push(line);
        let extra = self.log.len().saturating_sub(12);
        self.log.drain(..extra);
    }

    fn monster(&self) -> &Monster {
        &self.monsters[self.round]
    }

    fn resolve_bids(&mut self) {
        let bids: Vec<i64> = self.fighters.iter().map(|f| f.bid.unwrap_or(0)).collect();
        let best = *bids.iter().max().unwrap_or(&0);
        let top: Vec<usize> = (0..bids.len()).filter(|&i| bids[i] == best).collect();
        let bonus = self.monster().bonus;
        let all = self.fighters.iter().zip(&bids).map(|(f, b)| format!("{} {b}", f.id)).collect::<Vec<_>>().join(", ");
        self.say(format!("Bids: {all}."));
        if best > 0 && top.len() == 1 {
            let w = top[0];
            self.fighters[w].points -= best;
            self.fighters[w].hand.push(bonus);
            let line = format!("{} wins the {bonus}-strength weapon for {best} points.", self.fighters[w].id);
            self.say(line);
        } else {
            self.say("Nobody wins the extra weapon.".into());
        }
        for f in &mut self.fighters {
            f.bid = None;
        }
        self.phase = Phase::Ally;
        self.time_left_ms = ALLY_MS;
    }

    fn resolve_alliances(&mut self) {
        let choices: Vec<Vec<usize>> = self.fighters.iter().map(|f| f.ally.clone().unwrap_or_default()).collect();
        self.groups = form_groups(&choices);
        for (f, c) in self.fighters.iter_mut().zip(choices) {
            f.picked = c;
            f.ally = None;
        }
        let text: Vec<String> = self
            .groups
            .iter()
            .filter(|g| g.len() > 1)
            .map(|g| g.iter().map(|&i| self.fighters[i].id.clone()).collect::<Vec<_>>().join(" + "))
            .collect();
        self.say(if text.is_empty() { "No alliances: everyone fights alone.".into() } else { format!("Alliances: {}.", text.join("; ")) });
        self.phase = Phase::Attack;
        self.time_left_ms = ATTACK_MS;
    }

    fn resolve_attack(&mut self) {
        let Monster { name, strength, reward, .. } = self.monster().clone();
        let mut outcomes = Vec::new();
        for g in self.groups.clone() {
            let mut played = Vec::new();
            for &m in &g {
                let mut idx = self.fighters[m].attack.take().unwrap_or_default();
                idx.sort_unstable();
                idx.dedup();
                let mut cards = Vec::new();
                for &i in idx.iter().rev() {
                    if i < self.fighters[m].hand.len() {
                        cards.push(self.fighters[m].hand.remove(i));
                    }
                }
                played.push(cards);
            }
            let power = played.iter().flatten().sum();
            outcomes.push(Outcome { shares: vec![0; g.len()], members: g, played, power, won: false });
        }
        let best = outcomes.iter().filter(|o| o.power >= strength).map(|o| o.power).max();
        if let Some(best) = best {
            let top: Vec<usize> = (0..outcomes.len()).filter(|&k| outcomes[k].power == best).collect();
            if let [k] = top[..] {
                let o = &mut outcomes[k];
                o.won = true;
                let given: Vec<i64> = o.played.iter().map(|c| reward * c.iter().sum::<i64>() / o.power).collect();
                o.shares = given;
                let rest = reward - o.shares.iter().sum::<i64>();
                let lead = (0..o.members.len()).max_by_key(|&m| (o.played[m].iter().sum::<i64>(), std::cmp::Reverse(m))).unwrap_or(0);
                o.shares[lead] += rest;
                // A replay moment: one player alone beats the monster (played cards are shown to all).
                if let [solo] = o.members[..] {
                    let by = self.fighters[solo].id.clone();
                    self.moments.push(&by, format!("Beat {name} alone: all {reward} points"));
                }
            } else {
                let members: i64 = top.iter().map(|&k| outcomes[k].members.len() as i64).sum();
                for &k in &top {
                    outcomes[k].won = true;
                    outcomes[k].shares = vec![reward / members; outcomes[k].members.len()];
                }
            }
            for o in &outcomes {
                for (m, share) in o.members.iter().zip(&o.shares) {
                    self.fighters[*m].points += share;
                }
            }
            let winners: Vec<String> = outcomes
                .iter()
                .filter(|o| o.won)
                .map(|o| o.members.iter().map(|&i| self.fighters[i].id.clone()).collect::<Vec<_>>().join(" + "))
                .collect();
            self.say(format!("{name} (strength {strength}) is beaten by {} with power {best}.", winners.join(" and ")));
        } else {
            self.say(format!("Nobody beats {name} (strength {strength})."));
        }
        self.last = outcomes;
        self.groups.clear();
        self.round += 1;
        if self.round >= ROUNDS {
            self.phase = Phase::Over;
            self.time_left_ms = 0;
        } else {
            self.phase = Phase::Bid;
            self.time_left_ms = BID_MS;
        }
    }
}

pub struct Rules;

impl Game for Rules {
    type State = MaState;
    type Move = MaMove;
    type View = MaView;

    fn info() -> GameInfo {
        GameInfo {
            id: "monster-alliance",
            kind: "main",
            difficulty: "hard",
            tags: &["alliance", "bidding", "cards"],
            min_players: 4,
            max_players: 8,
            name: "Monster Alliance",
            short: "Bid for weapons, form alliances of up to three and beat monsters together to share their reward.",
        }
    }

    fn setup(players: &[String], seed: u64) -> Result<MaState, String> {
        if !(4..=8).contains(&players.len()) {
            return Err("Monster Alliance needs 4 to 8 players".into());
        }
        let mut rng = Rng::new(seed);
        let fighters = players
            .iter()
            .map(|id| Fighter {
                id: id.clone(),
                points: START_POINTS,
                hand: (0..HAND).map(|_| rng.int(1, 6)).collect(),
                bid: None,
                ally: None,
                attack: None,
                picked: Vec::new(),
            })
            .collect();
        let mut names = NAMES.to_vec();
        rng.shuffle(&mut names);
        let monsters = (0..ROUNDS)
            .map(|k| {
                let strength = 6 + 2 * (k as i64 + 1) + rng.int(0, 4);
                Monster { name: names[k], strength, reward: strength * 3 / 2, bonus: rng.int(4, 7) }
            })
            .collect();
        Ok(MaState {
            fighters,
            monsters,
            round: 0,
            phase: Phase::Bid,
            time_left_ms: BID_MS,
            groups: Vec::new(),
            last: Vec::new(),
            log: vec!["A monster approaches. Bid for the extra weapon.".into()],
            moments: Moments::default(),
        })
    }

    fn apply_move(s: &mut MaState, player: &str, mv: MaMove) -> Result<(), String> {
        let i = s.fighters.iter().position(|f| f.id == player).ok_or("Not in this game")?;
        let n = s.fighters.len();
        match (s.phase, mv) {
            (Phase::Bid, MaMove::Bid { amount }) => {
                if amount < 0 || amount > s.fighters[i].points {
                    return Err("Bid between 0 and your points".into());
                }
                if s.fighters[i].bid.is_some() {
                    return Err("You have bid".into());
                }
                s.fighters[i].bid = Some(amount);
                if s.fighters.iter().all(|f| f.bid.is_some()) {
                    s.resolve_bids();
                }
                Ok(())
            }
            (Phase::Ally, MaMove::Ally { mut partners }) => {
                partners.sort_unstable();
                partners.dedup();
                if partners.len() > 2 || partners.iter().any(|&p| p >= n || p == i) {
                    return Err("Pick up to two other players".into());
                }
                if s.fighters[i].ally.is_some() {
                    return Err("You have chosen".into());
                }
                s.fighters[i].ally = Some(partners);
                if s.fighters.iter().all(|f| f.ally.is_some()) {
                    s.resolve_alliances();
                }
                Ok(())
            }
            (Phase::Attack, MaMove::Attack { mut cards }) => {
                cards.sort_unstable();
                cards.dedup();
                if cards.iter().any(|&c| c >= s.fighters[i].hand.len()) {
                    return Err("No such card".into());
                }
                if s.fighters[i].attack.is_some() {
                    return Err("You have attacked".into());
                }
                s.fighters[i].attack = Some(cards);
                if s.fighters.iter().all(|f| f.attack.is_some()) {
                    s.resolve_attack();
                }
                Ok(())
            }
            (Phase::Over, _) => Err("Game is over".into()),
            _ => Err("Not now".into()),
        }
    }

    fn tick(s: &mut MaState, ms: u64) {
        if s.phase == Phase::Over {
            return;
        }
        s.time_left_ms -= ms as i64;
        if s.time_left_ms > 0 {
            return;
        }
        match s.phase {
            Phase::Bid => s.resolve_bids(),
            Phase::Ally => s.resolve_alliances(),
            Phase::Attack => s.resolve_attack(),
            Phase::Over => {}
        }
    }

    fn view_for(s: &MaState, player: &str) -> MaView {
        let me = s.fighters.iter().position(|f| f.id == player).unwrap_or(0);
        let done = |f: &Fighter| match s.phase {
            Phase::Bid => f.bid.is_some(),
            Phase::Ally => f.ally.is_some(),
            Phase::Attack => f.attack.is_some(),
            Phase::Over => true,
        };
        MaView {
            phase: match s.phase {
                Phase::Bid => "bid",
                Phase::Ally => "ally",
                Phase::Attack => "attack",
                Phase::Over => "over",
            },
            round: (s.round + 1).min(ROUNDS),
            rounds: ROUNDS,
            time_left_ms: s.time_left_ms,
            phase_time_ms: match s.phase {
                Phase::Bid => BID_MS,
                Phase::Ally => ALLY_MS,
                _ => ATTACK_MS,
            },
            me,
            monster: s.monsters.get(s.round).cloned(),
            fighters: s
                .fighters
                .iter()
                .enumerate()
                .map(|(i, f)| FighterView { index: i, id: f.id.clone(), points: f.points, cards: f.hand.len(), done: done(f) })
                .collect(),
            hand: s.fighters[me].hand.clone(),
            acted: done(&s.fighters[me]),
            groups: s.groups.clone(),
            last: s.last.clone(),
            log: s.log.clone(),
            score: s.fighters[me].points,
        }
    }

    fn is_over(s: &MaState) -> bool {
        s.phase == Phase::Over
    }

    fn results(s: &MaState) -> Vec<PlayerResult> {
        let scores: Vec<(String, i64)> = s.fighters.iter().map(|f| (f.id.clone(), f.points)).collect();
        rank_points(&scores)
    }

    fn to_move(s: &MaState) -> Vec<String> {
        s.fighters
            .iter()
            .filter(|f| match s.phase {
                Phase::Bid => f.bid.is_none(),
                Phase::Ally => f.ally.is_none(),
                Phase::Attack => f.attack.is_none(),
                Phase::Over => false,
            })
            .map(|f| f.id.clone())
            .collect()
    }

    fn moments(s: &MaState, player: &str) -> Vec<Moment> {
        s.moments.for_seat(player)
    }

    fn bot_move(s: &MaState, player: &str, seed: u64) -> MaMove {
        let mut rng = Rng::new(seed);
        let Some(i) = s.fighters.iter().position(|f| f.id == player) else { return MaMove::Bid { amount: 0 } };
        let me = &s.fighters[i];
        match s.phase {
            Phase::Bid => {
                let bonus = s.monsters.get(s.round).map_or(0, |m| m.bonus);
                let want = (bonus as f64 * (1.0 + rng.next_f64() * 0.5)).round() as i64;
                let cap = me.points / 3 + rng.int(0, 2);
                MaMove::Bid { amount: want.min(cap).min(me.points).max(0) }
            }
            Phase::Ally => {
                let mut others: Vec<(f64, usize)> = (0..s.fighters.len())
                    .filter(|&j| j != i)
                    .map(|j| {
                        let back = if s.fighters[j].picked.contains(&i) { 3.0 } else { 0.0 };
                        (s.fighters[j].hand.len() as f64 + back + rng.next_f64() * 2.0, j)
                    })
                    .collect();
                others.sort_by(|a, b| b.0.total_cmp(&a.0));
                MaMove::Ally { partners: others.iter().take(2).map(|&(_, j)| j).collect() }
            }
            Phase::Attack => {
                let size = s.groups.iter().find(|g| g.contains(&i)).map_or(1, |g| g.len()) as i64;
                let strength = s.monsters.get(s.round).map_or(0, |m| m.strength);
                let need = (strength + size - 1) / size + i64::from(size > 1);
                MaMove::Attack { cards: cover(&me.hand, need) }
            }
            Phase::Over => MaMove::Bid { amount: 0 },
        }
    }
}

pub const READY: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::testing::bot_games_with;

    fn players(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn alliances_need_mutual_choices() {
        let groups = form_groups(&[vec![1, 2], vec![0, 2], vec![0, 1], vec![0], vec![5], vec![4]]);
        assert_eq!(groups, vec![vec![0, 1, 2], vec![4, 5], vec![3]]);
        let groups = form_groups(&[vec![1], vec![2], vec![1], vec![]]);
        assert_eq!(groups, vec![vec![1, 2], vec![0], vec![3]]);
    }

    #[test]
    fn cover_finds_a_small_set() {
        let hand = [6, 1, 3, 2, 5];
        let c = cover(&hand, 8);
        assert!(c.iter().map(|&i| hand[i]).sum::<i64>() >= 8);
        assert_eq!(c.len(), 2);
        assert!(cover(&hand, 100).is_empty());
    }

    #[test]
    fn the_strongest_alliance_shares_by_power() {
        let mut s = Rules::setup(&players(4), 1).unwrap();
        s.monsters[0] = Monster { name: "Test", strength: 10, reward: 15, bonus: 5 };
        for f in &mut s.fighters {
            f.hand = vec![6, 4, 2, 1];
        }
        s.phase = Phase::Attack;
        s.groups = vec![vec![0, 1], vec![2], vec![3]];
        let plays = [vec![0, 1], vec![2], vec![0, 1], vec![]];
        for (p, cards) in players(4).iter().zip(plays) {
            Rules::apply_move(&mut s, p, MaMove::Attack { cards }).unwrap();
        }
        // Alliance 0+1 has 12 > 10 of player 2: it takes 15, split 10:2 (12 + 2, the rest to the leader).
        assert_eq!(s.fighters[0].points, START_POINTS + 13);
        assert_eq!(s.fighters[1].points, START_POINTS + 2);
        assert_eq!(s.fighters[2].points, START_POINTS);
        assert_eq!(s.fighters[0].hand, vec![2, 1]);
        assert_eq!(s.phase, Phase::Bid);
    }

    #[test]
    fn a_tied_auction_gives_nothing() {
        let mut s = Rules::setup(&players(4), 2).unwrap();
        for (p, amount) in players(4).iter().zip([5, 5, 2, 0]) {
            Rules::apply_move(&mut s, p, MaMove::Bid { amount }).unwrap();
        }
        assert!(s.fighters.iter().all(|f| f.hand.len() == HAND));
        assert_eq!(s.fighters[0].points, START_POINTS);
        assert!(Rules::apply_move(&mut s, "p1", MaMove::Ally { partners: vec![1, 2, 3] }).is_err());
    }

    #[test]
    fn hands_stay_secret() {
        let s = Rules::setup(&players(4), 3).unwrap();
        let v = Rules::view_for(&s, "p1");
        assert_eq!(v.hand, s.fighters[0].hand);
        assert_eq!(v.fighters[1].cards, HAND);
        assert_eq!(serde_json::to_string(&v).unwrap().matches("\"hand\"").count(), 1);
    }

    #[test]
    fn thousand_bot_games() {
        bot_games_with::<Rules>(1000, 4, 3000);
        bot_games_with::<Rules>(200, 8, 6000);
    }

    fn attack(groups: Vec<Vec<usize>>, plays: [Vec<usize>; 4]) -> MaState {
        let mut s = Rules::setup(&players(4), 1).unwrap();
        s.monsters[0] = Monster { name: "Rustjaw", strength: 10, reward: 15, bonus: 5 };
        for f in &mut s.fighters {
            f.hand = vec![6, 4, 2, 1];
        }
        s.phase = Phase::Attack;
        s.groups = groups;
        for (p, cards) in players(4).iter().zip(plays) {
            Rules::apply_move(&mut s, p, MaMove::Attack { cards }).unwrap();
        }
        s
    }

    #[test]
    fn a_monster_beaten_alone_is_a_replay_moment() {
        // p1 alone plays 6 + 4; the pair p3 + p4 only reaches 8.
        let s = attack(vec![vec![0], vec![1], vec![2, 3]], [vec![0, 1], vec![], vec![0], vec![2]]);
        let m = Rules::moments(&s, "p3");
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].by.as_str(), m[0].text.as_str()), ("p1", "Beat Rustjaw alone: all 15 points"));
        // An alliance win is no moment, nor a lone win tied with an alliance.
        let s = attack(vec![vec![0, 1], vec![2], vec![3]], [vec![0, 1], vec![2], vec![0, 1], vec![]]);
        assert!(Rules::moments(&s, "p1").is_empty());
        let s = attack(vec![vec![0], vec![1], vec![2, 3]], [vec![0, 1], vec![], vec![0], vec![1]]);
        assert!(s.last.iter().filter(|o| o.won).count() == 2);
        assert!(Rules::moments(&s, "p1").is_empty());
    }

    #[test]
    fn replay_moments_are_rare_but_happen() {
        let stats = crate::game::testing::moment_stats::<Rules>(300, 4, 3000);
        assert!(stats.with_any * 4 >= stats.games, "too rare: {stats:?}");
        assert!(stats.with_any * 10 <= stats.games * 8, "too common: {stats:?}");
        assert!(stats.most <= 4, "too many in one game: {stats:?}");
    }
}
