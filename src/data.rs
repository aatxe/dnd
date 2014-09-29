use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use std::rand::task_rng;
use std::rand::distributions::{IndependentSample, Range};
use crypto::sbuf::StdHeapAllocator;
use crypto::sha3::{hash, Sha3_512};
use serialize::hex::ToHex;
use serialize::json::{decode, encode};

pub struct World {
    pub users: HashMap<String, Player>,
    pub games: HashMap<String, Game>,
}

impl World {
    pub fn new() -> IoResult<World> {
        Ok(World {
            users: HashMap::new(),
            games: HashMap::new(),
        })
    }

    pub fn is_user_logged_in(&mut self, nickname: &str) -> bool {
        for user in self.users.keys() {
            if user.as_slice().eq(&nickname) {
                return true;
            }
        };
        false
    }

    pub fn add_user(&mut self, nickname: &str, player: Player) -> IoResult<()> {
        self.users.insert(String::from_str(nickname), player);
        Ok(())
    }

    pub fn remove_user(&mut self, nickname: &str) -> IoResult<()> {
        let nick = String::from_str(nickname);
        try!(self.users[nick].save());
        self.users.remove(&nick);
        Ok(())
    }

    pub fn get_user(&mut self, nickname: &str) -> IoResult<&mut Player> {
        let nick = String::from_str(nickname);
        if self.users.contains_key(&nick) {
            Ok(self.users.get_mut(&nick))
        } else {
            Err(IoError {
                kind: InvalidInput,
                desc: "User not found.",
                detail: None,
            })
        }
    }

    pub fn add_game(&mut self, name: &str, dm_nick: &str, chan: &str) -> IoResult<()> {
        let game = try!(Game::new(name.as_slice(), dm_nick.as_slice()));
        self.games.insert(String::from_str(chan), game);
        Ok(())
    }

    pub fn get_game(&mut self, chan: &str) -> IoResult<&mut Game> {
        let ch = String::from_str(chan);
        if self.games.contains_key(&ch) {
            Ok(self.games.get_mut(&ch))
        } else {
            Err(IoError {
                kind: InvalidInput,
                desc: "Game not found.",
                detail: None,
            })
        }
    }
}

pub struct Game {
    pub name: String,
    pub dm_nick: String,
    pub users: HashMap<String, Player>,
}

impl Game {
    pub fn new(name: &str, dm_nick: &str) -> IoResult<Game> {
        Ok(Game {
            name: String::from_str(name),
            dm_nick: String::from_str(dm_nick),
            users: HashMap::new(),
        })
    }

    pub fn login(&mut self, account: Player, nickname: &str, password: &str) -> IoResult<&str> {
        if account.password.as_slice().eq(&try!(Game::password_hash(password)).as_slice()) {
            self.users.insert(String::from_str(nickname), account);
            Ok("Login successful.")
        } else {
            Ok("Password incorrect.")
        }
    }

    fn password_hash(password: &str) -> IoResult<String> {
        let mut data = [0u8, ..64];
        try!(hash::<StdHeapAllocator>(Sha3_512, password.as_bytes(), data));
        Ok(data.to_hex())
    }

    pub fn roll() -> u8 {
        let d20 = Range::new(1i8, 21i8);
        let mut rng = task_rng();
        match d20.ind_sample(&mut rng).to_u8() {
            Some(0) => 1,
            Some(n) => n,
            None => 1,
        }
    }

    pub fn is_dm(&self, nickname: &str) -> bool {
        let nick = String::from_str(nickname);
        nick.eq(&self.dm_nick)
    }
}

#[deriving(Decodable, Encodable, Show, PartialEq, Clone)]
pub struct Stats {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub wisdom: u8,
    pub intellect: u8,
    pub charisma: u8,
}

impl Stats {
    pub fn new(strength: u8, dexterity: u8, constitution: u8, wisdom: u8,
               intellect: u8, charisma: u8) -> IoResult<Stats> {
        Ok(Stats {
            strength: strength,
            dexterity: dexterity,
            constitution: constitution,
            wisdom: wisdom,
            intellect: intellect,
            charisma: charisma,
        })
    }

    pub fn get_stat(&self, stat: &str) -> Option<u8> {
        match stat.to_ascii_lower().as_slice() {
            "strength" => Some(self.strength),
            "str" => Some(self.strength),
            "dexterity" => Some(self.dexterity),
            "dex" => Some(self.dexterity),
            "constitution" => Some(self.constitution),
            "con" => Some(self.constitution),
            "wisdom" => Some(self.wisdom),
            "wis" => Some(self.wisdom),
            "intellect" => Some(self.intellect),
            "int" => Some(self.intellect),
            "charisma" => Some(self.charisma),
            "cha" => Some(self.charisma),
            _ => None,
        }
    }

    pub fn update_stat(&mut self, stat: &str, value: u8) {
        match stat.to_ascii_lower().as_slice() {
            "strength" => self.strength = value,
            "str" => self.strength = value,
            "dexterity" => self.dexterity = value,
            "dex" => self.dexterity = value,
            "constitution" => self.constitution = value,
            "con" => self.constitution = value,
            "wisdom" => self.wisdom = value,
            "wis" => self.wisdom = value,
            "intellect" => self.intellect = value,
            "int" => self.intellect = value,
            "charisma" => self.charisma = value,
            "cha" => self.charisma = value,
            _ => (),
        }
    }

    pub fn increase_stat(&mut self, stat: &str, value: u8) {
        match stat.to_ascii_lower().as_slice() {
            "strength" => self.strength += value,
            "str" => self.strength += value,
            "dexterity" => self.dexterity += value,
            "dex" => self.dexterity += value,
            "constitution" => self.constitution += value,
            "con" => self.constitution += value,
            "wisdom" => self.wisdom += value,
            "wis" => self.wisdom += value,
            "intellect" => self.intellect += value,
            "int" => self.intellect += value,
            "charisma" => self.charisma += value,
            "cha" => self.charisma += value,
            _ => (),
        }
    }

    pub fn calc_bonus(stat: u8) -> i8 {
        let st = stat as i8;
        (st - 10) / 2
    }
}

#[deriving(Show, PartialEq)]
pub enum RollType {
    Basic,
    Strength,
    Dexterity,
    Constitution,
    Wisdom,
    Intellect,
    Charisma
}

impl RollType {
    pub fn to_roll_type(roll_type: &str) -> Option<RollType> {
        match roll_type.to_ascii_lower().as_slice() {
            "strength" => Some(Strength),
            "str" => Some(Strength),
            "dexterity" => Some(Dexterity),
            "dex" => Some(Dexterity),
            "constitution" => Some(Constitution),
            "con" => Some(Constitution),
            "wisdom" => Some(Wisdom),
            "wis" => Some(Wisdom),
            "intellect" => Some(Intellect),
            "int" => Some(Intellect),
            "charisma" => Some(Charisma),
            "cha" => Some(Charisma),
            _ => None,
        }
    }
}

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
    fn create_test(username: &str, password: &str, strength: u8, dexterity: u8, constitution: u8,
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

mod test {
    use super::{Game, Player, RollType, Stats, World};
    use super::{Basic, Strength, Dexterity, Constitution, Wisdom, Intellect};

    #[test]
    fn create_player_test() {
        let p = Player::create("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
        let m = Player {
            username: String::from_str("test"),
            password: Game::password_hash("test").unwrap(),
            stats: Stats::new(12, 12, 12, 12, 12, 12).unwrap(),
            feats: Vec::new(),
            temp_stats: None,
        };
        assert_eq!(p, m);
    }

    #[test]
    fn save_load_player_test() {
        let p = Player::create_test("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
        p.save().unwrap();
        let l = Player::load("test").unwrap();
        assert_eq!(l, p);
    }

    #[test]
    fn add_feat_test() {
        let mut p = Player::create_test("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
        assert_eq!(p.feats.len(), 0);
        p.add_feat("Test Feat");
        assert_eq!(p.feats.len(), 1);
        assert_eq!(p.feats[0].as_slice(), "Test Feat");
    }

    #[test]
    fn password_hash_test() {
        let s = String::from_str("9ece086e9bac491fac5c1d1046ca11d737b92a2b2ebd93f005d7b710110c0a678288166e7fbe796883a4f2e9b3ca9f484f521d0ce464345cc1aec96779149c14");
        let h = Game::password_hash("test").unwrap();
        assert_eq!(s, h);
    }


    #[test]
    fn worldless_roll_test() {
        for _ in range(0i, 1000i) {
            let r = Game::roll();
            assert!(r >= 1 && r <= 20);
        }
    }

    #[test]
    fn basic_roll_test() {
        let p = Player::create_test("test", "test", 12, 12, 8, 12, 12, 12).unwrap();
        for _ in range(0i, 1000i) {
            let r = p.roll(Basic);
            assert!(r >= 1 && r <= 20);
        }
    }

    #[test]
    fn positive_stat_roll_test() {
        let p = Player::create_test("test", "test", 12, 12, 8, 12, 12, 12).unwrap();
        for _ in range(0i, 1000i) {
            let r = p.roll(Dexterity);
            println!("{}", r)
            assert!(r >= 1 && r <= 21);
        }
    }

    #[test]
    fn negative_stat_roll_test() {
        let p = Player::create_test("test", "test", 12, 12, 8, 12, 12, 12).unwrap();
        for _ in range(0i, 1000i) {
            let r = p.roll(Constitution);
            println!("{}", r)
            assert!(r >= 1 && r <= 19);
        }
    }

    #[test]
    fn to_roll_type_test() {
        assert_eq!(RollType::to_roll_type("str"), Some(Strength));
        assert_eq!(RollType::to_roll_type("WISDOM"), Some(Wisdom));
        assert_eq!(RollType::to_roll_type("Intellect"), Some(Intellect));
        assert_eq!(RollType::to_roll_type("test"), None);
    }

    #[test]
    fn login_test() {
        let p = Player::create("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
        p.save().unwrap();
        let mut g = Game::new("test", "test").unwrap();
        g.login(p, "test", "test").unwrap();
    }

    #[test]
    fn get_stat_test() {
        let s = Stats::new(12, 12, 8, 12, 14, 12).unwrap();
        assert_eq!(s.get_stat("str"), Some(12));
        assert_eq!(s.get_stat("constitution"), Some(8));
        assert_eq!(s.get_stat("INTELLECT"), Some(14));
    }

    #[test]
    fn update_stat_test() {
        let mut s = Stats::new(12, 12, 12, 12, 12, 12).unwrap();
        s.update_stat("str", 10);
        assert_eq!(s.get_stat("str"), Some(10));
        s.update_stat("Con", 8);
        assert_eq!(s.get_stat("constitution"), Some(8));
        s.update_stat("InTeLlEcT", 14);
        assert_eq!(s.get_stat("INTELLECT"), Some(14));
    }

    #[test]
    fn increase_stat_test() {
        let mut s = Stats::new(12, 12, 7, 12, 12, 12).unwrap();
        s.increase_stat("str", 2);
        assert_eq!(s.get_stat("str"), Some(14));
        s.increase_stat("Con", 1);
        assert_eq!(s.get_stat("constitution"), Some(8));
        s.increase_stat("InTeLlEcT", 6);
        assert_eq!(s.get_stat("INTELLECT"), Some(18));
    }

    #[test]
    fn calc_bonus_test() {
        assert_eq!(Stats::calc_bonus(14), 2i8);
        assert_eq!(Stats::calc_bonus(11), 0i8);
        assert_eq!(Stats::calc_bonus(8), -1i8);
    }

    #[test]
    fn world_user_test() {
        let mut w = World::new().unwrap();
        let p = Player::create_test("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
        assert_eq!(w.is_user_logged_in("test"), false);
        w.add_user("test", p.clone()).unwrap();
        assert_eq!(*w.get_user("test").unwrap(), p);
        assert_eq!(w.is_user_logged_in("test"), true);
        w.remove_user("test").unwrap();
        assert_eq!(w.is_user_logged_in("test"), false);
        assert!(w.get_user("test").is_err());

    }

    #[test]
    fn world_game_test() {
        let mut w = World::new().unwrap();
        assert!(w.add_game("Dungeons and Tests", "test", "#test").is_ok());
    }

    #[test]
    fn get_game_test() {
        let mut w = World::new().unwrap();
        w.add_game("Dungeons and Tests", "test", "#test").unwrap();
        assert!(w.get_game("#test").is_ok());
        assert!(w.get_game("#test2").is_err());
    }

    #[test]
    fn is_dm_test() {
        let g = Game::new("Dungeons and Tests", "test").unwrap();
        assert!(g.is_dm("test"));
        assert!(!g.is_dm("test2"));
    }

    #[test]
    fn set_temp_stats_test() {
        let mut p = Player::create_test("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(10, 10, 10, 10, 10, 10).unwrap();
        p.set_temp_stats(s);
        assert_eq!(p.temp_stats, Some(s));
    }

    #[test]
    fn stats_fn_test() {
        let mut p = Player::create_test("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(10, 10, 10, 10, 10, 10).unwrap();
        assert_eq!(p.stats(), Stats::new(12, 12, 12, 12, 12, 12).unwrap());
        p.set_temp_stats(s);
        assert_eq!(p.stats(), s);
    }

    #[test]
    fn has_temp_stats_test() {
        let mut p = Player::create_test("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(10, 10, 10, 10, 10, 10).unwrap();
        assert!(!p.has_temp_stats());
        p.set_temp_stats(s);
        assert!(p.has_temp_stats());
    }

    #[test]
    fn clear_temp_stats_test() {
        let mut p = Player::create_test("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
        let s = Stats::new(10, 10, 10, 10, 10, 10).unwrap();
        p.set_temp_stats(s);
        assert!(p.has_temp_stats());
        p.clear_temp_stats()
    }
}
