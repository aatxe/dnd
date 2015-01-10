use std::collections::HashMap;
use std::io::IoResult;
use std::num::ToPrimitive;
use std::rand::thread_rng;
use std::rand::distributions::{IndependentSample, Range};
use data::player::Player;
use data::{BotResult, as_io};
use data::BotError::PasswordIncorrect;
use openssl::crypto::hash::{HashType, Hasher};
use rustc_serialize::hex::ToHex;

pub struct Game {
    pub name: String,
    pub dm_nick: String,
    pub users: HashMap<String, Player>,
}

impl Game {
    pub fn new(name: &str, dm_nick: &str) -> Game {
        Game {
            name: String::from_str(name),
            dm_nick: String::from_str(dm_nick),
            users: HashMap::new(),
        }
    }

    pub fn login(&mut self, account: Player, nickname: &str, password: &str) -> BotResult<&str> {
        if account.password == try!(as_io(Game::password_hash(password))) {
            self.users.insert(String::from_str(nickname), account);
            Ok("Login successful.")
        } else {
            Err(PasswordIncorrect)
        }
    }

    pub fn password_hash(password: &str) -> IoResult<String> {
        let mut hasher = Hasher::new(HashType::SHA512);
        try!(hasher.write_str(password));
        Ok(hasher.finalize().to_hex())
    }

    pub fn roll() -> u8 {
        let d20 = Range::new(1i8, 21i8);
        let mut rng = thread_rng();
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

#[cfg(test)]
mod test {
    use super::Game;
    use data::player::Player;

    #[test]
    fn password_hash() {
        let s = String::from_str("ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff");
        let h = Game::password_hash("test").unwrap();
        assert_eq!(h, s);
    }

    #[test]
    fn worldless_roll() {
        for _ in range(0u16, 1000) {
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
