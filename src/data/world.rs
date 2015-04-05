use std::borrow::ToOwned;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::io::{Error, ErrorKind, Result};
use data::{BotResult, Entity, as_io};
use data::BotError::{Io, NotFound};
use data::game::Game;
use data::monster::Monster;
use data::player::Player;

pub struct World {
    pub users: HashMap<String, Player>,
    pub user_channels: HashMap<String, String>,
    pub games: HashMap<String, Game>,
    pub monsters: HashMap<String, Vec<Monster>>,
}

impl World {
    pub fn new() -> World {
        World {
            users: HashMap::new(),
            user_channels: HashMap::new(),
            games: HashMap::new(),
            monsters: HashMap::new(),
        }
    }

    pub fn is_user_logged_in(&self, nickname: &str) -> bool {
        self.users.contains_key(&String::from_str(nickname))
    }

    pub fn add_user(&mut self, nickname: &str, chan: &str, player: Player) {
        self.users.insert(nickname.to_owned(), player);
        self.user_channels.insert(nickname.to_owned(), chan.to_owned());
    }

    pub fn remove_user(&mut self, nickname: &str) -> BotResult<&str> {
        let nick = String::from_str(nickname);
        try!(as_io(self.users[&nick].save()));
        self.users.remove(&nick);
        Ok(&self.user_channels[&nick])
    }

    pub fn get_user(&mut self, nickname: &str) -> BotResult<&mut Player> {
        let nick = String::from_str(nickname);
        if self.users.contains_key(&nick) {
            Ok(self.users.get_mut(&nick).unwrap())
        } else {
            Err(NotFound(String::from_str("User not found.")))
        }
    }

    pub fn game_exists(&self, chan: &str) -> bool {
        self.games.contains_key(&String::from_str(chan))
    }

    pub fn add_game(&mut self, name: &str, dm_nick: &str, chan: &str) {
        let game = Game::new(&name, &dm_nick);
        self.games.insert(String::from_str(chan), game);
    }

    pub fn get_game(&mut self, chan: &str) -> BotResult<&mut Game> {
        let ch = String::from_str(chan);
        if self.games.contains_key(&ch) {
            Ok(self.games.get_mut(&ch).unwrap())
        } else {
            Err(NotFound(String::from_str("Game not found.")))
        }
    }

    pub fn add_monster(&mut self, monster: Monster, chan: &str) -> usize {
        let chan = String::from_str(chan);
        let result = match self.monsters.entry(chan) {
            Vacant(entry) => entry.insert(Vec::new()),
            Occupied(entry) => entry.into_mut(),
        };
        result.push(monster);
        result.len() - 1
    }

    pub fn get_entity(&mut self, identifier: &str, chan: Option<&str>) -> BotResult<&mut Entity> {
        if identifier.starts_with("@") {
            let i: usize = match identifier[1..].parse() {
                Ok(n) => n,
                Err(_) => return Err(Io(Error::new(
                    ErrorKind::InvalidInput, "Non-integer identifier."
                ))),
            };
            if chan.is_some() {
                let chan_str = String::from_str(chan.unwrap());
                if self.monsters.contains_key(&chan_str) && i < self.monsters[&chan_str].len() {
                    Ok(self.monsters.get_mut(&chan_str).unwrap().get_mut(i).unwrap())
                } else {
                    Err(NotFound(String::from_str("No such monster.")))
                }
            } else {
                Err(Io(Error::new(ErrorKind::InvalidInput, "Monsters require a channel.")))
            }
        } else {
            let nick = String::from_str(identifier);
            if self.users.contains_key(&nick) {
                Ok(self.users.get_mut(&nick).unwrap())
            } else {
                Err(NotFound(String::from_str("User not found.")))
            }
        }
    }

    pub fn save_all(&self) -> Result<()> {
        for user in self.users.values() {
            try!(user.save());
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use data::Entity;
    use data::monster::Monster;
    use data::player::Player;
    use data::world::World;

    #[test]
    fn world_user() {
        let mut w = World::new();
        let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        assert!(!w.is_user_logged_in("test"));
        w.add_user("test", "#test", p.clone());
        assert_eq!(*w.get_user("test").unwrap(), p);
        assert!(w.is_user_logged_in("test"));
        assert_eq!(w.remove_user("test").unwrap(), "#test");
        assert!(!w.is_user_logged_in("test"));
        assert!(w.get_user("test").is_err());
    }

    #[test]
    fn game_exists() {
        let mut w = World::new();
        assert!(!w.game_exists("#test"));
        w.add_game("Dungeons and Tests", "test", "#test");
        assert!(w.game_exists("#test"));
    }

    #[test]
    fn get_game() {
        let mut w = World::new();
        w.add_game("Dungeons and Tests", "test", "#test");
        assert!(w.get_game("#test").is_ok());
        assert!(w.get_game("#test2").is_err());
    }

    #[test]
    fn add_monster() {
        let mut w = World::new();
        assert_eq!(w.add_monster(Monster::create("test", 20, 30, 12, 12, 12, 12, 12, 12), "#test"), 0);
        assert_eq!(w.add_monster(Monster::create("test2", 20, 30, 12, 12, 12, 12, 12, 12), "#test"), 1);
    }

    #[test]
    fn get_entity() {
        let mut w = World::new();
        let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        let m = Monster::create("TestZombie", 20, 30, 12, 12, 12, 12, 12, 12);
        w.add_user("test", "#test", p.clone());
        w.add_monster(m.clone(), "#test");
        assert_eq!(w.get_entity("test", None).unwrap().identifier(), p.identifier());
        assert_eq!(w.get_entity("@0", Some("#test")).unwrap().identifier(), m.identifier());
        assert_eq!(w.get_entity("test", Some("#test")).unwrap().identifier(), p.identifier());
    }

    #[test]
    fn save_all() {
        let mut w = World::new();
        let p = Player::create_test("test2", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        let q = Player::create_test("test3", "test", 20, 30, 12, 12, 12, 12, 12, 12);
        w.add_user("test2", "#test", p.clone());
        w.add_user("test3", "#test", q.clone());
        w.save_all().unwrap();
        let l = Player::load("test2").unwrap();
        let m = Player::load("test3").unwrap();
        assert_eq!(l, p);
        assert_eq!(m, q);
    }

}
