use std::collections::HashMap;
use std::io::IoResult;
use std::rand::task_rng;
use std::rand::distributions::{IndependentSample, Range};
use crypto::sbuf::StdHeapAllocator;
use crypto::sha3::{hash, Sha3Mode};
use data::player::Player;
use data::{BotResult, as_io};
use data::BotError::PasswordIncorrect;
use serialize::hex::ToHex;

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
        let mut data = [0u8, ..64];
        try!(hash::<StdHeapAllocator>(Sha3Mode::Sha3_512, password.as_bytes(), &mut data));
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

#[cfg(test)]
mod test {
    use super::Game;
    use data::player::Player;

    #[test]
    fn password_hash() {
        let s = String::from_str("9ece086e9bac491fac5c1d1046ca11d737b92a2b2ebd93f005d7b710110c0a678288166e7fbe796883a4f2e9b3ca9f484f521d0ce464345cc1aec96779149c14");
        let h = Game::password_hash("test").unwrap();
        assert_eq!(s, h);
    }

    #[test]
    fn worldless_roll() {
        for _ in range(0i, 1000i) {
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
