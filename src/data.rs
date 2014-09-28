use std::collections::HashMap;
use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use crypto::sbuf::DefaultAllocator;
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
        try!(hash::<DefaultAllocator>(Sha3_512, password.as_bytes(), data));
        Ok(data.to_hex())
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
}

#[deriving(Decodable, Encodable, Show, PartialEq, Clone)]
pub struct Player {
    pub username: String,
    pub password: String,

    pub stats: Stats,
    pub feats: Vec<String>,
}

impl Player {
    pub fn create(username: &str, password: &str, strength: u8, dexterity: u8, constitution: u8,
                  wisdom: u8, intellect: u8, charisma: u8) -> IoResult<Player> {
        Ok(Player {
            username: String::from_str(username),
            password: try!(Game::password_hash(password)),
            stats: try!(Stats::new(strength, dexterity, constitution, wisdom, intellect, charisma)),
            feats: Vec::new(),
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

#[test]
fn create_player_test() {
    let p = Player::create("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
    let m = Player {
        username: String::from_str("test"),
        password: String::from_str("test"),
        stats: Stats::new(12, 12, 12, 12, 12, 12).unwrap(),
        feats: Vec::new(),
    };
    assert_eq!(p, m);
}

#[test]
fn save_load_player_test() {
    let p = Player::create("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
    p.save().unwrap();
    let l = Player::load("test").unwrap();
    assert_eq!(l, p);
}
