use std::borrow::ToOwned;
use std::fs::{File, create_dir_all};
use std::io::{Error, ErrorKind, Result};
use std::io::prelude::*;
use std::path::Path;
use data::{BotResult, Entity, RollType, as_io};
use data::RollType::{Basic, Strength, Dexterity, Constitution, Wisdom, Intellect, Charisma};
use data::game::Game;
use data::stats::Stats;
use data::utils::Position;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use rustc_serialize::json::{decode, encode};

#[derive(RustcDecodable, RustcEncodable, Debug, PartialEq, Clone)]
pub struct Player {
    pub username: String,
    pub password: String,
    pub stats: Stats,
    pub feats: Vec<String>,
    pub temp_stats: Option<Stats>,
    pub position: Position,
}

impl Player {
    pub fn create(username: &str, password: &str, health: u8, movement: u8, strength: u8,
                  dexterity: u8, constitution: u8, wisdom: u8, intellect: u8, charisma: u8)
                  -> BotResult<Player> {
        Ok(Player {
            username: String::from_str(username),
            password: try!(as_io(Game::password_hash(password))),
            stats: Stats::new(health, movement, strength, dexterity, constitution, wisdom,
                              intellect, charisma),
            feats: Vec::new(),
            temp_stats: None,
            position: Position(0, 0),
        })
    }

    #[cfg(test)]
    pub fn create_test(username: &str, password: &str, health: u8, movement: u8, strength: u8,
                       dexterity: u8, constitution: u8, wisdom: u8, intellect: u8, charisma: u8)
                       -> Player {
        Player {
            username: String::from_str(username),
            password: String::from_str(password),
            stats: Stats::new(health, movement, strength, dexterity, constitution, wisdom,
                              intellect, charisma),
            feats: Vec::new(),
            temp_stats: None,
            position: Position(0, 0),
        }
    }

    pub fn load(username: &str) -> Result<Player> {
        let mut path = "users/".to_owned();
        path.push_str(username);
        path.push_str(".json");
        let mut file = try!(File::open(&Path::new(&path)));
        let mut data = String::new();
        try!(file.read_to_string(&mut data));
        decode(&data).map_err(|_| Error::new(
            ErrorKind::InvalidInput, "Failed to decode player data." 
        ))
    }

    pub fn save(&self) -> Result<()> {
        let mut path = "users/".to_owned();
        try!(create_dir_all(&Path::new(&path)));
        path.push_str(&self.username);
        path.push_str(".json");
        let mut f = try!(File::create(&Path::new(&path)));
        f.write_all(try!(encode(self).map_err(|_| Error::new(
            ErrorKind::InvalidInput, "Failed to encode player data."
        ))).as_bytes())
    }

    pub fn add_feat(&mut self, feat: &str) {
        self.feats.push(String::from_str(feat))
    }
}

impl Entity for Player {
    fn identifier(&self) -> &str {
        &self.username
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
        } as u8 {
            0 => 1,
            n => n,
        }
    }

    fn do_move(&mut self, pos: Position) -> BotResult<()> {
        if try!(self.position.distance(&pos)) <= self.stats().movement as i32 / 5 {
            self.position = pos;
            Ok(())
        } else {
            Err(super::BotError::InvalidInput(
                format!("You can move at most {} spaces in a turn.", self.stats().movement / 5)
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
    use super::Player;
    use data::Entity;
    use data::RollType::{Basic, Dexterity, Constitution};
    use data::game::Game;
    use data::stats::Stats;
    use data::utils::Position;

    #[test]
    fn create_player() {
        let p = Player::create("test", "test", 20, 30, 12, 12, 12, 12, 12, 12).unwrap();
        let m = Player {
            username: String::from_str("test"),
            password: Game::password_hash("test").unwrap(),
            stats: Stats::new(20, 30, 12, 12, 12, 12, 12, 12),
            feats: Vec::new(),
            temp_stats: None,
            position: Position(0, 0),
        };
        assert_eq!(p, m);
    }

    #[test]
    fn save_load_player() {
        let p = Player::create_test("test4", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        p.save().unwrap();
        let l = Player::load("test4").unwrap();
        assert_eq!(l, p);
    }

    #[test]
    fn add_feat() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        assert_eq!(p.feats.len(), 0);
        p.add_feat("Test Feat");
        assert_eq!(p.feats.len(), 1);
        assert_eq!(&p.feats[0][..], "Test Feat");
    }

    #[test]
    fn damage() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        assert_eq!(p.stats().health, 20);
        assert!(p.damage(5));
        assert_eq!(p.stats().health, 15);
        assert!(!p.damage(16));
        assert_eq!(p.stats().health, 0);
    }

    #[test]
    fn damage_temp_health() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        p.set_temp_stats(Stats::new(40, 30, 10, 10, 10, 10, 10, 10));
        assert_eq!(p.stats().health, 40);
        assert!(p.damage(5));
        assert_eq!(p.stats().health, 35);
        assert!(!p.damage(35));
        assert_eq!(p.stats().health, 0);
    }

    #[test]
    fn do_move_valid() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        assert_eq!(p.position(), &Position(0, 0));
        assert!(p.do_move(Position(6, 0)).is_ok());
        assert_eq!(p.position(), &Position(6, 0));
        assert!(p.do_move(Position(6, 6)).is_ok());
        assert_eq!(p.position(), &Position(6, 6));
        assert!(p.do_move(Position(9, 9)).is_ok());
        assert_eq!(p.position(), &Position(9, 9));
    }

    #[test]
    fn do_move_temp_valid() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        p.set_temp_stats(Stats::new(20, 25, 12, 12, 12, 12, 12, 12));
        assert_eq!(p.position(), &Position(0, 0));
        assert!(p.do_move(Position(5, 0)).is_ok());
        assert_eq!(p.position(), &Position(5, 0));
        assert!(p.do_move(Position(5, 5)).is_ok());
        assert_eq!(p.position(), &Position(5, 5));
        assert!(p.do_move(Position(8, 7)).is_ok());
        assert_eq!(p.position(), &Position(8, 7));
    }

    #[test]
    fn do_move_fail() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        assert!(p.do_move(Position(10, 1)).is_err());
        assert!(p.do_move(Position(7, 0)).is_err());
    }

    #[test]
    fn do_move_temp_fail() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        p.set_temp_stats(Stats::new(20, 25, 12, 12, 12, 12, 12, 12));
        assert!(p.do_move(Position(10, 1)).is_err());
        assert!(p.do_move(Position(6, 0)).is_err());
    }

    #[test]
    fn stats_fn() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        let s = Stats::new(20, 30, 10, 10, 10, 10, 10, 10);
        assert_eq!(p.stats(), Stats::new(20, 30, 12, 12, 12, 12, 12, 12));
        p.set_temp_stats(s);
        assert_eq!(p.stats(), s);
    }

    #[test]
    fn set_temp_stats() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        let s = Stats::new(20, 30, 10, 10, 10, 10, 10, 10);
        p.set_temp_stats(s);
        assert_eq!(p.temp_stats, Some(s));
    }

    #[test]
    fn has_temp_stats() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        let s = Stats::new(20, 30, 10, 10, 10, 10, 10, 10);
        assert!(!p.has_temp_stats());
        p.set_temp_stats(s);
        assert!(p.has_temp_stats());
    }

    #[test]
    fn clear_temp_stats() {
        let mut p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        let s = Stats::new(20, 30, 10, 10, 10, 10, 10, 10);
        p.set_temp_stats(s);
        assert!(p.has_temp_stats());
        p.clear_temp_stats()
    }

    #[test]
    fn basic_roll() {
        let p = Player::create_test("test", "test", 20, 30, 12, 12, 8, 12, 12, 12);
        for _ in 0..1000 {
            let r = p.roll(Basic);
            assert!(r >= 1 && r <= 20);
        }
    }

    #[test]
    fn positive_stat_roll() {
        let p = Player::create_test("test", "test", 20, 30, 12, 12, 8, 12, 12, 12);
        for _ in 0..1000 {
            let r = p.roll(Dexterity);
            println!("{}", r);
            assert!(r >= 1 && r <= 21);
        }
    }

    #[test]
    fn negative_stat_roll() {
        let p = Player::create_test("test", "test", 20, 30, 12, 12, 8, 12, 12, 12);
        for _ in 0..1000 {
            let r = p.roll(Constitution);
            println!("{}", r);
            assert!(r >= 1 && r <= 19);
        }
    }
}
