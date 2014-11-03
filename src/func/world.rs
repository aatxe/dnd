use data::{BotResult, Propagated, as_io};
use data::game::Game;
use data::utils::join_from;
use data::world::World;
use func::Functionality;
use func::utils::incorrect_format;
use irc::data::kinds::{IrcReader, IrcWriter};
use irc::server::Server;
use irc::server::utils::Wrapper;

pub struct Create<'a, T, U> where T: IrcWriter, U: IrcReader {
    bot: &'a Wrapper<'a, T, U>,
    user: &'a str,
    world: &'a mut World,
    chan: &'a str,
    title: String,
}

impl<'a, T, U> Create<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(bot: &'a Wrapper<'a, T, U>, user: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if args.len() < 3 { return Err(incorrect_format(user, "create", "channel campaign name")); }
        Ok(box Create { bot: bot, user: user, world: world, chan: args[1], title: join_from(args, 2) } as Box<Functionality>)
    }
}

impl<'a, T, U> Functionality for Create<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn do_func(&mut self) -> BotResult<()> {
        if self.world.game_exists(self.chan) {
            return Err(Propagated(
                format!("{}", self.user), format!("A campaign already exists on {}.", self.chan)
            ));
        }
        try!(as_io(self.bot.send_join(self.chan)));
        try!(as_io(self.bot.send_topic(self.chan, self.title[])));
        try!(as_io(self.bot.send_mode(self.chan, "+i")));
        self.world.add_game(self.title[], self.user, self.chan);
        let s = format!("Campaign created named {}.", self.title);
        try!(as_io(self.bot.send_privmsg(self.user, s[])));
        as_io(self.bot.send_invite(self.user, self.chan))
    }

    fn format() -> String {
        "channel campaign name".into_string()
    }
}

pub struct PrivateRoll<'a, T, U> where T: IrcWriter, U: IrcReader {
    bot: &'a Wrapper<'a, T, U>,
    user: &'a str,
}

impl<'a, T, U> PrivateRoll<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(bot: &'a Wrapper<'a, T, U>, user: &'a str) -> BotResult<Box<Functionality + 'a>> {
        Ok(box PrivateRoll { bot: bot, user: user } as Box<Functionality>)
    }
}

impl<'a, T, U> Functionality for PrivateRoll<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn do_func(&mut self) -> BotResult<()> {
        as_io(self.bot.send_privmsg(self.user, format!("You rolled {}.", Game::roll())[]))
    }

    fn format() -> String {
        "".into_string()
    }
}

pub struct SaveAll<'a, T, U> where T: IrcWriter, U: IrcReader {
    bot: &'a Wrapper<'a, T, U>,
    user: &'a str,
    world: &'a World,
}

impl<'a, T, U> SaveAll<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(bot: &'a Wrapper<'a, T, U>, user: &'a str, world: &'a World) -> BotResult<Box<Functionality + 'a>> {
        if !bot.config().is_owner(user) {
            Err(Propagated(format!("{}", user), format!("You must own the bot to do that!")))
        } else {
            Ok(box SaveAll { bot: bot, user: user, world: world } as Box<Functionality>)
        }
    }
}

impl<'a, T, U> Functionality for SaveAll<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn do_func(&mut self) -> BotResult<()> {
        try!(as_io(self.world.save_all()));
        as_io(self.bot.send_privmsg(self.user, "The world has been saved."))
    }

    fn format() -> String {
        "".into_string()
    }
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
        exp.push_str("MODE #test +i\r\n");
        exp.push_str("PRIVMSG test :Campaign created named Dungeons and Tests.\r\n");
        exp.push_str("INVITE test :#test\r\n");
        assert_eq!(data, exp);
    }

    #[test]
    fn create_failed_already_exists() {
        let data = test_helper(":test!test@test PRIVMSG test :create #test Dungeons and Tests\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :A campaign already exists on #test.\r\n"));
    }

    #[test]
    fn private_roll() {
        let data = test_helper(":test!test@test PRIVMSG test :roll\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data[..25].into_string(), format!("PRIVMSG test :You rolled "));
    }

    #[test]
    fn save_all_from_owner() {
        let data = test_helper(":test!test@test PRIVMSG test :saveall\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :The world has been saved.\r\n"));
    }

    #[test]
    fn save_all_from_non_owner() {
        let data = test_helper(":test2!test@test PRIVMSG test :saveall\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test2 :You must own the bot to do that!\r\n"));
    }
}
