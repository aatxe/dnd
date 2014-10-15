use std::io::IoResult;
use data::game::Game;
use data::utils::join_from;
use data::world::World;
use func::incorrect_format;
use irc::Bot;

pub fn create(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() >= 3 {
        if !world.game_exists(params[1]) {
            try!(bot.send_join(params[1]));
            let name = join_from(params.clone(), 2);
            try!(bot.send_topic(params[1], name.as_slice()));
            try!(bot.send_mode(params[1], "+i"));
            try!(world.add_game(name.as_slice(), user, params[1]));
            try!(bot.send_privmsg(user, format!("Campaign created named {}.", name).as_slice()));
            try!(bot.send_invite(user, params[1]));
        } else {
            try!(bot.send_privmsg(user, format!("A campaign already exists on {}.", params[1]).as_slice()));
        }
    } else {
        try!(incorrect_format(bot, user, "create", "channel campaign name"));
    }
    Ok(())
}

pub fn private_roll(bot: &Bot, user: &str) -> IoResult<()> {
    try!(bot.send_privmsg(user, format!("You rolled {}.", Game::roll()).as_slice()));
    Ok(())
}

pub fn save_all(bot: &Bot, user: &str, world: &World) -> IoResult<()> {
    if bot.config().is_owner(user) {
        try!(world.save_all());
        try!(bot.send_privmsg(user, "The world has been saved."));
    } else {
        try!(bot.send_privmsg(user, "You must own the bot to do that!"));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use func::test::test_helper;

    #[test]
    fn create_success() {
        let data = test_helper(":test!test@test PRIVMSG test :create #test Dungeons and Tests\r\n",
                               |_| { Ok(()) }).unwrap();
        let mut exp = String::from_str("JOIN :#test\r\n");
        exp.push_str("TOPIC #test :Dungeons and Tests\r\n");
        exp.push_str("MODE #test :+i\r\n");
        exp.push_str("PRIVMSG test :Campaign created named Dungeons and Tests.\r\n");
        exp.push_str("INVITE test :#test\r\n");
        assert_eq!(data.as_slice(), exp.as_bytes());
    }

    #[test]
    fn create_failed_already_exists() {
        let data = test_helper(":test!test@test PRIVMSG test :create #test Dungeons and Tests\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test")
            }
        ).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :A campaign already exists on #test.\r\n".as_bytes());
    }

    #[test]
    fn private_roll() {
        let data = test_helper(":test!test@test PRIVMSG test :roll\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.slice_to(25), "PRIVMSG test :You rolled ".as_bytes());
    }

    #[test]
    fn save_all_from_owner() {
        let data = test_helper(":test!test@test PRIVMSG test :saveall\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test :The world has been saved.\r\n".as_bytes());
    }

    #[test]
    fn save_all_from_non_owner() {
        let data = test_helper(":test2!test@test PRIVMSG test :saveall\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data.as_slice(), "PRIVMSG test2 :You must own the bot to do that!\r\n".as_bytes());
    }
}
