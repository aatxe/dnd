use std::io::IoResult;
use data::Entity;
use data::monster::Monster;
use data::utils::str_to_u8;
use data::world::World;
use func::{incorrect_format, permissions_test};
use irc::Bot;

pub fn add(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 10 {
        if !try!(permissions_test(bot, user, params[1], world)) { return Ok(()); }
        let mut valid = true;
        for s in params.slice_from(4).iter() {
            if str_to_u8(*s) == 0 {
                valid = false;
            }
        }
        if valid {
            let m = try!(Monster::create(params[2], str_to_u8(params[3]),
                str_to_u8(params[4]), str_to_u8(params[5]),
                str_to_u8(params[6]), str_to_u8(params[7]),
                str_to_u8(params[8]), str_to_u8(params[9])));
            let res = world.add_monster(m, params[1]);
            if res.is_ok() {
                try!(bot.send_privmsg(user, format!("Monster ({}) has been created as @{}.", params[2], try!(res)).as_slice()));
            } else {
                if let Some(err) = res.err() {
                    try!(bot.send_privmsg(user, format!("Failed to create monster: {}", err.desc).as_slice()));
                } else {
                    try!(bot.send_privmsg(user, "Failed to create monster for an unknown reason."));
                }
            }
        } else {
            try!(bot.send_privmsg(user, "Stats must be non-zero positive integers. Format is: "))
            try!(bot.send_privmsg(user, "addmonster chan name health str dex con wis int cha"));
        }
    } else {
        try!(incorrect_format(bot, user, "addmonster", "chan name health str dex con wis int cha"));
    }
    Ok(())
}

pub fn look_up(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if (params.len() == 3 || params.len() == 4) && params[2].starts_with("@") {
        if !try!(permissions_test(bot, user, params[1], world)) { return Ok(()); }
        let res = world.get_entity(params[2], Some(params[1]));
        if res.is_ok() {
            let m = try!(res);
            let tmp_msg = if m.has_temp_stats() {
                "Temp. "
            } else {
                ""
            };
            if params.len() == 3 {
                let s = format!("{} ({}): {}{}", m.identifier(), params[2], tmp_msg, m.stats());
                try!(bot.send_privmsg(user, s.as_slice()));
            } else {
                let s = match m.stats().get_stat(params[3]) {
                        Some(x) => format!("{} ({}): {}{} {}", m.identifier(), params[2], tmp_msg, x, params[3]),
                        None => format!("{} is not a valid stat.", params[3]),
                };
                try!(bot.send_privmsg(user, s.as_slice()));
            }
        } else {
            try!(bot.send_privmsg(user, format!("{} is not a valid monster.", params[2]).as_slice()));
        }
    } else if params.len() == 3 || params.len() == 4 {
        try!(bot.send_privmsg(user, format!("{} is not a valid monster.", params[2]).as_slice()));
    } else {
        try!(incorrect_format(bot, user, "mlookup", "channel target [stat]"));
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
    fn add_success() {
        let r = BufReader::new(":test!test@test PRIVMSG test :addmonster #test Test 20 12 12 12 12 12 12\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG test :Monster (Test) has been created as @0.\r\n".as_bytes());
    }

    #[test]
    fn add_failed_non_integers() {
        let r = BufReader::new(":test!test@test PRIVMSG test :addmonster #test Test 20 -12 a 12 12 12 12\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        let mut exp = String::from_str("PRIVMSG test :Stats must be non-zero positive integers. Format is: \r\n");
        exp.push_str("PRIVMSG test :addmonster chan name health str dex con wis int cha\r\n");
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), exp.as_bytes());
    }

    #[test]
    fn look_up_success() {
        let r = BufReader::new(":test!test@test PRIVMSG test :mlookup #test @0\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        world.add_monster(Monster::create("Test", 20, 12, 12, 12, 12, 12, 12).unwrap(), "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG test :Test (@0): Stats { health: 20, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }\r\n".as_bytes());
    }

    #[test]
    fn look_up_failed_no_monster() {
        let r = BufReader::new(":test!test@test PRIVMSG test :mlookup #test @1\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        world.add_monster(Monster::create("Test", 20, 12, 12, 12, 12, 12, 12).unwrap(), "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG test :@1 is not a valid monster.\r\n".as_bytes());
    }

    #[test]
    fn look_up_success_by_stat() {
        let r = BufReader::new(":test!test@test PRIVMSG test :mlookup #test @0 health\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        world.add_monster(Monster::create("Test", 20, 12, 12, 12, 12, 12, 12).unwrap(), "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG test :Test (@0): 20 health\r\n".as_bytes());
    }

    #[test]
    fn look_up_failed_invalid_stat() {
        let r = BufReader::new(":test!test@test PRIVMSG test :mlookup #test @0 test\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        world.add_monster(Monster::create("Test", 20, 12, 12, 12, 12, 12, 12).unwrap(), "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG test :test is not a valid stat.\r\n".as_bytes());
    }

    #[test]
    fn look_up_success_temporary() {
        let r = BufReader::new(":test!test@test PRIVMSG test :mlookup #test @0\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        let mut m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12).unwrap();
        m.set_temp_stats(Stats::new(20, 12, 12, 12, 12, 12, 12).unwrap());
        world.add_monster(m, "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG test :Test (@0): Temp. Stats { health: 20, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 }\r\n".as_bytes());
    }

    #[test]
    fn look_up_success_temporary_stat() {
        let r = BufReader::new(":test!test@test PRIVMSG test :mlookup #test @0 health\r\n".as_bytes());
        let mut world = World::new().unwrap();
        world.add_game("Test", "test", "#test").unwrap();
        let mut m = Monster::create("Test", 14, 12, 10, 12, 12, 12, 12).unwrap();
        m.set_temp_stats(Stats::new(20, 12, 12, 12, 12, 12, 12).unwrap());
        world.add_monster(m, "#test").unwrap();
        let mut bot = IrcBot::from_connection(Connection::new(MemWriter::new(), r).unwrap(), |bot, source, command, args| {
            process_world(bot, source, command, args, &mut world)
        }).unwrap();
        bot.output().unwrap();
        assert_eq!(bot.conn.writer().deref_mut().get_ref(), "PRIVMSG test :Test (@0): Temp. 20 health\r\n".as_bytes());
    }
}
