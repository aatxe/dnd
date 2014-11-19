use std::ascii::AsciiExt;
use data::{BotResult, Entity, as_io};
use data::BotError::Propagated;
use data::player::Player;
use data::utils::{join_from, str_to_u8};
use data::world::World;
use func::Functionality;
use func::utils::{incorrect_format, validate_from};
use irc::data::kinds::IrcStream;
use irc::server::utils::Wrapper;

pub struct Register<'a, T> where T: IrcStream {
    bot: &'a Wrapper<'a, T>,
    user: &'a str,
    username: &'a str, password: &'a str,
    health: u8, movement: u8,
    st: u8, dx: u8, cn: u8,
    ws: u8, it: u8, ch: u8,
}

impl<'a, T> Register<'a, T> where T: IrcStream {
    pub fn new(bot: &'a Wrapper<'a, T>, user: &'a str, args: Vec<&'a str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 11 {
            return Err(incorrect_format(user, "register",
                                        "username password health movement str dex con wis int cha")
            );
        }
        try!(validate_from(args.clone(), 3, user, "register",
                           "username password health movement str dex con wis int cha"));
        Ok(box Register {
            bot: bot,
            user: user,
            username: args[1], password: args[2],
            health: str_to_u8(args[3]), movement: str_to_u8(args[4]),
            st: str_to_u8(args[5]), dx: str_to_u8(args[6]), cn: str_to_u8(args[7]),
            ws: str_to_u8(args[8]), it: str_to_u8(args[9]), ch: str_to_u8(args[10]),
        } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for Register<'a, T> where T: IrcStream {
    fn do_func(&mut self) -> BotResult<()> {
        let p = try!(Player::create(self.username, self.password, self.health, self.movement,
                                    self.st, self.dx, self.cn, self.ws, self.it, self.ch));
        try!(as_io(p.save()));
        as_io(self.bot.send_privmsg(self.user, format!("Your account ({}) has been created.", self.username)[]))
    }

    fn format() -> String {
        "username password health movement str dex con wis int cha".into_string()
    }
}

pub struct Login<'a, T> where T: IrcStream {
    bot: &'a Wrapper<'a, T>,
    user: &'a str,
    world: &'a mut World,
    chan: &'a str,
    player: Player, password: &'a str,
}

impl<'a, T> Login<'a, T> where T: IrcStream {
    pub fn new(bot: &'a Wrapper<'a, T>, user: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 4 {
            return Err(incorrect_format(user, "login", "username password channel"));
        } else if world.is_user_logged_in(user) {
            return Err(Propagated(
                format!("{}", user),
                format!("You can only be logged into one account at once.\r\nUse logout to log out.")
            ));
        }
        Ok(box Login {
            bot: bot,
            user: user,
            world: world,
            chan: args[3],
            player: if let Ok(player) = Player::load(args[1]) {
                player
            } else {
                return Err(Propagated(
                    format!("{}", user),
                    format!("Account {} does not exist, or could not be loaded.", args[1])
                ));
            },
            password: args[2],
        } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for Login<'a, T> where T: IrcStream {
    fn do_func(&mut self) -> BotResult<()> {
        if let Some(game) = self.world.games.get_mut(&String::from_str(self.chan)) {
            let res = game.login(self.player.clone(), self.user, self.password);
            if res.is_ok() {
                try!(as_io(self.bot.send_privmsg(self.user, try!(res))));
                try!(as_io(self.bot.send_invite(self.user, self.chan)));
            } else {
                return Err(Propagated(format!("{}", self.user), format!("{}", res.unwrap_err())))
            }
        } else {
            return Err(Propagated(format!("{}", self.user), format!("Game not found on {}.", self.chan)))
        }
        self.world.add_user(self.user, self.chan, self.player.clone());
        Ok(())
    }

    fn format() -> String {
        "username password channel".into_string()
    }
}

pub struct Logout<'a, T> where T: IrcStream {
    bot: &'a Wrapper<'a, T>,
    user: &'a str,
    world: &'a mut World,
}

impl<'a, T> Logout<'a, T> where T: IrcStream {
    pub fn new(bot: &'a Wrapper<'a, T>, user: &'a str, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        Ok(box Logout { bot: bot, user: user, world: world } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for Logout<'a, T> where T: IrcStream {
    fn do_func(&mut self) -> BotResult<()> {
        if self.world.is_user_logged_in(self.user) {
            let chan = try!(self.world.remove_user(self.user));
            try!(as_io(self.bot.send_kick(chan, self.user, "Logged out.")));
            try!(as_io(self.bot.send_privmsg(self.user, "You've been logged out.")));
        } else {
            try!(as_io(self.bot.send_privmsg(self.user, "You're not currently logged in.")));
        }
        Ok(())
    }

    fn format() -> String {
        "".into_string()
    }
}

pub struct AddFeat<'a, T> where T: IrcStream {
    bot: &'a Wrapper<'a, T>,
    user: &'a str,
    world: &'a mut World,
    feat_name: String,
}

impl<'a, T> AddFeat<'a, T> where T: IrcStream {
    pub fn new(bot: &'a Wrapper<'a, T>, user: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if args.len() < 2 { return Err(incorrect_format(user, "addfeat", "name of feat")); }
        Ok(box AddFeat { bot: bot, user: user, world: world, feat_name: join_from(args, 1) } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for AddFeat<'a, T> where T: IrcStream {
    fn do_func(&mut self) -> BotResult<()> {
        if let Ok(player) = self.world.get_user(self.user) {
            player.add_feat(self.feat_name[]);
            try!(as_io(self.bot.send_privmsg(self.user, format!("Added {} feat.", self.feat_name)[])));
            Ok(())
        } else {
            Err(Propagated(format!("{}", self.user), format!("You must be logged in to add a feat.")))
        }
    }

    fn format() -> String {
        "name of feat".into_string()
    }
}

pub struct Save<'a, T> where T: IrcStream {
    bot: &'a Wrapper<'a, T>,
    user: &'a str,
    world: &'a mut World,
}

impl<'a, T> Save<'a, T> where T: IrcStream {
    pub fn new(bot: &'a Wrapper<'a, T>, user: &'a str, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        Ok(box Save { bot: bot, user: user, world: world } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for Save<'a, T> where T: IrcStream {
    fn do_func(&mut self) -> BotResult<()> {
        if let Ok(player) = self.world.get_user(self.user) {
            match player.save() {
                Ok(_) => try!(as_io(
                    self.bot.send_privmsg(self.user, format!("Saved {}.", player.username)[])
                )),
                Err(_) => try!(as_io(
                    self.bot.send_privmsg(self.user, format!("Failed to save {}.", player.username)[])
                )),
            }
            Ok(())
        } else {
            Err(Propagated(format!("{}", self.user), format!("You must be logged in to save.")))
        }
    }

    fn format() -> String {
        "".into_string()
    }
}

pub struct LookUpPlayer<'a, T> where T: IrcStream {
    bot: &'a Wrapper<'a, T>,
    resp: &'a str,
    world: &'a mut World,
    target_str: &'a str,
    stat_str: Option<&'a str>,
}

impl<'a, T> LookUpPlayer<'a, T> where T: IrcStream {
    pub fn new(bot: &'a Wrapper<'a, T>, resp: &'a str, args: Vec<&'a str>, world: &'a mut World) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 2 && args.len() != 3 {
            let dot = if resp.starts_with("#") {
                "."
            } else {
                ""
            };
            return Err(incorrect_format(resp, format!("{}lookup", dot)[], "target [stat]"));
        }
        Ok(box LookUpPlayer {
            bot: bot,
            resp: resp,
            world: world,
            target_str: args[1],
            stat_str: if args.len() == 3 {
                Some(args[2])
            } else {
                None
            },
        } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for LookUpPlayer<'a, T> where T: IrcStream {
    fn do_func(&mut self) -> BotResult<()> {
        let res = self.world.get_user(self.target_str);
        if res.is_err() {
            return Err(Propagated(format!("{}", self.resp), format!("{} is not logged in.", self.target_str)));
        }
        let p = try!(res);
        let temp = if p.has_temp_stats() { "Temp. " } else { "" };
        if self.stat_str.is_none() {
            let s = format!("{} ({}): {}{} Feats {}", p.username, self.target_str, temp, p.stats(), p.feats);
            as_io(self.bot.send_privmsg(self.resp, s[]))
        } else if self.stat_str.unwrap().eq_ignore_ascii_case("feats") || self.stat_str.unwrap().eq_ignore_ascii_case("feat") {
            let s = format!("{} ({}): {}", p.username, self.target_str, p.feats);
            as_io(self.bot.send_privmsg(self.resp, s[]))
        } else if self.stat_str.unwrap().eq_ignore_ascii_case("pos") || self.stat_str.unwrap().eq_ignore_ascii_case("position") {
            let s = format!("{} ({}): {}", p.username, self.target_str, p.position());
            as_io(self.bot.send_privmsg(self.resp, s[]))
        } else if let Some(x) = p.stats().get_stat(self.stat_str.unwrap()) {
            let s = format!("{} ({}): {}{} {}", p.identifier(), self.target_str, temp, x, self.stat_str.unwrap());
            as_io(self.bot.send_privmsg(self.resp, s[]))
        } else {
            Err(Propagated(format!("{}", self.resp), format!("{} is not a valid stat.", self.stat_str.unwrap())))
        }
    }

    fn format() -> String {
        "target [stat]".into_string()
    }
}

pub struct AddUpdate<'a, T> where T: IrcStream {
    bot: &'a Wrapper<'a, T>,
    user: &'a str,
    chan: &'a str,
    world: &'a mut World,
    stat_str: &'a str,
    value: u8,
    update: bool,
}

impl<'a, T> AddUpdate<'a, T> where T: IrcStream {
    pub fn new(bot: &'a Wrapper<'a, T>, user: &'a str, chan: &'a str, args: Vec<&'a str>, world: &'a mut World, update: bool) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 3 {
            return Err(incorrect_format(chan, if update { ".update" } else { ".increase" }, "stat value"));
        }
        Ok(box AddUpdate {
            bot: bot,
            user: user,
            chan: chan,
            world: world,
            stat_str: args[1],
            value: if let Some(n) = from_str(args[2]) {
                n
            } else {
                return Err(Propagated(format!("{}", chan), format!("{} is not a valid positive integer.", args[2])));
            },
            update: update,
        } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for AddUpdate<'a, T> where T: IrcStream {
    fn do_func(&mut self) -> BotResult<()> {
        if let Ok(p) = self.world.get_user(self.user) {
            if self.update {
                p.stats.update_stat(self.stat_str, self.value);
                try!(as_io(
                    self.bot.send_privmsg(self.chan, format!("{} ({}) now has {} {}.", p.username, self.user, self.value, self.stat_str)[])
                ));
            } else {
                p.stats.increase_stat(self.stat_str, self.value);
                let k = if let Some(i) = p.stats.get_stat(self.stat_str) { i } else { 0 };
                try!(as_io(
                    self.bot.send_privmsg(self.chan, format!("{} ({}) now has {} {}.", p.username, self.user, k, self.stat_str)[])
                ));
            }
            Ok(())
        } else {
            Err(Propagated(format!("{}", self.chan), format!("You're not logged in.")))
        }
    }

    fn format() -> String {
        "stat value".into_string()
    }
}

#[cfg(test)]
mod test {
    use data::as_io;
    use data::player::Player;
    use func::test::test_helper;

    #[test]
    fn register_success() {
        let data = test_helper(":test!test@test PRIVMSG test :register test5 test 20 30 12 12 12 12 12 12\r\n",
                    |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Your account (test5) has been created.\r\n"));
    }

    #[test]
    fn register_failed_invalid_stats() {
        let data = test_helper(":test!test@test PRIVMSG test :register test5 test 20 30 12 -12 a 12 12 12\r\n",
                    |_| { Ok(()) }).unwrap();
        let mut exp = String::from_str("PRIVMSG test :Stats must be non-zero positive integers. Format is:\r\n");
        exp.push_str("PRIVMSG test :register username password health movement str dex con wis int cha\r\n");
        assert_eq!(data, exp);
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
        exp.push_str("INVITE test #test\r\n");
        assert_eq!(data, exp);
    }

    #[test]
    fn login_failed_password_incorrect() {
        let data = test_helper(":test!test@test PRIVMSG test :login login ztest #test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Password incorrect.\r\n"));
    }

    #[test]
    fn login_failed_game_not_found() {
        let data = test_helper(":test!test@test PRIVMSG test :login login test #test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Game not found on #test.\r\n"));
    }

    #[test]
    fn login_failed_player_not_found() {
        let data = test_helper(":test!test@test PRIVMSG test :login missing test #test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Account missing does not exist, or could not be loaded.\r\n"));
    }

    #[test]
    fn login_failed_already_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :login login test #test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = try!(as_io(Player::load("login")));
                try!(if let Some(game) = world.games.get_mut(&String::from_str("#test")) {
                    game.login(p.clone(), "test", "test")
                } else {
                    Ok("")
                });
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        let mut exp = String::from_str("PRIVMSG test :You can only be logged into one account at once.\r\n");
        exp.push_str("PRIVMSG test :Use logout to log out.\r\n");
        assert_eq!(data, exp);
    }

    #[test]
    fn logout_success() {
        let data = test_helper(":test!test@test PRIVMSG test :logout\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = try!(as_io(Player::load("login")));
                try!(if let Some(game) = world.games.get_mut(&String::from_str("#test")) {
                    game.login(p.clone(), "test", "test")
                } else {
                    Ok("")
                });
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("KICK #test test :Logged out.\r\nPRIVMSG test :You've been logged out.\r\n"));
    }

    #[test]
    fn logout_failed_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :logout\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :You're not currently logged in.\r\n"));
    }

    #[test]
    fn add_feat_success() {
        let data = test_helper(":test!test@test PRIVMSG test :addfeat Test Feat\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = try!(as_io(Player::load("login")));
                try!(if let Some(game) = world.games.get_mut(&String::from_str("#test")) {
                    game.login(p.clone(), "test", "test")
                } else {
                    Ok("")
                });
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Added Test Feat feat.\r\n"));
    }

    #[test]
    fn add_feat_failed_invalid_format() {
        let data = test_helper(":test!test@test PRIVMSG test :addfeat\r\n", |_| { Ok(()) }).unwrap();
        let mut exp = String::from_str("PRIVMSG test :Incorrect format for addfeat. Format is:\r\n");
        exp.push_str("PRIVMSG test :addfeat name of feat\r\n");
        assert_eq!(data, exp);
    }

    #[test]
    fn add_feat_failed_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :addfeat Test Feat\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :You must be logged in to add a feat.\r\n"));
    }

    #[test]
    fn save_success() {
        let data = test_helper(":test!test@test PRIVMSG test :save\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test6", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :Saved test6.\r\n"));
    }

    #[test]
    fn save_failed_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :save\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :You must be logged in to save.\r\n"));
    }

    #[test]
    fn lookup_query_success() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        let exp = String::from_str("PRIVMSG test :test (test): Stats { health: 20, movement: 30, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 } Feats []\r\n");
        assert_eq!(data, exp);
    }

    #[test]
    fn lookup_query_success_feats() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test feats\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :test (test): []\r\n"));
    }

    #[test]
    fn lookup_query_success_position() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test pos\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :test (test): Position(0, 0)\r\n"));
    }

    #[test]
    fn lookup_query_success_stat() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test health\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :test (test): 20 health\r\n"));
    }

    #[test]
    fn lookup_query_failed_invalid_stat() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG test :test is not a valid stat.\r\n"));
    }

    #[test]
    fn lookup_query_failed_user_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG test :lookup test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG test :test is not logged in.\r\n"));
    }

    #[test]
    fn lookup_channel_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        let exp = String::from_str("PRIVMSG #test :test (test): Stats { health: 20, movement: 30, strength: 12, dexterity: 12, constitution: 12, wisdom: 12, intellect: 12, charisma: 12 } Feats []\r\n");
        assert_eq!(data, exp);
    }

    #[test]
    fn lookup_channel_success_feats() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test feats\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG #test :test (test): []\r\n"));
    }

    #[test]
    fn lookup_channel_success_position() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test pos\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG #test :test (test): Position(0, 0)\r\n"));
    }

    #[test]
    fn lookup_channel_success_stat() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test health\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG #test :test (test): 20 health\r\n"));
    }

    #[test]
    fn lookup_channel_failed_invalid_stat() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test test\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG #test :test is not a valid stat.\r\n"));
    }

    #[test]
    fn lookup_channel_failed_user_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.lookup test\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG #test :test is not logged in.\r\n"));
    }

    #[test]
    fn add_stat_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.increase str 1\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG #test :test (test) now has 13 str.\r\n"));
    }

    #[test]
    fn update_stat_success() {
        let data = test_helper(":test!test@test PRIVMSG #test :.update str 16\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG #test :test (test) now has 16 str.\r\n"));
    }

    #[test]
    fn add_update_failed_invalid_stat_value() {
        let data = test_helper(":test!test@test PRIVMSG #test :.update str a\r\n",
            |world| {
                world.add_game("Dungeons and Tests", "test", "#test");
                let p = Player::create_test("test", "test", 20, 30, 12, 12, 12, 12, 12, 12);
                world.add_user("test", "#test", p);
                Ok(())
            }
        ).unwrap();
        assert_eq!(data, format!("PRIVMSG #test :a is not a valid positive integer.\r\n"));
    }

    #[test]
    fn add_update_failed_user_not_logged_in() {
        let data = test_helper(":test!test@test PRIVMSG #test :.update str 16\r\n", |_| { Ok(()) }).unwrap();
        assert_eq!(data, format!("PRIVMSG #test :You're not logged in.\r\n"));
    }
}
