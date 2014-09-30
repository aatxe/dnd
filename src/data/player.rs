use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use std::rand::task_rng;
use std::rand::distributions::{IndependentSample, Range};
use data::{RollType, Basic, Strength, Dexterity, Constitution, Wisdom, Intellect, Charisma, Game};
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
    pub fn create(username: &str, password: &str, strength: u8, dexterity: u8, constitution: u8,
                  wisdom: u8, intellect: u8, charisma: u8) -> IoResult<Player> {
        Ok(Player {
            username: String::from_str(username),
            password: try!(Game::password_hash(password)),
            stats: try!(Stats::new(strength, dexterity, constitution, wisdom, intellect, charisma)),
            feats: Vec::new(),
            temp_stats: None,
        })
    }

    #[cfg(test)]
    pub fn create_test(username: &str, password: &str, strength: u8, dexterity: u8, constitution: u8,
                  wisdom: u8, intellect: u8, charisma: u8) -> IoResult<Player> {
        Ok(Player {
            username: String::from_str(username),
            password: String::from_str(password),
            stats: try!(Stats::new(strength, dexterity, constitution, wisdom, intellect, charisma)),
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

    pub fn roll(&self, roll_type: RollType) -> u8 {
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

    pub fn stats(&self) -> Stats {
        match self.temp_stats {
            Some(stats) => stats,
            None => self.stats,
        }
    }

    pub fn has_temp_stats(&self) -> bool {
        match self.temp_stats {
            Some(_) => true,
            None => false,
        }
    }

    pub fn set_temp_stats(&mut self, stats: Stats) {
        self.temp_stats = Some(stats);
    }

    pub fn clear_temp_stats(&mut self) {
        self.temp_stats = None;
    }
}
