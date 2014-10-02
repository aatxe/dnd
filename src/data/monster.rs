use std::io::IoResult;
use std::rand::task_rng;
use std::rand::distributions::{IndependentSample, Range};
use data::{Entity, RollType, Basic, Strength, Dexterity, Constitution, Wisdom, Intellect, Charisma};
use data::stats::Stats;

#[deriving(Show, PartialEq)]
pub struct Monster {
    pub name: String,
    pub stats: Stats,
    pub temp_stats: Option<Stats>,
}


impl Monster {
    pub fn create(name: &str, health: u8, strength: u8, dexterity: u8, constitution: u8,
                  wisdom: u8, intellect: u8, charisma: u8) -> IoResult<Monster> {
        Ok(Monster {
            name: String::from_str(name),
            stats: try!(Stats::new(health, strength, dexterity, constitution,
                                   wisdom, intellect, charisma)),
            temp_stats: None,
        })
    }
}

impl Entity for Monster {
    fn identifier(&self) -> &str {
        self.name.as_slice()
    }

    fn roll(&self, roll_type: RollType) -> u8 {
        let d20 = Range::new(1i8, 21i8);
        let mut rng = task_rng();
        match match roll_type {
            Basic => d20.ind_sample(&mut rng),
            Strength => d20.ind_sample(&mut rng) + Stats::calc_bonus(self.stats.strength),
            Dexterity => d20.ind_sample(&mut rng) + Stats::calc_bonus(self.stats.dexterity),
            Constitution => d20.ind_sample(&mut rng) + Stats::calc_bonus(self.stats.constitution),
            Wisdom => d20.ind_sample(&mut rng) + Stats::calc_bonus(self.stats.wisdom),
            Intellect => d20.ind_sample(&mut rng) + Stats::calc_bonus(self.stats.intellect),
            Charisma => d20.ind_sample(&mut rng) + Stats::calc_bonus(self.stats.charisma),
        }.to_u8() {
            Some(0) => 1,
            Some(n) => n,
            None => 1,
        }
    }

    fn stats(&self) -> Stats {
        match self.temp_stats {
            Some(stats) => stats,
            None => self.stats,
        }
    }

    fn has_temp_stats(&self) -> bool {
        match self.temp_stats {
            Some(_) => true,
            None => false,
        }
    }

    fn set_temp_stats(&mut self, stats: Stats) {
        self.temp_stats = Some(stats);
    }

    fn clear_temp_stats(&mut self) {
        self.temp_stats = None;
    }
}

#[cfg(test)]
mod test {
    use super::Monster;
    use data::{Entity, Basic, Dexterity, Constitution};
    use data::stats::Stats;

    #[test]
    fn create_monster_test() {
        let m = Monster::create("test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let n = Monster {
            name: String::from_str("test"),
            stats: Stats::new(20, 12, 12, 12, 12, 12, 12).unwrap(),
            temp_stats: None,
        };
        assert_eq!(m, n);
    }

    #[test]
    fn stats_fn_test() {
        let mut m = Monster::create("test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(20, 10, 10, 10, 10, 10, 10).unwrap();
        assert_eq!(m.stats(), Stats::new(20, 12, 12, 12, 12, 12, 12).unwrap());
        m.set_temp_stats(s);
        assert_eq!(m.stats(), s);
    }

    #[test]
    fn set_temp_stats_test() {
        let mut m = Monster::create("test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(20, 10, 10, 10, 10, 10, 10).unwrap();
        m.set_temp_stats(s);
        assert_eq!(m.temp_stats, Some(s));
    }

    #[test]
    fn has_temp_stats_test() {
        let mut m = Monster::create("test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(20, 10, 10, 10, 10, 10, 10).unwrap();
        assert!(!m.has_temp_stats());
        m.set_temp_stats(s);
        assert!(m.has_temp_stats());
    }

    #[test]
    fn clear_temp_stats_test() {
        let mut m = Monster::create("test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(20, 10, 10, 10, 10, 10, 10).unwrap();
        m.set_temp_stats(s);
        assert!(m.has_temp_stats());
        m.clear_temp_stats()
    }

    #[test]
    fn basic_roll_test() {
        let m = Monster::create("test", 20, 12, 12, 8, 12, 12, 12).unwrap();
        for _ in range(0i, 1000i) {
            let r = m.roll(Basic);
            assert!(r >= 1 && r <= 20);
        }
    }

    #[test]
    fn positive_stat_roll_test() {
        let m = Monster::create("test", 20, 12, 12, 8, 12, 12, 12).unwrap();
        for _ in range(0i, 1000i) {
            let r = m.roll(Dexterity);
            println!("{}", r)
            assert!(r >= 1 && r <= 21);
        }
    }

    #[test]
    fn negative_stat_roll_test() {
        let m = Monster::create("test", 20, 12, 12, 8, 12, 12, 12).unwrap();
        for _ in range(0i, 1000i) {
            let r = m.roll(Constitution);
            println!("{}", r)
            assert!(r >= 1 && r <= 19);
        }
    }
}
