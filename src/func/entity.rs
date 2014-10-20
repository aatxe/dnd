use data::{Basic, BotResult, Entity, Propagated, RollType, as_io};
use data::stats::Stats;
use data::utils::str_to_u8;
use data::world::World;
use func::{Functionality, incorrect_format, permissions_test, permissions_test_rf};
use irc::Bot;

pub struct Roll<'a> {
    bot: &'a Bot + 'a,
    chan: &'a str,
    target: &'a Entity + 'a,
    stat_str: Option<&'a str>,
    stat: Option<RollType>,
}

impl <'a> Roll<'a> {
    pub fn new(bot: &'a Bot, user: &'a str, chan: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Roll<'a>> {
        if args.len() > 3 {
            return Err(Propagated(format!("{}", chan),
                   format!("Invalid format. Use '.roll [@monster]' or '.roll [@monster] (stat)'.")))
        }
        Ok(Roll {
            bot: bot,
            chan: chan,
            target: {
                let (res, err) = if args.len() == 3 || args.len() == 2 && args[1].starts_with("@") {
                    if let Err(perm) = permissions_test_rf(user, chan, world) {
                        return Err(perm);
                    }
                    (world.get_entity(args[1], Some(chan)),
                     format!("{} is not a valid monster.", args[1]))
                } else {
                    (world.get_entity(user, None), format!("{} is not logged in.", user))
                };
                if res.is_ok() {
                    try!(res)
                } else {
                    return Err(Propagated(format!("{}", chan), err));
                }
            },
            stat_str: if args.len() == 3 && args[1].starts_with("@") {
                Some(args[2])
            } else if args.len() == 2 && !args[1].starts_with("@") {
                Some(args[1])
            } else {
                None
            },
            stat: if args.len() == 3 && args[1].starts_with("@") {
                RollType::to_roll_type(args[2])
            } else if args.len() == 2 && !args[1].starts_with("@") {
                RollType::to_roll_type(args[1])
            } else {
                Some(Basic)
            }
        })
    }
}

impl <'a> Functionality for Roll<'a> {
    fn do_func(&self) -> BotResult<()> {
        if self.stat.is_none() {
            return Err(Propagated(
                format!("{}", self.chan),
                format!("{} is not a valid stat.\r\nOptions: str dex con wis int cha (or their full names).", self.stat_str.unwrap())
            )); // We do not check if self.stat_str is none because it cannot be based on new(...).
        }
        let r = self.target.roll(self.stat.unwrap());
        try!(as_io(
            self.bot.send_privmsg(self.chan, format!("{} rolled {}.", self.target.identifier(), r).as_slice())
        ));
        Ok(())
    }
}

pub fn damage(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> BotResult<()> {
    if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
    if params.len() == 3 {
        let res = world.get_entity(params[1], Some(chan));
        if res.is_ok() {
            let e = try!(res);
            if let Some(n) = from_str(params[2]) {
                let m = if e.damage(n) {
                    format!("{} ({}) took {} damage and has {} health remaining.", e.identifier(), params[1], params[2], e.stats().health)
                } else {
                    format!("{} ({}) has fallen unconscious.", e.identifier(), params[1])
                };
                try!(as_io(bot.send_privmsg(chan, m.as_slice())));
            } else {
                try!(as_io(
                    bot.send_privmsg(chan, format!("{} is not a valid positive integer.", params[2]).as_slice())
                ));
            }
        } else {
            let m = if params[1].starts_with("@") {
                format!("{} is not a valid monster.", params[1])
            } else {
                format!("{} is not logged in.", params[1])
            };
            try!(as_io(bot.send_privmsg(chan, m.as_slice())));
        }
    } else {
        try!(incorrect_format(bot, chan, ".damage", "target value"));
    }
    Ok(())
}

pub fn set_temp_stats(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> BotResult<()> {
    if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
    if params.len() == 9 {
        let res = world.get_entity(params[1], Some(chan));
        if res.is_ok() {
            let p = try!(res);
            let mut valid = true;
            for s in params.slice_from(3).iter() {
                if str_to_u8(*s) == 0 {
                    valid = false;
                }
            }
            if valid {
                p.set_temp_stats(Stats::new(str_to_u8(params[2]),
                                            str_to_u8(params[3]), str_to_u8(params[4]),
                                            str_to_u8(params[5]), str_to_u8(params[6]),
                                            str_to_u8(params[7]), str_to_u8(params[8])));
                try!(as_io(
                    bot.send_privmsg(chan, format!("{} ({}) now has temporary {}.", p.identifier(), params[1], p.stats()).as_slice())
                ));
            } else {
                try!(as_io(
                    bot.send_privmsg(chan, "Stats must be non-zero positive integers. Format is: ")
                ));
                try!(as_io(bot.send_privmsg(chan, ".temp target health str dex con wis int cha")));
            }
        } else {
            let m = if params[1].starts_with("@") {
                format!("{} is not a valid monster.", params[1])
            } else {
                format!("{} is not logged in.", params[1])
            };
            try!(as_io(bot.send_privmsg(chan, m.as_slice())));
        }
    } else {
        try!(incorrect_format(bot, chan, ".temp", "target health str dex con wis int cha"));
    }
    Ok(())
}

pub fn clear_temp_stats(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> BotResult<()> {
    if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
    if params.len() == 2 {
        let res = world.get_entity(params[1], Some(chan));
        if res.is_ok() {
            let p = try!(res);
            p.clear_temp_stats();
            try!(as_io(
                bot.send_privmsg(chan, format!("{} ({}) has reverted to {}.", p.identifier(), params[1], p.stats()).as_slice())
            ));
        } else {
            let m = if params[1].starts_with("@") {
                format!("{} is not a valid monster.", params[1])
            } else {
                format!("{} is not logged in.", params[1])
            };
            try!(as_io(bot.send_privmsg(chan, m.as_slice())));
        }
    } else {
        try!(incorrect_format(bot, chan, ".cleartemp", "target"));
    }
    Ok(())
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
        assert_eq!(data.slice_to(27), "PRIVMSG #test :Test rolled ".as_bytes());
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
        assert_eq!(data.slice_to(27), "PRIVMSG #test :Test rolled ".as_bytes());
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
        assert_eq!(data.as_slice(), exp.as_bytes());
    }

    #[test]
    fn roll_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll @0\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :@0 is not a valid monster.\r\n".as_bytes());
    }

    #[test]
    fn roll_failed_user_is_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test is not logged in.\r\n".as_bytes());
    }

    #[test]
    fn roll_failed_invalid_format() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll a b c\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :Invalid format. Use '.roll [@monster]' or '.roll [@monster] (stat)'.\r\n".as_bytes());
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
        assert_eq!(data.as_slice(), "PRIVMSG #test :Test (@0) took 5 damage and has 15 health remaining.\r\n".as_bytes());
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
        assert_eq!(data.as_slice(), "PRIVMSG #test :Test (@0) has fallen unconscious.\r\n".as_bytes());
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
        assert_eq!(data.as_slice(), "PRIVMSG #test :a is not a valid positive integer.\r\n".as_bytes());
    }

    #[test]
    fn damage_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.damage @0 3\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :@0 is not a valid monster.\r\n".as_bytes());
    }

    #[test]
    fn damage_failed_user_is_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.damage test 15\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test is not logged in.\r\n".as_bytes());
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
        assert_eq!(data.as_slice(), "PRIVMSG #test :Test (@0) now has temporary Stats { health: 20, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }.\r\n".as_bytes());
    }

    #[test]
    fn set_temp_stats_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.temp @0 20 12 12 12 12 12 12\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :@0 is not a valid monster.\r\n".as_bytes());
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
        let mut exp = String::from_str("PRIVMSG #test :Stats must be non-zero positive integers. Format is: \r\n");
        exp.push_str("PRIVMSG #test :.temp target health str dex con wis int cha\r\n");
        assert_eq!(data.as_slice(), exp.as_bytes());
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
        assert_eq!(data.as_slice(), "PRIVMSG #test :Test (@0) has reverted to Stats { health: 14, strength: 12, dexterity: 10, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }.\r\n".as_bytes());
    }

    #[test]
    fn clear_temp_stats_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.cleartemp @0\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :@0 is not a valid monster.\r\n".as_bytes());
    }

    #[test]
    fn clear_temp_stats_failed_user_is_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.cleartemp test\r\n",
            |world| {
                world.add_game("Test", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test is not logged in.\r\n".as_bytes());
    }
}
