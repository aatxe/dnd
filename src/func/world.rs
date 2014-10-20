use data::{BotResult, Propagated, as_io};
use data::game::Game;
use data::utils::join_from;
use data::world::World;
use func::{Functionality, incorrect_format_rf};
use irc::Bot;

pub struct Create<'a> {
    bot: &'a Bot + 'a,
    user: &'a str,
    world: &'a mut World,
    chan: &'a str,
    title: String,
}

impl <'a> Create<'a> {
    pub fn new(bot: &'a Bot, user: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Create<'a>> {
        if args.len() < 3 { return Err(incorrect_format_rf(user, "create", "channel campaign name")); }
        Ok(Create { bot: bot, user: user, world: world, chan: args[1], title: join_from(args, 2) })
    }
}

impl <'a> Functionality for Create<'a> {
    fn do_func(&mut self) -> BotResult<()> {
        if self.world.game_exists(self.chan) {
            return Err(Propagated(
                format!("{}", self.user), format!("A campaign already exists on {}.", self.chan)
            ));
        }
        try!(as_io(self.bot.send_join(self.chan)));
        try!(as_io(self.bot.send_topic(self.chan, self.title.as_slice())));
        try!(as_io(self.bot.send_mode(self.chan, "+i")));
        self.world.add_game(self.title.as_slice(), self.user, self.chan);
        let s = format!("Campaign created named {}.", self.title);
        try!(as_io(self.bot.send_privmsg(self.user, s.as_slice())));
        as_io(self.bot.send_invite(self.user, self.chan))
    }
}

pub struct PrivateRoll<'a> {
    bot: &'a Bot + 'a,
    user: &'a str,
}

impl <'a> PrivateRoll<'a> {
    pub fn new(bot: &'a Bot, user: &'a str) -> BotResult<PrivateRoll<'a>> {
        Ok(PrivateRoll { bot: bot, user: user })
    }
}

impl <'a> Functionality for PrivateRoll<'a> {
    fn do_func(&mut self) -> BotResult<()> {
        as_io(self.bot.send_privmsg(self.user, format!("You rolled {}.", Game::roll()).as_slice()))
    }
}

pub struct SaveAll<'a> {
    bot: &'a Bot + 'a,
    user: &'a str,
    world: &'a World,
}

impl <'a> SaveAll<'a> {
    pub fn new(bot: &'a Bot, user: &'a str, world: &'a World) -> BotResult<SaveAll<'a>> {
        if !bot.config().is_owner(user) {
            Err(Propagated(format!("{}", user), format!("You must own the bot to do that!")))
        } else {
            Ok(SaveAll { bot: bot, user: user, world: world })
        }
    }
}

impl <'a> Functionality for SaveAll<'a> {
    fn do_func(&mut self) -> BotResult<()> {
        try!(as_io(self.world.save_all()));
        as_io(self.bot.send_privmsg(self.user, "The world has been saved."))
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
        exp.push_str("MODE #test :+i\r\n");
        exp.push_str("PRIVMSG test :Campaign created named Dungeons and Tests.\r\n");
        exp.push_str("INVITE test :#test\r\n");
        assert_eq!(data.as_slice(), exp.as_bytes());
    }

    #[test]
    fn create_failed_already_exists() {
        let data = test_helper(":test!test@test PRIVMSG test :create #test Dungeons and Tests\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                Ok(())
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
