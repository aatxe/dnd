extern crate irc;

use self::entity::{ClearTempStats, Damage, Roll, SetTempStats};
use self::monster::{AddMonster, LookUpMonster};
use self::player::{AddFeat, AddUpdate, Login, Logout, LookUpPlayer, Register, Save};
use self::world::{Create, PrivateRoll, SaveAll};
use std::io::IoResult;
use data::{BotError, BotResult, Entity, NotFound, Propagated};
use data::utils::str_to_u8;
use data::world::World;
use irc::Bot;
use irc::bot::IrcBot;
use irc::data::{IrcReader, IrcWriter};

pub mod entity;
pub mod monster;
pub mod player;
pub mod world;

pub trait Functionality {
    fn do_func(&mut self) -> BotResult<()>;
}

pub fn process_world<T, U>(bot: &IrcBot<T, U>, source: &str, command: &str, args: &[&str], world: &mut World) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    match (command, args) {
        ("PRIVMSG", [chan, msg]) => {
            let user = match source.find('!') {
                Some(i) => source.slice_to(i),
                None => "",
            };
            let tokens: Vec<&str> = msg.split_str(" ").collect();
            let func = if !chan.starts_with("#") {
                match tokens[0] {
                    "register" => Register::new(bot, user, tokens),
                    "login" => Login::new(bot, user, tokens, world),
                    "create" => Create::new(bot, user, tokens, world),
                    "logout" => Logout::new(bot, user, world),
                    "addfeat" => AddFeat::new(bot, user, tokens, world),
                    "roll" => PrivateRoll::new(bot, user),
                    "saveall" => SaveAll::new(bot, user, world),
                    "save" => Save::new(bot, user, world),
                    "lookup" => LookUpPlayer::new(bot, user, tokens, world),
                    "mlookup" => LookUpMonster::new(bot, user, tokens, world),
                    "addmonster" => AddMonster::new(bot, user, tokens, world),
                    _ => Err(Propagated(format!("{}", user), format!("{} is not a valid command.", tokens[0])))
                }
            } else {
                if tokens[0].starts_with(".") {
                    match tokens[0].slice_from(1) {
                        "roll" => Roll::new(bot, user, chan, tokens, world),
                        "lookup" => LookUpPlayer::new(bot, chan, tokens, world),
                        "update" => AddUpdate::new(bot, user, chan, tokens, world, true),
                        "increase" => AddUpdate::new(bot, user, chan, tokens, world, false),
                        "temp" => SetTempStats::new(bot, user, chan, tokens, world),
                        "cleartemp" => ClearTempStats::new(bot, user, chan, tokens, world),
                        "damage" => Damage::new(bot, user, chan, tokens, world),
                        _ => Err(NotFound(tokens[0].into_string()))
                    }
                } else {
                    Err(NotFound(tokens[0].into_string()))
                }
            };
            if let Err(Propagated(resp, msg)) = func {
                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
            } else if let Err(Propagated(resp, msg)) = func.unwrap().do_func() {
                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
            };
        },
        _ => (),
    }
    Ok(())
}

fn get_target<'a>(maybe: &str, fallback: &str, resp: &str, chan: &str, world: &'a mut World) -> BotResult<&'a mut Entity + 'a> {
    let (res, err) = if maybe.starts_with("@") {
        if let Err(perm) = permissions_test(fallback, chan, world) { return Err(perm); }
        (world.get_entity(maybe, Some(chan)), format!("{} is not a valid monster.", maybe))
    } else {
        (world.get_entity(fallback, None), format!("{} is not logged in.", fallback))
    };
    if res.is_ok() { res } else { Err(Propagated(format!("{}", resp), err)) }
}

pub fn validate_from(args: Vec<&str>, from: uint, resp: &str, cmd: &str, format: &str) -> BotResult<()> {
    for s in args.slice_from(from).iter() {
        if str_to_u8(*s) == 0 {
            return Err(Propagated(
                format!("{}", resp),
                format!("Stats must be non-zero positive integers. Format is:\r\n{} {}", cmd, format)
            ));
        }
    }
    Ok(())
}

pub fn permissions_test(user: &str, chan: &str, world: &mut World) -> BotResult<()> {
    let res = world.get_game(chan);
    if res.is_err() {
        Err(Propagated(String::from_str(user), format!("There is no game in {}.", chan)))
    } else if !try!(res).is_dm(user) {
        Err(Propagated(String::from_str(user), String::from_str("You must be the DM to do that!")))
    } else {
        Ok(())
    }
}

pub fn incorrect_format(resp: &str, cmd: &str, format: &str) -> BotError {
    Propagated(
        format!("{}", resp),
        format!("Incorrect format for {}. Format is:\r\n{} {}", cmd, cmd, format),
    )
}

#[cfg(test)]
mod test {
    use super::process_world;
    use std::io::{MemReader, MemWriter};
    use data::{BotResult, Propagated, as_io};
    use data::world::World;
    use irc::Bot;
    use irc::bot::IrcBot;
    use irc::conn::Connection;

    pub fn test_helper(input: &str, world_hook: |&mut World| -> BotResult<()>) -> BotResult<Vec<u8>> {
        let mut world = World::new();
        try!(world_hook(&mut world));
        let mut bot = try!(as_io(
            IrcBot::from_connection(try!(as_io(
                Connection::new(MemWriter::new(), MemReader::new(input.as_bytes().to_vec()))
            )), |bot, source, command, args| {
                process_world(bot, source, command, args, &mut world)
            })
        ));
        try!(as_io(bot.output()));
        Ok(bot.conn.writer().deref().get_ref().to_vec())
    }

    #[test]
    fn permissions_test_no_game() {
        let res = super::permissions_test("test", "#test", &mut World::new());
        assert!(res.is_err());
        if let Propagated(left, right) = res.unwrap_err() {
            assert_eq!(left, format!("test"))
            assert_eq!(right, format!("There is no game in #test."))
        } else {
            fail!("permissions_test(...) returned an unexpected error type.");
        }
    }

    #[test]
    fn permissions_test_not_dm() {
        let mut world = World::new();
        world.add_game("Test", "test", "#test");
        let res = super::permissions_test("test2", "#test", &mut world);
        assert!(res.is_err());
        if let Propagated(left, right) = res.unwrap_err() {
            assert_eq!(left, format!("test2"))
            assert_eq!(right, format!("You must be the DM to do that!"))
        } else {
            fail!("permissions_test(...) returned an unexpected error type.");
        }
    }

    #[test]
    fn permissions_test_success() {
        let mut world = World::new();
        world.add_game("Test", "test", "#test");
        assert!(super::permissions_test("test", "#test", &mut world).is_ok());
    }

    #[test]
    fn incorrect_format() {
        let res = super::incorrect_format("test", "a", "b c");
        if let Propagated(left, right) = res {
            assert_eq!(left, format!("test"))
            assert_eq!(right, format!("Incorrect format for a. Format is:\r\na b c"))
        } else {
            fail!("incorrect_format(...) returned an unexpected error type.");
        }
    }
}
