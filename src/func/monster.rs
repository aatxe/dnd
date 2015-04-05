use std::ascii::AsciiExt;
use data::{BotResult, Entity, as_io};
use data::BotError::Propagated;
use data::monster::Monster;
use data::utils::str_to_u8;
use data::world::World;
use func::Functionality;
use func::utils::{get_target, incorrect_format, permissions_test, validate_from};
use irc::client::prelude::*;

pub struct AddMonster<'a, T: IrcRead, U: IrcWrite> {
    bot: &'a ServerExt<'a, T, U>,
    user: &'a str,
    world: &'a mut World,
    chan: &'a str, name: &'a str,
    health: u8, movement: u8,
    st: u8, dx: u8, cn: u8,
    ws: u8, it: u8, ch: u8,
}

impl<'a, T: IrcRead, U: IrcWrite> AddMonster<'a, T, U> {
    pub fn new(bot: &'a ServerExt<'a, T, U>, user: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if let Err(perm) = permissions_test(user, args[1], world) {
            return Err(perm);
        } else if args.len() != 11 {
            return Err(incorrect_format(user, "addmonster",
                                        "chan name health movement str dex con wis int cha"));
        }
        try!(validate_from(args.clone(), 3, user, "addmonster",
                           "chan name health movement str dex con wis int cha"));
        Ok(Box::new(AddMonster {
            bot: bot,
            user: user,
            world: world,
            chan: args[1], name: args[2],
            health: str_to_u8(args[3]), movement: str_to_u8(args[4]),
            st: str_to_u8(args[5]), dx: str_to_u8(args[6]), cn: str_to_u8(args[7]),
            ws: str_to_u8(args[8]), it: str_to_u8(args[9]), ch: str_to_u8(args[10]),
        }))
    }
}

impl<'a, T: IrcRead, U: IrcWrite> Functionality for AddMonster<'a, T, U> {
    fn do_func(&mut self) -> BotResult<()> {
        let m = Monster::create(self.name, self.health, self.movement, self.st, self.dx, self.cn,
                                self.ws, self.it, self.ch);
        let s = format!("Monster ({}) has been created as @{}.",
                        self.name, self.world.add_monster(m, self.chan));
        as_io(self.bot.send_privmsg(self.user, &s))
    }
}

pub struct LookUpMonster<'a, T: IrcRead, U: IrcWrite> {
    bot: &'a ServerExt<'a, T, U>,
    user: &'a str,
    world: &'a mut World,
    chan: &'a str,
    target_str: &'a str,
    stat_str: Option<&'a str>,
}

impl<'a, T: IrcRead, U: IrcWrite> LookUpMonster<'a, T, U> {
    pub fn new(bot: &'a ServerExt<'a, T, U>, user: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 3 && args.len() != 4 {
            return Err(incorrect_format(user, "mlookup", "channel target [stat]"));
        } else if let Err(perm) = permissions_test(user, args[1], world) {
            return Err(perm);
        } else if !args[2].starts_with("@") {
            return Err(Propagated(format!("{}", user), format!("{} is not a valid monster.", args[2])));
        }
        Ok(Box::new(LookUpMonster {
            bot: bot,
            user: user,
            world: world,
            chan: args[1],
            target_str: args[2],
            stat_str: if args.len() == 4 {
                Some(args[3])
            } else {
                None
            },
        }))
    }
}

impl<'a, T: IrcRead, U: IrcWrite> Functionality for LookUpMonster<'a, T, U> {
    fn do_func(&mut self) -> BotResult<()> {
        let target = try!(get_target(self.target_str, self.user, self.user, self.chan, self.world));
        let temp = if target.has_temp_stats() {
            "Temp. "
        } else {
            ""
        };
        if self.stat_str.is_none() {
            let s = format!("{} ({}): {}{:?}", target.identifier(), self.target_str, temp, target.stats());
            as_io(self.bot.send_privmsg(self.user, &s))
        } else if self.stat_str.unwrap().eq_ignore_ascii_case("pos") || self.stat_str.unwrap().eq_ignore_ascii_case("position") {
            let s = format!("{} ({}): {:?}", target.identifier(), self.target_str, target.position());
            as_io(self.bot.send_privmsg(self.user, &s))
        } else if let Some(x) = target.stats().get_stat(self.stat_str.unwrap()) {
            let s = format!("{} ({}): {}{} {}", target.identifier(), self.target_str, temp, x, self.stat_str.unwrap());
            as_io(self.bot.send_privmsg(self.user, &s))
        } else {
            Err(Propagated(format!("{}", self.user), format!("{} is not a valid stat.", self.stat_str.unwrap())))
        }
    }
}

#[cfg(test)]
mod test {
    use data::Entity;
    use data::monster::Monster;
    use data::stats::Stats;
    use func::test::test_helper;

    #[test]
    fn add_success() {
        let data = test_helper(":test!test@test PRIVMSG test :addmonster #test Test 20 30 12 12 12 12 12 12\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Monster (Test) has been created as @0.\r\n"));
    }

    #[test]
    fn add_failed_non_integers() {
        let data = test_helper(":test!test@test PRIVMSG test :addmonster #test Test 20 30 -12 a 12 12 12 12\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        let mut exp = "PRIVMSG test :Stats must be non-zero positive integers. Format is:\r\n".to_string();
        exp.push_str("PRIVMSG test :addmonster chan name health movement str dex con wis int cha\r\n");
        assert_eq!(data, exp);
    }

    #[test]
    fn look_up_success() {
        let data = test_helper(":test!test@test PRIVMSG test :mlookup #test @0\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                world.add_monster(Monster::create("Test", 20, 30, 12, 12, 12, 12, 12, 12), "#test");
                Ok(())
            }
        ).unwrap();
        let exp = "PRIVMSG test :Test (@0): Stats { health: 20, movement: 30, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }\r\n".to_string();
        assert_eq!(data, exp);
    }

    #[test]
    fn look_up_failed_no_monster() {
        let data = test_helper(":test!test@test PRIVMSG test :mlookup #test @1\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                world.add_monster(Monster::create("Test", 20, 30, 12, 12, 12, 12, 12, 12), "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :@1 is not a valid monster.\r\n"));
    }

    #[test]
    fn look_up_success_by_stat() {
        let data = test_helper(":test!test@test PRIVMSG test :mlookup #test @0 health\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                world.add_monster(Monster::create("Test", 20, 30, 12, 12, 12, 12, 12, 12), "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Test (@0): 20 health\r\n"));
    }

    #[test]
    fn look_up_failed_invalid_stat() {
        let data = test_helper(":test!test@test PRIVMSG test :mlookup #test @0 test\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                world.add_monster(Monster::create("Test", 20, 30, 12, 12, 12, 12, 12, 12), "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :test is not a valid stat.\r\n"));
    }

    #[test]
    fn look_up_success_temporary() {
        let data = test_helper(":test!test@test PRIVMSG test :mlookup #test @0\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let mut m = Monster::create("Test", 14, 30, 12, 10, 12, 12, 12, 12);
                m.set_temp_stats(Stats::new(20, 30, 12, 12, 12, 12, 12, 12));
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        let exp = "PRIVMSG test :Test (@0): Temp. Stats { health: 20, movement: 30, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }\r\n".to_string();
        assert_eq!(data, exp);
    }

    #[test]
    fn look_up_success_temporary_stat() {
        let data = test_helper(":test!test@test PRIVMSG test :mlookup #test @0 health\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let mut m = Monster::create("Test", 14, 30, 12, 10, 12, 12, 12, 12);
                m.set_temp_stats(Stats::new(20, 30, 12, 12, 12, 12, 12, 12));
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Test (@0): Temp. 20 health\r\n"));
    }

    #[test]
    fn look_up_position() {
        let data = test_helper(":test!test@test PRIVMSG test :mlookup #test @0 pos\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                world.add_monster(Monster::create("Test", 20, 30, 12, 12, 12, 12, 12, 12), "#test");
                Ok(())
            }
        ).unwrap();
        let exp = "PRIVMSG test :Test (@0): Position(0, 0)\r\n".to_string();
        assert_eq!(data, exp);
    }
}
