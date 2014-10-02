use std::collections::HashMap;
use std::io::{InvalidInput, IoError, IoResult};
use data::Entity;
use data::game::Game;
use data::monster::Monster;
use data::player::Player;

pub struct World {
    pub users: HashMap<String, Player>,
    pub games: HashMap<String, Game>,
    pub monsters: Vec<Monster>,
}

impl World {
    pub fn new() -> IoResult<World> {
        Ok(World {
            users: HashMap::new(),
            games: HashMap::new(),
            monsters: Vec::new(),
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

    pub fn add_monster(&mut self, monster: Monster) -> IoResult<uint> {
        self.monsters.push(monster);
        Ok(self.monsters.len() - 1)
    }

    pub fn get_entity(&mut self, identifier: &str) -> IoResult<&mut Entity> {
        if identifier.starts_with("@") {
            let i = match from_str(identifier.slice_from(1)) {
                Some(n) => n,
                None => 0,
            };
            if i < self.monsters.len() {
                Ok(self.monsters.get_mut(i) as &mut Entity)
            } else {
                Err(IoError {
                    kind: InvalidInput,
                    desc: "No such monster.",
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
}

#[cfg(test)]
mod test {
    use data::player::Player;
    use data::world::World;

    #[test]
    fn world_user_test() {
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
}
