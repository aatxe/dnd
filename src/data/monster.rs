use std::num::ToPrimitive;
use std::rand::thread_rng;
use std::rand::distributions::{IndependentSample, Range};
use data::{BotResult, Entity, RollType};
use data::BotError::InvalidInput;
use data::RollType::{Basic, Strength, Dexterity, Constitution, Wisdom, Intellect, Charisma};
use data::stats::Stats;
use data::utils::Position;

#[deriving(Show, PartialEq, Clone)]
pub struct Monster {
    pub name: String,
    pub stats: Stats,
    pub temp_stats: Option<Stats>,
    pub position: Position,
}


impl Monster {
    pub fn create(name: &str, health: u8, movement: u8, strength: u8, dexterity: u8,
                  constitution: u8, wisdom: u8, intellect: u8, charisma: u8) -> Monster {
        Monster {
            name: String::from_str(name),
            stats: Stats::new(health, movement, strength, dexterity, constitution, wisdom,
                              intellect, charisma),
            temp_stats: None,
            position: Position(0, 0),
        }
    }
}

impl Entity for Monster {
    fn identifier(&self) -> &str {
        self.name[]
    }

    fn position(&self) -> &Position {
        &self.position
    }

    fn damage(&mut self, amount: u8) -> bool {
        if self.temp_stats.is_some() {
            let mut temp = self.temp_stats.unwrap();
            let ret = temp.damage(amount);
            self.temp_stats = Some(temp);
            ret
        } else {
            self.stats.damage(amount)
        }
    }

    fn roll(&self, roll_type: RollType) -> u8 {
        let d20 = Range::new(1i8, 21i8);
        let mut rng = thread_rng();
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

    fn do_move(&mut self, pos: Position) -> BotResult<()> {
        if try!(self.position.distance(&pos)) <= self.stats().movement as int / 5 {
            self.position = pos;
            Ok(())
        } else {
            Err(InvalidInput(
                format!("{} can move at most {} spaces in a turn.",
                        self.identifier(), self.stats().movement / 5)
            ))
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
    use data::Entity;
    use data::RollType::{Basic, Dexterity, Constitution};
    use data::stats::Stats;
    use data::utils::Position;

    #[test]
    fn create_monster() {
        let m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        let n = Monster {
            name: String::from_str("test"),
            stats: Stats::new(20, 30, 12, 12, 12, 12, 12, 12),
            temp_stats: None,
            position: Position(0, 0),
        };
        assert_eq!(m, n);
    }

    #[test]
    fn damage() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        assert_eq!(m.stats().health, 20);
        assert!(m.damage(5));
        assert_eq!(m.stats().health, 15);
        assert!(!m.damage(16));
        assert_eq!(m.stats().health, 0);
    }

    #[test]
    fn damage_temp_health() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        m.set_temp_stats(Stats::new(40, 30, 10, 10, 10, 10, 10, 10));
        assert_eq!(m.stats().health, 40);
        assert!(m.damage(5));
        assert_eq!(m.stats().health, 35);
        assert!(!m.damage(35));
        assert_eq!(m.stats().health, 0);
    }

    #[test]
    fn do_move_valid() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        assert_eq!(m.position(), &Position(0, 0));
        assert!(m.do_move(Position(6, 0)).is_ok());
        assert_eq!(m.position(), &Position(6, 0));
        assert!(m.do_move(Position(6, 6)).is_ok());
        assert_eq!(m.position(), &Position(6, 6));
        assert!(m.do_move(Position(9, 9)).is_ok());
        assert_eq!(m.position(), &Position(9, 9));
    }

    #[test]
    fn do_move_temp_valid() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        m.set_temp_stats(Stats::new(20, 25, 12, 12, 12, 12, 12, 12));
        assert_eq!(m.position(), &Position(0, 0));
        assert!(m.do_move(Position(5, 0)).is_ok());
        assert_eq!(m.position(), &Position(5, 0));
        assert!(m.do_move(Position(5, 5)).is_ok());
        assert_eq!(m.position(), &Position(5, 5));
        assert!(m.do_move(Position(8, 7)).is_ok());
        assert_eq!(m.position(), &Position(8, 7));
    }

    #[test]
    fn do_move_fail() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        assert!(m.do_move(Position(10, 1)).is_err());
        assert!(m.do_move(Position(7, 0)).is_err());
    }

    #[test]
    fn do_move_temp_fail() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        m.set_temp_stats(Stats::new(20, 25, 12, 12, 12, 12, 12, 12));
        assert!(m.do_move(Position(10, 1)).is_err());
        assert!(m.do_move(Position(6, 0)).is_err());
    }

    #[test]
    fn stats_fn() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        let s = Stats::new(20, 30, 10, 10, 10, 10, 10, 10);
        assert_eq!(m.stats(), Stats::new(20, 30, 12, 12, 12, 12, 12, 12));
        m.set_temp_stats(s);
        assert_eq!(m.stats(), s);
    }

    #[test]
    fn set_temp_stats() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        let s = Stats::new(20, 30, 10, 10, 10, 10, 10, 10);
        m.set_temp_stats(s);
        assert_eq!(m.temp_stats, Some(s));
    }

    #[test]
    fn has_temp_stats() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        let s = Stats::new(20, 30, 10, 10, 10, 10, 10, 10);
        assert!(!m.has_temp_stats());
        m.set_temp_stats(s);
        assert!(m.has_temp_stats());
    }

    #[test]
    fn clear_temp_stats() {
        let mut m = Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12);
        let s = Stats::new(20, 30, 10, 10, 10, 10, 10, 10);
        m.set_temp_stats(s);
        assert!(m.has_temp_stats());
        m.clear_temp_stats()
    }

    #[test]
    fn basic_roll() {
        let m = Monster::create("test", 20, 30, 12, 12, 8, 12, 12, 12);
        for _ in range(0i, 1000i) {
            let r = m.roll(Basic);
            assert!(r >= 1 && r <= 20);
        }
    }

    #[test]
    fn positive_stat_roll() {
        let m = Monster::create("test", 20, 30, 12, 12, 8, 12, 12, 12);
        for _ in range(0i, 1000i) {
            let r = m.roll(Dexterity);
            println!("{}", r);
            assert!(r >= 1 && r <= 21);
        }
    }

    #[test]
    fn negative_stat_roll() {
        let m = Monster::create("test", 20, 30, 12, 12, 8, 12, 12, 12);
        for _ in range(0i, 1000i) {
            let r = m.roll(Constitution);
            println!("{}", r);
            assert!(r >= 1 && r <= 19);
        }
    }
}
