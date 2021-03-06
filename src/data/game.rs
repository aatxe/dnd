use std::collections::HashMap;
use std::io::Result;
use std::io::prelude::*;
use data::player::Player;
use data::{BotResult, as_io};
use data::BotError::PasswordIncorrect;
use openssl::crypto::hash::{Type, Hasher};
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use rustc_serialize::hex::ToHex;

pub struct Game {
    pub name: String,
    pub dm_nick: String,
    pub users: HashMap<String, Player>,
}

impl Game {
    pub fn new(name: &str, dm_nick: &str) -> Game {
        Game {
            name: name.to_string(),
            dm_nick: dm_nick.to_string(),
            users: HashMap::new(),
        }
    }

    pub fn login(&mut self, account: Player, nickname: &str, password: &str) -> BotResult<&str> {
        if account.password == try!(as_io(Game::password_hash(password))) {
            self.users.insert(nickname.to_string(), account);
            Ok("Login successful.")
        } else {
            Err(PasswordIncorrect)
        }
    }

    pub fn password_hash(password: &str) -> Result<String> {
        let mut hasher = Hasher::new(Type::SHA512);
        try!(hasher.write_all(password.as_bytes()));
        Ok(hasher.finish().to_hex())
    }

    pub fn roll() -> u8 {
        let d20 = Range::new(1i8, 21i8);
        let mut rng = thread_rng();
        match d20.ind_sample(&mut rng) as u8 {
            0 => 1,
            n => n,
        }
    }

    pub fn is_dm(&self, nickname: &str) -> bool {
        &self.dm_nick == nickname
    }
}

#[cfg(test)]
mod test {
    use super::Game;
    use data::player::Player;

    #[test]
    fn password_hash() {
        let s = "ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84\
                 f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff".to_string();
        let h = Game::password_hash("test").unwrap();
        assert_eq!(h, s);
    }

    #[test]
    fn worldless_roll() {
        for _ in 0..1000 {
            let r = Game::roll();
            assert!(r >= 1 && r <= 20);
        }
    }

    #[test]
    fn login() {
        let p = Player::create("test", "test", 20, 30, 12, 12, 12, 12, 12, 12).unwrap();
        p.save().unwrap();
        let mut g = Game::new("test", "test");
        g.login(p, "test", "test").unwrap();
    }

    #[test]
    fn is_dm() {
        let g = Game::new("Dungeons and Tests", "test");
        assert!(g.is_dm("test"));
        assert!(!g.is_dm("test2"));
    }
}
