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
    use std::io::{BufReader, MemWriter};
    use data::Entity;
    use data::monster::Monster;
    use data::stats::Stats;
    use data::world::World;
    use func::process_world;
    use irc::Bot;
    use irc::bot::IrcBot;
    use irc::conn::Connection;

    #[test]
    fn set_temp_stats_success() {
        let r = BufReader::new(":test!test@test PRIVMSG #test :.temp @0 20 12 12 12 12 12 12\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        let m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12).unwrap();
        world.add_monster(m, "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG #test :Test (@0) now has temporary Stats { health: 20, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }.\r\n".as_bytes());
    }

    #[test]
    fn set_temp_stats_failed_does_not_exist() {
        let r = BufReader::new(":test!test@test PRIVMSG #test :.temp @0 20 12 12 12 12 12 12\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG #test :@0 is not a valid monster.\r\n".as_bytes());
    }

    #[test]
    fn set_temp_stats_failed_non_integers() {
        let r = BufReader::new(":test!test@test PRIVMSG #test :.temp @0 20 -12 a 12 12 12 12\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        let m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12).unwrap();
        world.add_monster(m, "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        let mut exp = String::from_str("PRIVMSG #test :Stats must be non-zero positive integers. Format is: \r\n");
        exp.push_str("PRIVMSG #test :.temp target health str dex con wis int cha\r\n");
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), exp.as_bytes());
    }

    #[test]
    fn clear_temp_stats_success() {
        let r = BufReader::new(":test!test@test PRIVMSG #test :.cleartemp @0\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        let mut m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12).unwrap();
        m.set_temp_stats(Stats::new(20, 12, 12, 12, 12, 12, 12).unwrap());
        world.add_monster(m, "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG #test :Test (@0) has reverted to Stats { health: 14, strength: 12, dexterity: 10, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }.\r\n".as_bytes());
    }

    #[test]
    fn clear_temp_stats_failed_does_not_exist() {
        let r = BufReader::new(":test!test@test PRIVMSG #test :.cleartemp @0\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG #test :@0 is not a valid monster.\r\n".as_bytes());
    }
}
