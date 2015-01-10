extern crate irc;

use self::entity::{ClearTempStats, Damage, Move, Roll, SetTempStats};
use self::monster::{AddMonster, LookUpMonster};
use self::player::{AddFeat, AddUpdate, Login, Logout, LookUpPlayer, Register, Save};
use self::world::{Create, PrivateRoll, SaveAll};
use std::borrow::ToOwned;
use std::io::IoResult;
use data::{BotResult, as_io};
use data::BotError::{InvalidInput, NotFound, Propagated};
use data::world::World;
use irc::data::kinds::{IrcReader, IrcWriter};
use irc::server::Server;
use irc::server::utils::Wrapper;

pub mod entity;
pub mod monster;
pub mod player;
pub mod world;

pub trait Functionality {
    fn do_func(&mut self) -> BotResult<()>;
}

pub struct Help<'a, T: IrcReader, U: IrcWriter> {
    bot: &'a Wrapper<'a, T, U>,
    resp: &'a str,
    cmd: Option<&'a str>,
}

impl<'a, T: IrcReader, U: IrcWriter> Help<'a, T, U> {
    pub fn new(bot: &'a Wrapper<'a, T, U>, resp: &'a str, args: Vec<&'a str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 1 && args.len() != 2 { return Err(utils::incorrect_format(resp, "help", "[command]")); }
        Ok(box Help { bot: bot, resp: resp,
                      cmd: if args.len() == 2 { Some(args[1]) } else { None }
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for Help<'a, T, U> {
    fn do_func(&mut self) -> BotResult<()> {
        if let Some(cmd) = self.cmd {
            // FIXME: There has to be some way of improving this.
            let format: &str = if cmd.starts_with(".") {
                match &cmd[1..] {
                    "roll" => "[@monster] [stat]",
                    "lookup" => "target [stat]",
                    "update" => "stat value",
                    "increase" => "stat value",
                    "temp" => "target health str dex con wis int cha",
                    "cleartemp" => "target",
                    "damage" => "target value",
                    "move" => "[@monster] x y",
                    _ => return Err(Propagated(format!("{}", self.resp), format!("{} is not a valid command.", self.cmd.unwrap())))
                }
            } else {
                match cmd {
                    "register" => "username password health str dex con wis int cha",
                    "login" => "username password channel",
                    "create" => "channel campaign name",
                    "logout" => "",
                    "addfeat" => "name of feat",
                    "roll" => "",
                    "saveall" => "",
                    "save" => "",
                    "lookup" => "target [stat]",
                    "mlookup" => "channel target [stat]",
                    "addmonster" => "chan name health str dex con wis int cha",
                    _ => return Err(Propagated(format!("{}", self.resp), format!("{} is not a valid command.", self.cmd.unwrap())))
                }
            };
            as_io(self.bot.send_privmsg(self.resp, &format!("Format: {} {}", self.cmd.unwrap(), format)[]))
        } else {
            let mut s = String::from_str("List of Commands:\r\n");
            s.push_str("Channel commands: .roll .lookup .update .increase .temp .cleartemp .damage .move\r\n");
            s.push_str("Query commands: register login create logout addfeat roll saveall save lookup mlookup addmonster\r\n");
            s.push_str(&format!("If you need additional help, use {}help [command].", if self.resp.starts_with("#") { "." } else { "" })[]);
            as_io(self.bot.send_privmsg(self.resp, &s[]))
        }
    }
}

fn tokenize<'a>(line: &'a str, vec: &'a mut Vec<String>) -> BotResult<Vec<&'a str>> {
    /* FIXME: tokenizer removes multiple spaces in quoted tokens */
    let mut flag = false;
    let mut s = String::new();
    for token in line.split_str(" ") {
        if token.starts_with("\"") {
            s.push_str(token);
            s.push_str(" ");
            flag = true;
        } else if flag {
            s.push_str(token);
            if token.ends_with("\"") {
                vec.push(s[1..s.len() - 1].to_owned());
                s = String::new();
                flag = false;
            } else {
                s.push_str(" ");
            }
        } else {
            vec.push(token.to_owned());
        }
    }
    if s.len() != 0 {
        Err(InvalidInput("Could not tokenize malformed arguments.".to_owned()))
    } else {
        Ok(vec.iter().map(|s| &s[]).collect())
    }
}

pub fn process_world<'a, T: IrcReader, U: IrcWriter>(bot: &'a Wrapper<'a, T, U>, source: &'a str, 
    command: &str, args: &[&'a str], token_store: &'a mut Vec<String>, world: &'a mut World) 
    -> IoResult<()> {
    match (command, args) {
        ("PRIVMSG", [chan, msg]) => {
            let user = source.find('!').map_or("", |i| &source[..i]);
            let tokens = match tokenize(msg, token_store) {
                Err(InvalidInput(msg)) => return bot.send_privmsg(user, &msg[]),
                Err(_) => return bot.send_privmsg(user, "Something went seriously wrong."),
                Ok(tokens) => tokens,
            };
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
                    "help" => Help::new(bot, user, tokens),
                    _ => Err(Propagated(format!("{}", user), format!("{} is not a valid command.", tokens[0])))
                }
            } else {
                if tokens[0].starts_with(".") {
                    match &tokens[0][1..] {
                        "roll" => Roll::new(bot, user, chan, tokens, world),
                        "lookup" => LookUpPlayer::new(bot, chan, tokens, world),
                        "update" => AddUpdate::new(bot, user, chan, tokens, world, true),
                        "increase" => AddUpdate::new(bot, user, chan, tokens, world, false),
                        "temp" => SetTempStats::new(bot, user, chan, tokens, world),
                        "cleartemp" => ClearTempStats::new(bot, user, chan, tokens, world),
                        "damage" => Damage::new(bot, user, chan, tokens, world),
                        "move" => Move::new(bot, user, chan, tokens, world),
                        "help" => Help::new(bot, chan, tokens),
                        _ => Err(NotFound(tokens[0].to_owned()))
                    }
                } else {
                    Err(NotFound(tokens[0].to_owned()))
                }
            };
            if let Err(Propagated(resp, msg)) = func {
                try!(bot.send_privmsg(&resp[], &msg[]));
            } else if let Err(Propagated(resp, msg)) = func.and_then(|mut f| f.do_func()) {
                try!(bot.send_privmsg(&resp[], &msg[]));
            }
        },
        ("NOTICE", [_, suffix]) => {
            if suffix.starts_with("***") {
                try!(bot.identify());
            }
        }
        _ => (),
    }
    Ok(())
}

mod utils {
    use data::{BotError, BotResult, Entity};
    use data::BotError::Propagated;
    use data::utils::str_to_u8;
    use data::world::World;

    pub fn get_target<'a>(maybe: &str, fallback: &str, resp: &str, chan: &str, world: &'a mut World) -> BotResult<&'a mut (Entity + 'a)> {
        let (res, err) = if maybe.starts_with("@") {
            if let Err(perm) = permissions_test(fallback, chan, world) { return Err(perm); }
            (world.get_entity(maybe, Some(chan)), format!("{} is not a valid monster.", maybe))
        } else {
            (world.get_entity(fallback, None), format!("{} is not logged in.", fallback))
        };
        if res.is_ok() { res } else { Err(Propagated(format!("{}", resp), err)) }
    }

    pub fn validate_from(args: Vec<&str>, from: usize, resp: &str, cmd: &str, format: &str) -> BotResult<()> {
        for s in args[from..].iter() {
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
}

#[cfg(test)]
mod test {
    use super::process_world;
    use std::borrow::ToOwned;
    use std::default::Default;
    use std::io::{MemReader, MemWriter};
    use data::{BotResult};
    use data::BotError::Propagated;
    use data::world::World;
    use irc::conn::Connection;
    use irc::data::Config;
    use irc::server::{IrcServer, Server};
    use irc::server::utils::Wrapper;

    pub fn test_helper<F>(input: &str, world_hook: F) -> BotResult<String> 
        where F: FnOnce(&mut World) -> BotResult<()> {
        let mut world = World::new();
        try!(world_hook(&mut world));
        let server = IrcServer::from_connection(Config {
            owners: Some(vec!["test".to_owned()]),
            nickname: Some("test".to_owned()),  
            .. Default::default()
        }, Connection::new(MemReader::new(input.as_bytes().to_vec()), MemWriter::new()));
        for message in server.iter() {
            let message = message.unwrap();
            println!("{:?}", message);
            let mut args: Vec<_> = message.args.iter().map(|s| &s[]).collect();
            if let Some(ref suffix) = message.suffix {
                args.push(&suffix[])
            }
            let source = message.prefix.unwrap_or(String::new());
            let mut token_store = Vec::new();
            process_world(&Wrapper::new(&server), &source[], &message.command[], &args[], &mut token_store, &mut world).unwrap();
        }
        Ok(String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap())
    }

    #[test]
    fn tokenize() {
        let mut store = Vec::new();
        assert_eq!(super::tokenize("a bb ccc", &mut store), Ok(vec!("a", "bb", "ccc")));
        store = Vec::new();
        assert_eq!(super::tokenize("ab 3 ca", &mut store), Ok(vec!("ab", "3", "ca")));
        store = Vec::new();
        assert_eq!(super::tokenize("\"a b c\" d", &mut store), Ok(vec!("a b c", "d")));
        store = Vec::new();
        assert_eq!(super::tokenize("e \"a b c\" d", &mut store), Ok(vec!("e", "a b c", "d")));
        store = Vec::new();
        assert!(super::tokenize("\"a b c d", &mut store).is_err());
        store = Vec::new();
        assert!(super::tokenize("a \"b \"c d", &mut store).is_err());
    }

    #[test]
    fn permissions_test_no_game() {
        let res = super::utils::permissions_test("test", "#test", &mut World::new());
        assert!(res.is_err());
        if let Propagated(left, right) = res.unwrap_err() {
            assert_eq!(left, format!("test"));
            assert_eq!(right, format!("There is no game in #test."));
        } else {
            panic!("permissions_test(...) returned an unexpected error type.");
        }
    }

    #[test]
    fn permissions_test_not_dm() {
        let mut world = World::new();
        world.add_game("Test", "test", "#test");
        let res = super::utils::permissions_test("test2", "#test", &mut world);
        assert!(res.is_err());
        if let Propagated(left, right) = res.unwrap_err() {
            assert_eq!(left, format!("test2"));
            assert_eq!(right, format!("You must be the DM to do that!"));
        } else {
            panic!("permissions_test(...) returned an unexpected error type.");
        }
    }

    #[test]
    fn permissions_test_success() {
        let mut world = World::new();
        world.add_game("Test", "test", "#test");
        assert!(super::utils::permissions_test("test", "#test", &mut world).is_ok());
    }

    #[test]
    fn incorrect_format() {
        let res = super::utils::incorrect_format("test", "a", "b c");
        if let Propagated(left, right) = res {
            assert_eq!(left, format!("test"));
            assert_eq!(right, format!("Incorrect format for a. Format is:\r\na b c"));
        } else {
            panic!("incorrect_format(...) returned an unexpected error type.");
        }
    }

    #[test]
    fn non_command_message_in_channel() {
        let data = test_helper(":test!test@test PRIVMSG #test :Hi there!\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!(""));
    }

    #[test]
    fn non_command_message_in_query() {
        let data = test_helper(":test!test@test PRIVMSG test :Hi there!\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Hi is not a valid command.\r\n"));
    }

    #[test]
    fn specific_help_channel_command() {
        let data = test_helper(":test!test@test PRIVMSG test :help .roll\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Format: .roll [@monster] [stat]\r\n"));
    }

    #[test]
    fn specific_help_query_command() {
        let data = test_helper(":test!test@test PRIVMSG test :help register\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Format: register username password health str dex con wis int cha\r\n"));
    }

    #[test]
    fn specific_help_channel_command_not_found() {
        let data = test_helper(":test!test@test PRIVMSG test :help .test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :.test is not a valid command.\r\n"));
    }

    #[test]
    fn specific_help_query_command_not_found() {
        let data = test_helper(":test!test@test PRIVMSG test :help test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :test is not a valid command.\r\n"));
    }

    #[test]
    fn general_help_in_channel() {
        let data = test_helper(":test!test@test PRIVMSG #test :.help\r\n", |_| { Ok(()) }).unwrap();
        let mut exp = String::from_str("PRIVMSG #test :List of Commands:\r\n");
        exp.push_str("PRIVMSG #test :Channel commands: .roll .lookup .update .increase .temp .cleartemp .damage .move\r\n");
        exp.push_str("PRIVMSG #test :Query commands: register login create logout addfeat roll saveall save lookup mlookup addmonster\r\n");
        exp.push_str("PRIVMSG #test :If you need additional help, use .help [command].\r\n");
        assert_eq!(data, exp)
    }

    #[test]
    fn general_help_in_query() {
        let data = test_helper(":test!test@test PRIVMSG test :help\r\n", |_| { Ok(()) }).unwrap();
        let mut exp = String::from_str("PRIVMSG test :List of Commands:\r\n");
        exp.push_str("PRIVMSG test :Channel commands: .roll .lookup .update .increase .temp .cleartemp .damage .move\r\n");
        exp.push_str("PRIVMSG test :Query commands: register login create logout addfeat roll saveall save lookup mlookup addmonster\r\n");
        exp.push_str("PRIVMSG test :If you need additional help, use help [command].\r\n");
        assert_eq!(data, exp)
    }
}
