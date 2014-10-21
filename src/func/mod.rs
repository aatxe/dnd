extern crate irc;

use self::entity::{ClearTempStats, Damage, Roll, SetTempStats};
use self::monster::{AddMonster, LookUpMonster};
use self::player::{AddFeat, AddUpdate, Login, Logout, LookUpPlayer, Register, Save};
use self::world::{Create, PrivateRoll, SaveAll};
use std::io::IoResult;
use data::{BotError, BotResult, Entity, Propagated, as_io, to_io};
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
            try!(to_io(if !chan.starts_with("#") {
                match tokens[0] {
                    "register" => {
                        let register = Register::new(bot, user, tokens);
                        if let Err(Propagated(resp, msg)) = register {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = register.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "login" => {
                        let login = Login::new(bot, user, tokens, world);
                        if let Err(Propagated(resp, msg)) = login {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = login.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "create" => {
                        let create = Create::new(bot, user, tokens, world);
                        if let Err(Propagated(resp, msg)) = create {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = create.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "logout" => {
                        let logout = Logout::new(bot, user, world);
                        if let Err(Propagated(resp, msg)) = logout {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = logout.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "addfeat" => {
                        let addfeat = AddFeat::new(bot, user, tokens, world);
                        if let Err(Propagated(resp, msg)) = addfeat {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = addfeat.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "roll" => {
                        let roll = PrivateRoll::new(bot, user);
                        if let Err(Propagated(resp, msg)) = roll {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = roll.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "saveall" => {
                        let saveall = SaveAll::new(bot, user, world);
                        if let Err(Propagated(resp, msg)) = saveall {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = saveall.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "save" => {
                        let save = Save::new(bot, user, world);
                        if let Err(Propagated(resp, msg)) = save {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = save.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "lookup" => {
                        let lookup = LookUpPlayer::new(bot, user, tokens, world);
                        if let Err(Propagated(resp, msg)) = lookup {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = lookup.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "mlookup" => {
                        let mlookup = LookUpMonster::new(bot, user, tokens, world);
                        if let Err(Propagated(resp, msg)) = mlookup {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = mlookup.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    "addmonster" => {
                        let add = AddMonster::new(bot, user, tokens, world);
                        if let Err(Propagated(resp, msg)) = add {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        } else if let Err(Propagated(resp, msg)) = add.unwrap().do_func() {
                            try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                        };
                        Ok(())
                    },
                    _ => Ok(())
                }
            } else {
                if tokens[0].starts_with(".") {
                    match tokens[0].slice_from(1) {
                        "roll" => {
                            let roll = Roll::new(bot, user, chan, tokens, world);
                            if let Err(Propagated(resp, msg)) = roll {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            } else if let Err(Propagated(resp, msg)) = roll.unwrap().do_func() {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            };
                            Ok(())
                        },
                        "lookup" => {
                            let lookup = LookUpPlayer::new(bot, chan, tokens, world);
                            if let Err(Propagated(resp, msg)) = lookup {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            } else if let Err(Propagated(resp, msg)) = lookup.unwrap().do_func() {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            };
                            Ok(())
                        },
                        "update" => {
                            let update = AddUpdate::new(bot, user, chan, tokens, world, true);
                            if let Err(Propagated(resp, msg)) = update {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            } else if let Err(Propagated(resp, msg)) = update.unwrap().do_func() {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            };
                            Ok(())
                        },
                        "increase" => {
                            let update = AddUpdate::new(bot, user, chan, tokens, world, false);
                            if let Err(Propagated(resp, msg)) = update {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            } else if let Err(Propagated(resp, msg)) = update.unwrap().do_func() {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            };
                            Ok(())
                        },
                        "temp" => {
                            let temp = SetTempStats::new(bot, user, chan, tokens, world);
                            if let Err(Propagated(resp, msg)) = temp {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            } else if let Err(Propagated(resp, msg)) = temp.unwrap().do_func() {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            };
                            Ok(())
                        },
                        "cleartemp" => {
                            let clear = ClearTempStats::new(bot, user, chan, tokens, world);
                            if let Err(Propagated(resp, msg)) = clear {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            } else if let Err(Propagated(resp, msg)) = clear.unwrap().do_func() {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            };
                            Ok(())
                        },
                        "damage" => {
                            let damage = Damage::new(bot, user, chan, tokens, world);
                            if let Err(Propagated(resp, msg)) = damage {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            } else if let Err(Propagated(resp, msg)) = damage.unwrap().do_func() {
                                try!(bot.send_privmsg(resp.as_slice(), msg.as_slice()));
                            };
                            Ok(())
                        },
                        _ => Ok(())
                    }
                } else {
                    Ok(())
                }
            }));
        },
        _ => (),
    }
    Ok(())
}

fn get_target<'a>(maybe: &str, fallback: &str, resp: &str, chan: &str, world: &'a mut World) -> BotResult<&'a mut Entity + 'a> {
    let (res, err) = if maybe.starts_with("@") {
        if let Err(perm) = permissions_test_rf(fallback, chan, world) { return Err(perm); }
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

pub fn permissions_test_rf(user: &str, chan: &str, world: &mut World) -> BotResult<()> {
    let res = world.get_game(chan);
    if res.is_err() {
        Err(Propagated(String::from_str(user), format!("There is no game in {}.", chan)))
    } else if !try!(res).is_dm(user) {
        Err(Propagated(String::from_str(user), String::from_str("You must be the DM to do that!")))
    } else {
        Ok(())
    }
}

pub fn permissions_test(bot: &Bot, user: &str, chan: &str, world: &mut World) -> BotResult<bool> {
    let mut ret = true;
    let res = world.get_game(chan);
    if res.is_err() {
        try!(as_io(bot.send_privmsg(user, format!("There is no game in {}.", chan).as_slice())));
        ret = false;
    } else if !try!(res).is_dm(user) {
        try!(as_io(bot.send_privmsg(user, "You must be the DM to do that!")));
        ret = false;
    };
    Ok(ret)
}

pub fn incorrect_format_rf(resp: &str, cmd: &str, format: &str) -> BotError {
    Propagated(
        format!("{}", resp),
        format!("Incorrect format for {}. Format is:\r\n{} {}", cmd, cmd, format),
    )
}

pub fn incorrect_format(bot: &Bot, resp: &str, cmd: &str, format: &str) -> BotResult<()> {
    try!(as_io(
        bot.send_privmsg(resp, format!("Incorrect format for {}. Format is:", cmd).as_slice())
    ));
    try!(as_io(bot.send_privmsg(resp, format!("{} {}", cmd, format).as_slice())));
    Ok(())
}

#[cfg(test)]
mod test {
    use super::process_world;
    use std::io::{MemReader, MemWriter};
    use std::io::util::NullReader;
    use data::{BotResult, as_io};
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
        let bot = IrcBot::from_connection(Connection::new(MemWriter::new(), NullReader).unwrap(), |_, _, _, _| {
            Ok(())
        }).unwrap();
        assert!(!super::permissions_test(&bot, "test", "#test", &mut World::new()).unwrap());
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG test :There is no game in #test.\r\n".as_bytes());
    }

    #[test]
    fn permissions_test_not_dm() {
        let mut world = World::new();
        let bot = IrcBot::from_connection(Connection::new(MemWriter::new(), NullReader).unwrap(), |_, _, _, _| {
            Ok(())
        }).unwrap();
        world.add_game("Test", "test", "#test");
        assert!(!super::permissions_test(&bot, "test2", "#test", &mut world).unwrap());
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG test2 :You must be the DM to do that!\r\n".as_bytes());
    }

    #[test]
    fn permissions_test_success() {
        let mut world = World::new();
        let bot = IrcBot::from_connection(Connection::new(MemWriter::new(), NullReader).unwrap(), |_, _, _, _| {
            Ok(())
        }).unwrap();
        world.add_game("Test", "test", "#test");
        assert!(super::permissions_test(&bot, "test", "#test", &mut world).unwrap());
    }

    #[test]
    fn incorrect_format() {
        let bot = IrcBot::from_connection(Connection::new(MemWriter::new(), NullReader).unwrap(), |_, _, _, _| {
            Ok(())
        }).unwrap();
        super::incorrect_format(&bot, "test", "a", "b c").unwrap();
        let mut exp = String::from_str("PRIVMSG test :Incorrect format for a. Format is:\r\n");
        exp.push_str("PRIVMSG test :a b c\r\n");
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), exp.as_bytes());
    }
}
