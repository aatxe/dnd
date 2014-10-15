use std::io::IoResult;
use data::{Basic, Entity, RollType};
use data::stats::Stats;
use data::utils::str_to_u8;
use data::world::World;
use func::{incorrect_format, permissions_test};
use irc::Bot;

pub fn roll(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 1 || (params.len() == 2 && params[1].starts_with("@")) {
        let res = if params.len() == 2 {
            if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
            world.get_entity(params[1], Some(chan))
        } else {
            world.get_entity(user, None)
        };
        if res.is_ok() {
            let e = try!(res);
            let r = e.roll(Basic);
            try!(bot.send_privmsg(chan, format!("{} rolled {}.", e.identifier(), r).as_slice()));
        } else {
            let m = if params.len() == 2 {
                format!("{} is not a valid monster.", params[1])
            } else {
                format!("{} is not logged in.", user)
            };
            try!(bot.send_privmsg(chan, m.as_slice()));
        }
    } else if params.len() == 2 || (params.len() == 3 && params[1].starts_with("@")) {
        let res = if params.len() == 3 {
            if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
            world.get_entity(params[1], Some(chan))
        } else {
            world.get_entity(user, None)
        };
        if res.is_ok() {
            let e = try!(res);
            let stat = if params.len() == 3 {
                params[2]
            } else {
                params[1]
            };
            let rt = RollType::to_roll_type(stat);
            match rt {
                Some(roll_type) => {
                    let r = e.roll(roll_type);
                    try!(bot.send_privmsg(chan, format!("{} rolled {}.", e.identifier(), r).as_slice()));
                },
                None => {
                    try!(bot.send_privmsg(chan, format!("{} is not a valid stat.", stat).as_slice()));
                    try!(bot.send_privmsg(chan, "Options: str dex con wis int cha (or their full names)."));
                }
            }
        } else {
            let m = if params.len() == 3 {
                format!("{} is not a valid monster.", params[1])
            } else {
                format!("{} is not logged in.", user)
            };
            try!(bot.send_privmsg(chan, m.as_slice()));
        }
    } else {
        try!(bot.send_privmsg(chan, "Invalid format. Use '.roll [@monster]' or '.roll [@monster] (stat)'."));
    }
    Ok(())
}

pub fn damage(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
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
                try!(bot.send_privmsg(chan, m.as_slice()));
            } else {
                try!(bot.send_privmsg(chan, format!("{} is not a valid positive integer.", params[2]).as_slice()));
            }
        } else {
            let m = if params[1].starts_with("@") {
                format!("{} is not a valid monster.", params[1])
            } else {
                format!("{} is not logged in.", params[1])
            };
            try!(bot.send_privmsg(chan, m.as_slice()));
        }
    } else {
        try!(incorrect_format(bot, chan, ".damage", "target value"));
    }
    Ok(())
}

pub fn set_temp_stats(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
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
                p.set_temp_stats(try!(Stats::new(str_to_u8(params[2]),
                                                 str_to_u8(params[3]), str_to_u8(params[4]),
                                                 str_to_u8(params[5]), str_to_u8(params[6]),
                                                 str_to_u8(params[7]), str_to_u8(params[8]))));
                try!(bot.send_privmsg(chan, format!("{} ({}) now has temporary {}.", p.identifier(), params[1], p.stats()).as_slice()));
            } else {
                try!(bot.send_privmsg(chan, "Stats must be non-zero positive integers. Format is: "))
                try!(bot.send_privmsg(chan, ".temp target health str dex con wis int cha"));
            }
        } else {
            let m = if params[1].starts_with("@") {
                format!("{} is not a valid monster.", params[1])
            } else {
                format!("{} is not logged in.", params[1])
            };
            try!(bot.send_privmsg(chan, m.as_slice()));
        }
    } else {
        try!(incorrect_format(bot, chan, ".temp", "target health str dex con wis int cha"));
    }
    Ok(())
}

pub fn clear_temp_stats(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
    if params.len() == 2 {
        let res = world.get_entity(params[1], Some(chan));
        if res.is_ok() {
            let p = try!(res);
            p.clear_temp_stats();
            try!(bot.send_privmsg(chan, format!("{} ({}) has reverted to {}.", p.identifier(), params[1], p.stats()).as_slice()));
        } else {
            let m = if params[1].starts_with("@") {
                format!("{} is not a valid monster.", params[1])
            } else {
                format!("{} is not logged in.", params[1])
            };
            try!(bot.send_privmsg(chan, m.as_slice()));
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
                try!(world.add_game("Test", "test", "#test"));
                let m = try!(Monster::create("Test", 14, 12, 10, 12, 12, 12, 12));
                try!(world.add_monster(m, "#test"));
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.slice_to(27), "PRIVMSG #test :Test rolled ".as_bytes());
    }

    #[test]
    fn roll_success_stat() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll @0 con\r\n",
            |world| {
                try!(world.add_game("Test", "test", "#test"));
                let m = try!(Monster::create("Test", 14, 12, 10, 12, 12, 12, 12));
                try!(world.add_monster(m, "#test"));
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.slice_to(27), "PRIVMSG #test :Test rolled ".as_bytes());
    }

    #[test]
    fn roll_failed_invalid_stat() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll @0 test\r\n",
            |world| {
                try!(world.add_game("Test", "test", "#test"));
                let m = try!(Monster::create("Test", 14, 12, 10, 12, 12, 12, 12));
                try!(world.add_monster(m, "#test"));
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
                world.add_game("Test", "test", "#test")
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :@0 is not a valid monster.\r\n".as_bytes());
    }

    #[test]
    fn roll_failed_user_is_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.roll\r\n",
            |world| {
                world.add_game("Test", "test", "#test")
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
    fn set_temp_stats_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.temp @0 20 12 12 12 12 12 12\r\n",
            |world| {
                try!(world.add_game("Test", "test", "#test"));
                let m = try!(Monster::create("Test", 14, 12, 10, 12, 12, 12, 12));
                try!(world.add_monster(m, "#test"));
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :Test (@0) now has temporary Stats { health: 20, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }.\r\n".as_bytes());
    }

    #[test]
    fn set_temp_stats_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.temp @0 20 12 12 12 12 12 12\r\n",
            |world| {
                world.add_game("Test", "test", "#test")
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :@0 is not a valid monster.\r\n".as_bytes());
    }

    #[test]
    fn set_temp_stats_failed_non_integers() {
        let data = test_helper(":test!test@test PRIVMSG #test :.temp @0 20 -12 a 12 12 12 12\r\n",
            |world| {
                try!(world.add_game("Test", "test", "#test"));
                let m = try!(Monster::create("Test", 14, 12, 10, 12, 12, 12, 12));
                try!(world.add_monster(m, "#test"));
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
                try!(world.add_game("Test", "test", "#test"));
                let mut m = try!(Monster::create("Test", 14, 12, 10, 12, 12, 12, 12));
                m.set_temp_stats(try!(Stats::new(20, 12, 12, 12, 12, 12, 12)));
                try!(world.add_monster(m, "#test"));
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :Test (@0) has reverted to Stats { health: 14, strength: 12, dexterity: 10, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }.\r\n".as_bytes());
    }

    #[test]
    fn clear_temp_stats_failed_monster_does_not_exist() {
        let data = test_helper(":test!test@test PRIVMSG #test :.cleartemp @0\r\n",
            |world| {
                world.add_game("Test", "test", "#test")
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :@0 is not a valid monster.\r\n".as_bytes());
    }

    #[test]
    fn clear_temp_stats_failed_user_is_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.cleartemp test\r\n",
            |world| {
                world.add_game("Test", "test", "#test")
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test is not logged in.\r\n".as_bytes());
    }
}
