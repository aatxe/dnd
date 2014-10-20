use std::ascii::AsciiExt;
use data::{BotResult, Entity, as_io};
use data::player::Player;
use data::utils::{join_from, str_to_u8};
use data::world::World;
use func::incorrect_format;
use irc::Bot;

pub fn register(bot: &Bot, user: &str, params: Vec<&str>) -> BotResult<()> {
    if params.len() == 10 {
        let mut valid = true;
        for s in params.slice_from(3).iter() {
            if str_to_u8(*s) == 0 {
                valid = false;
            }
        }
        if valid {
            let p = try!(Player::create(params[1], params[2], str_to_u8(params[3]),
                str_to_u8(params[4]), str_to_u8(params[5]),
                str_to_u8(params[6]), str_to_u8(params[7]),
                str_to_u8(params[8]), str_to_u8(params[9])));
            try!(as_io(p.save()));
            try!(as_io(
                bot.send_privmsg(user, format!("Your account ({}) has been created.", params[1]).as_slice())
            ));
        } else {
            try!(as_io(
                bot.send_privmsg(user, "Stats must be non-zero positive integers. Format is: ")
            ));
            try!(as_io(
                bot.send_privmsg(user, "register username password health str dex con wis int cha")
            ));
        }
    } else {
        try!(incorrect_format(bot, user, "register", "username password health str dex con wis int cha"));
    }
    Ok(())
}

pub fn login(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> BotResult<()> {
    if params.len() == 4 {
        let pr = Player::load(params[1]);
        if pr.is_ok() && !world.is_user_logged_in(user) {
            let p = pr.unwrap();
            let mut success = false;
            match world.games.find_mut(&String::from_str(params[3])) {
                Some(game) => {
                    let res = game.login(p.clone(), user, params[2]);
                    if !res.is_err() {
                        try!(as_io(bot.send_privmsg(user, try!(res))));
                        try!(as_io(bot.send_invite(user, params[3])));
                        success = true;
                    } else {
                        try!(as_io(
                            bot.send_privmsg(user, format!("{}", res.unwrap_err()).as_slice())
                        ));
                    }
                },
                None => try!(as_io(
                    bot.send_privmsg(user, format!("Game not found on {}.", params[3]).as_slice())
                )),
            };
            if success {
                world.add_user(user, p);
            }
        } else if pr.is_err() {
            try!(as_io(
                bot.send_privmsg(user, format!("Account {} does not exist, or could not be loaded.", params[1]).as_slice())
            ));
        } else {
            try!(as_io(
                bot.send_privmsg(user, "You can only be logged into one account at once.")
            ));
            try!(as_io(
                bot.send_privmsg(user, "Use logout to log out.")
            ));
        }
    } else {
        try!(incorrect_format(bot, user, "login", "username password channel"));
    }
    Ok(())
}

pub fn logout(bot: &Bot, user: &str, world: &mut World) -> BotResult<()> {
    if world.is_user_logged_in(user) {
        try!(world.remove_user(user));
        try!(as_io(bot.send_privmsg(user, "You've been logged out.")));
    } else {
        try!(as_io(bot.send_privmsg(user, "You're not currently logged in.")));
    }
    Ok(())
}

pub fn add_feat(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> BotResult<()> {
    if params.len() > 1 {
        let res = world.get_user(user);
        if res.is_ok() {
            let name = join_from(params.clone(), 1);
            let player = try!(res);
            player.add_feat(name.as_slice());
            try!(as_io(bot.send_privmsg(user, format!("Added {} feat.", name).as_slice())));
        } else {
            try!(as_io(bot.send_privmsg(user, "You must be logged in to add a feat.")));
        }
    } else {
        try!(incorrect_format(bot, user, "addfeat", "name of feat"));
    }
    Ok(())
}

pub fn save(bot: &Bot, user: &str, world: &mut World) -> BotResult<()> {
    let res = world.get_user(user);
    if res.is_ok() {
        let player = try!(res);
        match player.save() {
            Ok(_) => try!(as_io(
                bot.send_privmsg(user, format!("Saved {}.", player.username).as_slice())
            )),
            Err(_) => try!(as_io(
                bot.send_privmsg(user, format!("Failed to save {}.", player.username).as_slice())
            )),
        }
    } else {
        try!(as_io(bot.send_privmsg(user, "You must be logged in to save.")));
    }
    Ok(())
}

pub fn look_up(bot: &Bot, resp: &str, world: &mut World, params: Vec<&str>) -> BotResult<()> {
    if params.len() == 2 || params.len() == 3 {
        let res = world.get_user(params[1]);
        if res.is_ok() {
            let p = try!(res);
            let tmp_msg = if p.has_temp_stats() {
                "Temp. "
            } else {
                ""
            };
            if params.len() == 2 {
                let s = format!("{} ({}): {}{} Feats {}", p.username, params[1], tmp_msg, p.stats(), p.feats);
                try!(as_io(bot.send_privmsg(resp, s.as_slice())));
            } else if params[2].eq_ignore_ascii_case("feats") || params[2].eq_ignore_ascii_case("feat") {
                let s = format!("{} ({}): {}", p.username, params[1], p.feats);
                try!(as_io(bot.send_privmsg(resp, s.as_slice())));
            } else {
                let s = match p.stats().get_stat(params[2]) {
                        Some(x) => format!("{} ({}): {}{} {}", p.username, params[1], tmp_msg, x, params[2]),
                        None => format!("{} is not a valid stat.", params[2]),
                };
                try!(as_io(bot.send_privmsg(resp, s.as_slice())));
            }
        } else {
            try!(as_io(
                bot.send_privmsg(resp, format!("{} is not logged in.", params[1]).as_slice())
            ));
        }
    } else {
        let dot = if resp.starts_with("#") {
            "."
        } else {
            ""
        };
        try!(incorrect_format(bot, resp, format!("{}lookup", dot).as_slice(), "target [stat]"));
    }
    Ok(())
}

pub fn add_update(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>, update: bool) -> BotResult<()> {
    if params.len() == 3 {
        let res = world.get_user(user);
        if res.is_ok() {
            let p = try!(res);
            if let Some(n) = from_str(params[2]) {
                if update {
                    p.stats.update_stat(params[1], n);
                    try!(as_io(
                        bot.send_privmsg(chan, format!("{} ({}) now has {} {}.", p.username, user, n, params[1]).as_slice())
                    ));
                } else {
                    p.stats.increase_stat(params[1], n);
                    let k = if let Some(i) = p.stats.get_stat(params[1]) { i } else { 0 };
                    try!(as_io(
                        bot.send_privmsg(chan, format!("{} ({}) now has {} {}.", p.username, user, k, params[1]).as_slice())
                    ));
                }
            } else {
                try!(as_io(
                    bot.send_privmsg(chan, format!("{} is not a valid positive integer.", params[2]).as_slice())
                ));
            }
        } else {
            try!(as_io(bot.send_privmsg(chan, "You're not logged in.")));
        }
    } else if update {
        try!(incorrect_format(bot, chan, ".update", "stat value"));
    } else {
        try!(incorrect_format(bot, chan, ".increase", "stat value"));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use data::as_io;
    use data::player::Player;
    use func::test::test_helper;

    #[test]
    fn register_success() {
        let data = test_helper(":test!test@test PRIVMSG test :register test5 test 20 12 12 12 12 12 12\r\n",
                    |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :Your account (test5) has been created.\r\n".as_bytes());
    }

    #[test]
    fn register_failed_invalid_stats() {
        let data = test_helper(":test!test@test PRIVMSG test :register test5 test 20 12 -12 a 12 12 12\r\n",
                    |_| { Ok(()) }).unwrap();
        let mut exp = String::from_str("PRIVMSG test :Stats must be non-zero positive integers. Format is: \r\n");
        exp.push_str("PRIVMSG test :register username password health str dex con wis int cha\r\n");
        assert_eq!(data.as_slice(), exp.as_bytes());
    }

    #[test]
    fn login_success() {
        let data = test_helper(":test!test@test PRIVMSG test :login login test #test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                Ok(())
            }
        ).unwrap();
        let mut exp = String::from_str("PRIVMSG test :Login successful.\r\n");
        exp.push_str("INVITE test :#test\r\n");
        assert_eq!(data.as_slice(), exp.as_bytes());
    }

    #[test]
    fn login_failed_password_incorrect() {
        let data = test_helper(":test!test@test PRIVMSG test :login login ztest #test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :Password incorrect.\r\n".as_bytes());
    }

    #[test]
    fn login_failed_game_not_found() {
        let data = test_helper(":test!test@test PRIVMSG test :login login test #test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :Game not found on #test.\r\n".as_bytes());
    }

    #[test]
    fn login_failed_player_not_found() {
        let data = test_helper(":test!test@test PRIVMSG test :login missing test #test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :Account missing does not exist, or could not be loaded.\r\n".as_bytes());
    }

    #[test]
    fn login_failed_already_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :login login test #test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = try!(as_io(Player::load("login")));
                try!(if let Some(game) = world.games.find_mut(&String::from_str("#test")) {
                    game.login(p.clone(), "test", "test")
                } else {
                    Ok("")
                });
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        let mut exp = String::from_str("PRIVMSG test :You can only be logged into one account at once.\r\n");
        exp.push_str("PRIVMSG test :Use logout to log out.\r\n");
        assert_eq!(data.as_slice(), exp.as_bytes());
    }

    #[test]
    fn logout_success() {
        let data = test_helper(":test!test@test PRIVMSG test :logout\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = try!(as_io(Player::load("login")));
                try!(if let Some(game) = world.games.find_mut(&String::from_str("#test")) {
                    game.login(p.clone(), "test", "test")
                } else {
                    Ok("")
                });
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :You've been logged out.\r\n".as_bytes());
    }

    #[test]
    fn logout_failed_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :logout\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :You're not currently logged in.\r\n".as_bytes());
    }

    #[test]
    fn add_feat_success() {
        let data = test_helper(":test!test@test PRIVMSG test :addfeat Test Feat\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = try!(as_io(Player::load("login")));
                try!(if let Some(game) = world.games.find_mut(&String::from_str("#test")) {
                    game.login(p.clone(), "test", "test")
                } else {
                    Ok("")
                });
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :Added Test Feat feat.\r\n".as_bytes());
    }

    #[test]
    fn add_feat_failed_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :addfeat Test Feat\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :You must be logged in to add a feat.\r\n".as_bytes());
    }

    #[test]
    fn save_success() {
        let data = test_helper(":test!test@test PRIVMSG test :save\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test6", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :Saved test6.\r\n".as_bytes());
    }

    #[test]
    fn save_failed_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :save\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :You must be logged in to save.\r\n".as_bytes());
    }

    #[test]
    fn lookup_query_success() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :test (test): Stats { health: 20, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 } Feats []\r\n".as_bytes());
    }

    #[test]
    fn lookup_query_success_feats() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test feats\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :test (test): []\r\n".as_bytes());
    }

    #[test]
    fn lookup_query_success_stat() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test health\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :test (test): 20 health\r\n".as_bytes());
    }

    #[test]
    fn lookup_query_failed_invalid_stat() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :test is not a valid stat.\r\n".as_bytes());
    }

    #[test]
    fn lookup_query_failed_user_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :test is not logged in.\r\n".as_bytes());
    }

    #[test]
    fn lookup_channel_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test (test): Stats { health: 20, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 } Feats []\r\n".as_bytes());
    }

    #[test]
    fn lookup_channel_success_feats() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test feats\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test (test): []\r\n".as_bytes());
    }

    #[test]
    fn lookup_channel_success_stat() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test health\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test (test): 20 health\r\n".as_bytes());
    }

    #[test]
    fn lookup_channel_failed_invalid_stat() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test is not a valid stat.\r\n".as_bytes());
    }

    #[test]
    fn lookup_channel_failed_user_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test is not logged in.\r\n".as_bytes());
    }

    #[test]
    fn add_stat_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.increase str 1\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test (test) now has 13 str.\r\n".as_bytes());
    }

    #[test]
    fn update_stat_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.update str 16\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :test (test) now has 16 str.\r\n".as_bytes());
    }

    #[test]
    fn add_update_failed_invalid_stat_value() {
        let data = test_helper(":test!test@test PRIVMSG #test :.update str a\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 12, 12, 12, 12, 12, 12);
                world.add_user("test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :a is not a valid positive integer.\r\n".as_bytes());
    }

    #[test]
    fn add_update_failed_user_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.update str 16\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG #test :You're not logged in.\r\n".as_bytes());
    }
}
