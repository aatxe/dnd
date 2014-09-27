use std::collections::HashMap;
use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use serialize::json::{decode, encode};

pub struct Game {
    pub name: String,
    pub users: HashMap<String, Player>,
}

impl Game {
    pub fn new(name: &str) -> IoResult<Game> {
        Ok(Game {
            name: String::from_str(name),
            users: HashMap::new(),
        })
    }

    pub fn login(&mut self, account: Player, nickname: &str, password: &str) -> IoResult<&str> {
        if account.password.as_slice().eq(&password) {
            self.users.insert(String::from_str(nickname), account);
            Ok("Login successful.")
        } else {
            Ok("Password incorrect.")
        }
    }
}

#[deriving(Decodable, Encodable, Show, PartialEq)]
pub struct Player {
    pub username: String,
    pub password: String,

    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub wisdom: u8,
    pub intellect: u8,
    pub charisma: u8,
}

impl Player {
    pub fn create(username: &str, password: &str, strength: u8, dexterity: u8, constitution: u8,
                  wisdom: u8, intellect: u8, charisma: u8) -> IoResult<Player> {
        Ok(Player {
            username: String::from_str(username),
            password: String::from_str(password),
            strength: strength,
            dexterity: dexterity,
            constitution: constitution,
            wisdom: wisdom,
            intellect: intellect,
            charisma: charisma,
        })
    }

    pub fn load(username: &str) -> IoResult<Player> {
        let path = String::from_str(username).append(".json");
        let mut file = try!(File::open(&Path::new(path.as_slice())));
        let data = try!(file.read_to_string());
        decode(data.as_slice()).map_err(|e| IoError {
            kind: InvalidInput,
            desc: "Decoder error",
            detail: Some(e.to_string()),
        })
    }

    pub fn save(&self) -> IoResult<()> {
        let path = self.username.clone().append(".json");
        let mut f = File::create(&Path::new(path.as_slice()));
        f.write_str(encode(self).as_slice())
    }
}

#[test]
fn create_player_test() {
    let p = Player::create("test", "test", 12, 12, 12, 12, 12, 12).unwrap();
    let m = Player {
        username: String::from_str("test"),
        password: String::from_str("test"),
        strength: 12,
        dexterity: 12,
        constitution: 12,
        wisdom: 12,
        intellect: 12,
        charisma: 12,
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
