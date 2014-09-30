use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::io::{InvalidInput, IoError, IoResult};
use data::game::Game;
use data::player::Player;

pub mod game;
pub mod player;
pub mod stats;

pub mod utils {
    pub fn join_from(words: Vec<&str>, pos: uint) -> String {
        let mut res = String::new();
        for word in words.slice_from(pos).iter() {
            res.push_str(*word);
            res.push(' ');
        }
        let len = res.len() - 1;
        res.truncate(len);
        res
    }

    pub fn str_to_u8(s: &str) -> u8 {
        match from_str(s) {
            Some(n) => n,
            None => 0,
        }
    }
}

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


#[cfg(test)]
mod test {
    use super::{RollType, World};
    use super::{Strength, Wisdom, Intellect};
    use super::player::Player;
    use super::utils::{join_from, str_to_u8};

    #[test]
    fn to_roll_type_test() {
        assert_eq!(RollType::to_roll_type("str"), Some(Strength));
        assert_eq!(RollType::to_roll_type("WISDOM"), Some(Wisdom));
        assert_eq!(RollType::to_roll_type("Intellect"), Some(Intellect));
        assert_eq!(RollType::to_roll_type("test"), None);
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
    fn str_to_u8_test() {
        assert_eq!(str_to_u8("4"), 4);
        assert_eq!(str_to_u8("-4"), 0);
        assert_eq!(str_to_u8("x"), 0);
    }

    #[test]
    fn join_from_test() {
        assert_eq!(join_from(vec!["hi","there","friend"], 0).as_slice(), "hi there friend");
            assert_eq!(join_from(vec!["hi","their","friend"], 1).as_slice(), "their friend");
    }
}
