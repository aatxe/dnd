use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use std::rand::task_rng;
use std::rand::distributions::{IndependentSample, Range};
use data::{Entity, RollType, Basic, Strength, Dexterity, Constitution, Wisdom, Intellect, Charisma};
use data::game::Game;
use data::stats::Stats;
use serialize::json::{decode, encode};

#[deriving(Decodable, Encodable, Show, PartialEq, Clone)]
pub struct Player {
    pub username: String,
    pub password: String,

    pub stats: Stats,
    pub feats: Vec<String>,
    pub temp_stats: Option<Stats>,
}

impl Player {
    pub fn create(username: &str, password: &str, health: u8, strength: u8, dexterity: u8, constitution: u8,
                  wisdom: u8, intellect: u8, charisma: u8) -> IoResult<Player> {
        Ok(Player {
            username: String::from_str(username),
            password: try!(Game::password_hash(password)),
            stats: try!(Stats::new(health, strength, dexterity, constitution,
                                   wisdom, intellect, charisma)),
            feats: Vec::new(),
            temp_stats: None,
        })
    }

    #[cfg(test)]
    pub fn create_test(username: &str, password: &str, health: u8, strength: u8, dexterity: u8, constitution: u8,
                  wisdom: u8, intellect: u8, charisma: u8) -> IoResult<Player> {
        Ok(Player {
            username: String::from_str(username),
            password: String::from_str(password),
            stats: try!(Stats::new(health, strength, dexterity, constitution,
                                   wisdom, intellect, charisma)),
            feats: Vec::new(),
            temp_stats: None,
        })
    }

    pub fn load(username: &str) -> IoResult<Player> {
        let mut path = String::from_str(username);
        path.push_str(".json");
        let mut file = try!(File::open(&Path::new(path.as_slice())));
        let data = try!(file.read_to_string());
        decode(data.as_slice()).map_err(|e| IoError {
            kind: InvalidInput,
            desc: "Decoder error",
            detail: Some(e.to_string()),
        })
    }

    pub fn save(&self) -> IoResult<()> {
        let mut path = self.username.clone();
        path.push_str(".json");
        let mut f = File::create(&Path::new(path.as_slice()));
        f.write_str(encode(self).as_slice())
    }

    pub fn add_feat(&mut self, feat: &str) {
        self.feats.push(String::from_str(feat))
    }
}

impl Entity for Player {
    fn identifier(&self) -> &str {
        self.username.as_slice()
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
    use super::Player;
    use data::{Basic, Dexterity, Constitution};
    use data::game::Game;
    use data::stats::Stats;

    #[test]
    fn create_player_test() {
        let p = Player::create("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let m = Player {
            username: String::from_str("test"),
            password: Game::password_hash("test").unwrap(),
            stats: Stats::new(20, 12, 12, 12, 12, 12, 12).unwrap(),
            feats: Vec::new(),
            temp_stats: None,
        };
        assert_eq!(p, m);
    }

    #[test]
    fn save_load_player_test() {
        let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        p.save().unwrap();
        let l = Player::load("test").unwrap();
        assert_eq!(l, p);
    }

    #[test]
    fn add_feat_test() {
        let mut p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        assert_eq!(p.feats.len(), 0);
        p.add_feat("Test Feat");
        assert_eq!(p.feats.len(), 1);
        assert_eq!(p.feats[0].as_slice(), "Test Feat");
    }

    #[test]
    fn stats_fn_test() {
        let mut p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(20, 10, 10, 10, 10, 10, 10).unwrap();
        assert_eq!(p.stats(), Stats::new(20, 12, 12, 12, 12, 12, 12).unwrap());
        p.set_temp_stats(s);
        assert_eq!(p.stats(), s);
    }

    #[test]
    fn set_temp_stats_test() {
        let mut p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(20, 10, 10, 10, 10, 10, 10).unwrap();
        p.set_temp_stats(s);
        assert_eq!(p.temp_stats, Some(s));
    }

    #[test]
    fn has_temp_stats_test() {
        let mut p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(20, 10, 10, 10, 10, 10, 10).unwrap();
        assert!(!p.has_temp_stats());
        p.set_temp_stats(s);
        assert!(p.has_temp_stats());
    }

    #[test]
    fn clear_temp_stats_test() {
        let mut p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(20, 10, 10, 10, 10, 10, 10).unwrap();
        p.set_temp_stats(s);
        assert!(p.has_temp_stats());
        p.clear_temp_stats()
    }

    #[test]
    fn basic_roll_test() {
        let p = Player::create_test("test", "test", 20, 12, 12, 8, 12, 12, 12).unwrap();
        for _ in range(0i, 1000i) {
            let r = p.roll(Basic);
            assert!(r >= 1 && r <= 20);
        }
    }

    #[test]
    fn positive_stat_roll_test() {
        let p = Player::create_test("test", "test", 20, 12, 12, 8, 12, 12, 12).unwrap();
        for _ in range(0i, 1000i) {
            let r = p.roll(Dexterity);
            println!("{}", r)
            assert!(r >= 1 && r <= 21);
        }
    }

    #[test]
    fn negative_stat_roll_test() {
        let p = Player::create_test("test", "test", 20, 12, 12, 8, 12, 12, 12).unwrap();
        for _ in range(0i, 1000i) {
            let r = p.roll(Constitution);
            println!("{}", r)
            assert!(r >= 1 && r <= 19);
        }
    }
}
