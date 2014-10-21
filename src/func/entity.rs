use data::{Basic, BotResult, Entity, Propagated, RollType, as_io};
use data::stats::Stats;
use data::utils::str_to_u8;
use data::world::World;
use func::Functionality;
use func::utils::{get_target, incorrect_format, permissions_test, validate_from};
use irc::Bot;

pub struct Roll<'a> {
    bot: &'a Bot + 'a,
    chan: &'a str,
    target: &'a Entity + 'a,
    stat_str: Option<&'a str>,
    stat: Option<RollType>,
}

impl <'a> Roll<'a> {
    pub fn new(bot: &'a Bot, user: &'a str, chan: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if args.len() > 3 {
            return Err(Propagated(format!("{}", chan),
                   format!("Invalid format. Use '.roll [@monster]' or '.roll [@monster] (stat)'.")))
        }
        let (stat_str, stat) = if args.len() == 3 && args[1].starts_with("@") {
            (Some(args[2]), RollType::to_roll_type(args[2]))
        } else if args.len() == 2 && !args[1].starts_with("@") {
            (Some(args[1]), RollType::to_roll_type(args[1]))
        } else {
            (None, Some(Basic))
        };
        Ok(box Roll {
            bot: bot,
            chan: chan,
            target: try!(get_target(if args.len() > 1 { args[1] } else { "" }, user, chan, chan, world)),
            stat_str: stat_str,
            stat: stat,
        } as Box<Functionality>)
    }
}

impl <'a> Functionality for Roll<'a> {
    fn do_func(&mut self) -> BotResult<()> {
        if self.stat.is_none() {
            return Err(Propagated(
                format!("{}", self.chan),
                format!("{} is not a valid stat.\r\nOptions: str dex con wis int cha (or their full names).", self.stat_str.unwrap())
            )); // We do not check if self.stat_str is none because it cannot be based on new(...).
        }
        let s = format!("{} rolled {}.",
                        self.target.identifier(), self.target.roll(self.stat.unwrap()));
        as_io(self.bot.send_privmsg(self.chan, s.as_slice()))
    }
}

pub struct Damage<'a> {
    bot: &'a Bot + 'a,
    chan: &'a str,
    target_str: &'a str,
    target: &'a mut Entity + 'a,
    value: u8,
}

impl <'a> Damage<'a> {
    pub fn new(bot: &'a Bot, user: &'a str, chan: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 3 { return Err(incorrect_format(chan, ".damage", "target value")); }
        Ok(box Damage {
            bot: bot,
            chan: chan,
            target_str: args[1],
            target: try!(get_target(args[1], user, chan, chan, world)),
            value: if let Some(n) = from_str(args[2]) {
                n
            } else {
                return Err(Propagated(
                        format!("{}", chan),
                        format!("{} is not a valid positive integer.", args[2])
                ));
            },
        } as Box<Functionality>)
    }
}

impl <'a> Functionality for Damage<'a> {
    fn do_func(&mut self) -> BotResult<()> {
        let m = if self.target.damage(self.value) {
            format!("{} ({}) took {} damage and has {} health remaining.", self.target.identifier(),
                    self.target_str, self.value, self.target.stats().health)
        } else {
            format!("{} ({}) has fallen unconscious.", self.target.identifier(), self.target_str)
        };
        as_io(self.bot.send_privmsg(self.chan, m.as_slice()))
    }
}

pub struct SetTempStats<'a> {
    bot: &'a Bot + 'a,
    chan: &'a str,
    target_str: &'a str,
    target: &'a mut Entity + 'a,
    health: u8,
    st: u8, dx: u8, cn: u8,
    ws: u8, it: u8, ch: u8,
}

impl <'a> SetTempStats<'a> {
    pub fn new(bot: &'a Bot, user: &'a str, chan: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if let Err(perm) = permissions_test(user, chan, world) {
            return Err(perm);
        } else if args.len() != 9 {
            return Err(incorrect_format(chan, ".temp", "target health str dex con wis int cha"));
        }
        try!(validate_from(args.clone(), 3, chan, ".temp", "target health str dex con wis int cha"));
        Ok(box SetTempStats {
            bot: bot,
            chan: chan,
            target_str: args[1],
            target: try!(get_target(args[1], user, chan, chan, world)),
            health: str_to_u8(args[2]),
            st: str_to_u8(args[3]), dx: str_to_u8(args[4]), cn: str_to_u8(args[5]),
            ws: str_to_u8(args[6]), it: str_to_u8(args[7]), ch: str_to_u8(args[8]),
        } as Box<Functionality>)
    }
}

impl <'a> Functionality for SetTempStats<'a> {
    fn do_func(&mut self) -> BotResult<()> {
        self.target.set_temp_stats(Stats::new(self.health,
                                             self.st, self.dx, self.cn, self.ws, self.it, self.ch));
        let s = format!("{} ({}) now has temporary {}.",
                        self.target.identifier(), self.target_str, self.target.stats());
        as_io(self.bot.send_privmsg(self.chan, s.as_slice()))
    }
}

pub struct ClearTempStats<'a> {
    bot: &'a Bot + 'a,
    chan: &'a str,
    target_str: &'a str,
    target: &'a mut Entity + 'a,
}

impl <'a> ClearTempStats<'a> {
    pub fn new(bot: &'a Bot, user: &'a str, chan: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if let Err(perm) = permissions_test(user, chan, world) {
            return Err(perm);
        } else if args.len() != 2 {
            return Err(incorrect_format(chan, ".cleartemp", "target"));
        }
        Ok(box ClearTempStats {
            bot: bot,
            chan: chan,
            target_str: args[1],
            target: try!(get_target(args[1], user, chan, chan, world)),
        } as Box<Functionality>)
    }
}

impl <'a> Functionality for ClearTempStats<'a> {
    fn do_func(&mut self) -> BotResult<()> {
        self.target.clear_temp_stats();
        let s = format!("{} ({}) has reverted to {}.",
                        self.target.identifier(), self.target_str, self.target.stats());
        as_io(self.bot.send_privmsg(self.chan, s.as_slice()))
    }
}

#[cfg(test)]
mod test {
    use data::Entity;
    use data::monster::Monster;
    use data::stats::Stats;
    use func::test::test_helper;

    #[test]
    fn roll_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll @0\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12);
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data.slice_to(27).to_vec()), Ok(format!("PRIVMSG #test :Test rolled ")));
    }

    #[test]
    fn roll_success_stat() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll @0 con\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12);
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data.slice_to(27).to_vec()), Ok(format!("PRIVMSG #test :Test rolled ")));
    }

    #[test]
    fn roll_failed_invalid_stat() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll @0 test\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12);
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        let mut exp = String::from_str("PRIVMSG #test :test is not a valid stat.\r\n");
        exp.push_str("PRIVMSG #test :Options: str dex con wis int cha (or their full names).\r\n");
        assert_eq!(String::from_utf8(data), Ok(exp));
    }

    #[test]
    fn roll_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll @0\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :@0 is not a valid monster.\r\n")));
    }

    #[test]
    fn roll_failed_user_is_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :test is not logged in.\r\n")));
    }

    #[test]
    fn roll_failed_invalid_format() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll a b c\r\n", |_| { Ok(()) }).unwrap();
        let exp = String::from_str("PRIVMSG #test :Invalid format. Use '.roll [@monster]' or '.roll [@monster] (stat)'.\r\n");
        assert_eq!(String::from_utf8(data), Ok(exp));
    }

    #[test]
    fn damage_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.damage @0 5\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let m = Monster::create("Test", 20, 12, 12, 12, 12, 12, 12);
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :Test (@0) took 5 damage and has 15 health remaining.\r\n")));
    }

    #[test]
    fn damage_success_unconscious() {
        let data = test_helper(":test!test@test PRIVMSG #test :.damage @0 20\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let m = Monster::create("Test", 20, 12, 12, 12, 12, 12, 12);
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :Test (@0) has fallen unconscious.\r\n")));
    }

    #[test]
    fn damage_failed_invalid_amount() {
        let data = test_helper(":test!test@test PRIVMSG #test :.damage @0 a\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let m = Monster::create("Test", 20, 12, 12, 12, 12, 12, 12);
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :a is not a valid positive integer.\r\n")));
    }

    #[test]
    fn damage_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.damage @0 3\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :@0 is not a valid monster.\r\n")));
    }

    #[test]
    fn damage_failed_user_is_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.damage test 15\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :test is not logged in.\r\n")));
    }

    #[test]
    fn set_temp_stats_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.temp @0 20 12 12 12 12 12 12\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12);
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        let exp = String::from_str("PRIVMSG #test :Test (@0) now has temporary Stats { health: 20, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }.\r\n");
        assert_eq!(String::from_utf8(data), Ok(exp));
    }

    #[test]
    fn set_temp_stats_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.temp @0 20 12 12 12 12 12 12\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :@0 is not a valid monster.\r\n")));
    }

    #[test]
    fn set_temp_stats_failed_non_integers() {
        let data = test_helper(":test!test@test PRIVMSG #test :.temp @0 20 -12 a 12 12 12 12\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12);
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        let mut exp = String::from_str("PRIVMSG #test :Stats must be non-zero positive integers. Format is:\r\n");
        exp.push_str("PRIVMSG #test :.temp target health str dex con wis int cha\r\n");
        assert_eq!(String::from_utf8(data), Ok(exp));
    }

    #[test]
    fn clear_temp_stats_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.cleartemp @0\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                let mut m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12);
                m.set_temp_stats(Stats::new(20, 12, 12, 12, 12, 12, 12));
                world.add_monster(m, "#test");
                Ok(())
            }
        ).unwrap();
        let exp = String::from_str("PRIVMSG #test :Test (@0) has reverted to Stats { health: 14, strength: 12, dexterity: 10, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }.\r\n");
        assert_eq!(String::from_utf8(data), Ok(exp));
    }

    #[test]
    fn clear_temp_stats_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.cleartemp @0\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :@0 is not a valid monster.\r\n")));
    }

    #[test]
    fn clear_temp_stats_failed_user_is_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.cleartemp test\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(String::from_utf8(data), Ok(format!("PRIVMSG #test :test is not logged in.\r\n")));
    }
}
