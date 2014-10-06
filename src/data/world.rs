use std::collections::HashMap;
use std::collections::hashmap::{Occupied, Vacant};
use std::io::{InvalidInput, IoError, IoResult};
use data::Entity;
use data::game::Game;
use data::monster::Monster;
use data::player::Player;

pub struct World {
    pub users: HashMap<String, Player>,
    pub games: HashMap<String, Game>,
    pub monsters: HashMap<String, Vec<Monster>>,
}

impl World {
    pub fn new() -> IoResult<World> {
        Ok(World {
            users: HashMap::new(),
            games: HashMap::new(),
            monsters: HashMap::new(),
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

    pub fn add_monster(&mut self, monster: Monster, chan: &str) -> IoResult<uint> {
        let result = match self.monsters.entry(String::from_str(chan)) {
            Vacant(entry) => entry.set(Vec::new()),
            Occupied(entry) => entry.into_mut(),
        };
        result.push(monster);
        Ok(result.len() - 1)
    }

    pub fn get_entity(&mut self, identifier: &str, chan: Option<&str>) -> IoResult<&mut Entity> {
        if identifier.starts_with("@") {
            let i = match from_str(identifier.slice_from(1)) {
                Some(n) => n,
                None => 0,
            };
            if chan.is_some() {
                let chan_str = String::from_str(chan.unwrap());
                if i < self.monsters.get_mut(&chan_str).len() {
                    Ok(self.monsters.get_mut(&chan_str).get_mut(i) as &mut Entity)
                } else {
                    Err(IoError {
                        kind: InvalidInput,
                        desc: "No such monster.",
                        detail: None,
                    })
                }
            } else {
                Err(IoError {
                    kind: InvalidInput,
                    desc: "Monsters require a channel.",
                    detail: None,
                })
            }
        } else {
            let nick = String::from_str(identifier);
            if self.users.contains_key(&nick) {
                Ok(self.users.get_mut(&nick) as &mut Entity)
            } else {
                Err(IoError {
                    kind: InvalidInput,
                    desc: "User not found.",
                    detail: None,
                })
            }
        }
    }

    pub fn save_all(&self) -> IoResult<()> {
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
        let mut w = World::new().unwrap();
        let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        assert_eq!(w.is_user_logged_in("test"), false);
        w.add_user("test", p.clone()).unwrap();
        assert_eq!(*w.get_user("test").unwrap(), p);
        assert_eq!(w.is_user_logged_in("test"), true);
        w.remove_user("test").unwrap();
        assert_eq!(w.is_user_logged_in("test"), false);
        assert!(w.get_user("test").is_err());

    }

    #[test]
    fn add_game() {
        let mut w = World::new().unwrap();
        assert!(w.add_game("Dungeons and Tests", "test", "#test").is_ok());
    }

    #[test]
    fn get_game() {
        let mut w = World::new().unwrap();
        w.add_game("Dungeons and Tests", "test", "#test").unwrap();
        assert!(w.get_game("#test").is_ok());
        assert!(w.get_game("#test2").is_err());
    }

    #[test]
    fn add_monster() {
        let mut w = World::new().unwrap();
        assert!(w.add_monster(Monster::create("test", 20, 12, 12, 12, 12, 12, 12).unwrap(), "#test").is_ok());
        assert!(w.add_monster(Monster::create("test2", 20, 12, 12, 12, 12, 12, 12).unwrap(), "#test").is_ok());
    }

    #[test]
    fn get_entity() {
        let mut w = World::new().unwrap();
        let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let m = Monster::create("TestZombie", 20, 12, 12, 12, 12, 12, 12).unwrap();
        w.add_user("test", p.clone()).unwrap();
        w.add_monster(m.clone(), "#test").unwrap();
        assert_eq!(w.get_entity("test", None).unwrap().identifier(), p.identifier());
        assert_eq!(w.get_entity("@0", Some("#test")).unwrap().identifier(), m.identifier());
        assert_eq!(w.get_entity("test", Some("#test")).unwrap().identifier(), p.identifier());
    }

    #[test]
    fn save_all() {
        let mut w = World::new().unwrap();
        let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        let q = Player::create_test("test2", "test", 20, 12, 12, 12, 12, 12, 12).unwrap();
        w.add_user("test", p.clone()).unwrap();
        w.add_user("test2", q.clone()).unwrap();
        w.save_all().unwrap();
        let l = Player::load("test").unwrap();
        let m = Player::load("test2").unwrap();
        assert_eq!(l, p);
        assert_eq!(m, q);
    }

}
