extern crate crypto;
extern crate irc;
extern crate serialize;

use std::io::IoResult;
use data::{Basic, Player, RollType, World};

mod data;

fn str_to_u8(s: &str) -> u8 {
    match from_str(s) {
        Some(n) => n,
        None => 0,
    }
}

fn join_from(words: Vec<&str>, pos: uint) -> String {
    let mut res = String::new();
    for word in words.slice_from(pos).iter() {
        res.push_str(*word);
        res.push(' ');
    }
    let len = res.len() - 1;
    res.truncate(len);
    res
}

fn do_create(bot: &irc::Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() >= 3 {
        try!(bot.send_join(params[1]));
        let name = join_from(params.clone(), 2);
        try!(bot.send_topic(params[1], name.as_slice()));
        try!(bot.send_mode(params[1], "+i"));
        try!(world.add_game(name.as_slice(), user, params[1]));
        try!(bot.send_privmsg(user, format!("Campaign created named {}.", name).as_slice()));
        try!(bot.send_invite(user, params[1]));
    } else {
        try!(bot.send_privmsg(user, "Incorrect format for game creation. Format is:"));
        try!(bot.send_privmsg(user, "create channel campaign name"));
    }
    Ok(())
}

fn do_register(bot: &irc::Bot, user: &str, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 9 {
        let mut valid = true;
        for s in params.slice_from(3).iter() {
            if str_to_u8(*s) == 0 {
                valid = false;
            }
        }
        if valid {
            let p = try!(Player::create(params[1], params[2],
                str_to_u8(params[3]), str_to_u8(params[4]),
                str_to_u8(params[5]), str_to_u8(params[6]),
                str_to_u8(params[7]), str_to_u8(params[8])));
            try!(p.save());
            try!(bot.send_privmsg(user, format!("Your account ({}) has been created.", params[1]).as_slice()));
        } else {
            try!(bot.send_privmsg(user, "Stats must be non-zero positive integers. Format is: "))
            try!(bot.send_privmsg(user, "register username password str dex con wis int cha"));
        }
    } else {
        try!(bot.send_privmsg(user, "Incorrect format for registration. Format is:"));
        try!(bot.send_privmsg(user, "register username password str dex con wis int cha"));
    }
    Ok(())
}

fn do_login(bot: &irc::Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 4 {
        let pr = Player::load(params[1]);
        if pr.is_ok() && !world.is_user_logged_in(user) {
            let p = try!(pr);
            try!(world.add_user(user, p.clone()));
            match world.games.find_mut(&String::from_str(params[3])) {
                Some(game) => {
                    let res = try!(game.login(p, user, params[2]));
                    try!(bot.send_privmsg(user, res));
                    if "Login successful.".eq(&res) {
                        try!(bot.send_invite(user, params[3]));
                    };
                },
                None => try!(bot.send_privmsg(user, format!("Game not found on channel {}.", params[2]).as_slice())),
            };
        } else if pr.is_err() {
            try!(bot.send_privmsg(user, format!("Account {} does not exist, or could not be loaded.", params[1]).as_slice()));
        } else {
            try!(bot.send_privmsg(user, "You can only be logged into one account at once."));
            try!(bot.send_privmsg(user, "Use logout to log out."));
        }
    } else {
        try!(bot.send_privmsg(user, "Incorrect format for login: Format is:"));
        try!(bot.send_privmsg(user, "login username password channel"));
    }
    Ok(())
}

fn do_logout(bot: &irc::Bot, user: &str, world: &mut World) -> IoResult<()> {
    if world.is_user_logged_in(user) {
        try!(world.remove_user(user));
        try!(bot.send_privmsg(user, "You've been logged out."));
    } else {
        try!(bot.send_privmsg(user, "You're not currently logged in."));
    }
    Ok(())
}

fn do_add_feat(bot: &irc::Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() > 1 {
        let res = world.get_user(user);
        if res.is_ok() {
            let name = join_from(params.clone(), 1);
            let player = try!(res);
            player.add_feat(name.as_slice());
            try!(bot.send_privmsg(user, format!("Added {} feat.", name).as_slice()));
        } else {
            try!(bot.send_privmsg(user, "You must be logged in to add a feat."));
        }
    } else {
        try!(bot.send_privmsg(user, "Can't add feat without a name. Format is:"));
        try!(bot.send_privmsg(user, "addfeat name of feat"));
    }
    Ok(())
}

fn do_roll(bot: &irc::Bot, user: &str, chan: &str,
           world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 1 {
        let res = world.get_user(user);
        if res.is_ok() {
            let player = try!(res);
            let r = player.roll(Basic);
            try!(bot.send_privmsg(chan, format!("{} rolled {}.", user, r).as_slice()));
        } else {
            try!(bot.send_privmsg(chan, format!("{} is not logged in.", user).as_slice()));
        }
    } else if params.len() == 2 {
        let res = world.get_user(user);
        if res.is_ok() {
            let player = try!(res);
            let rt = RollType::to_roll_type(params[1]);
            match rt {
                Some(roll_type) => {
                    let r = player.roll(roll_type);
                    try!(bot.send_privmsg(chan, format!("{} rolled {}.", user, r).as_slice()));
                },
                None => {
                    try!(bot.send_privmsg(chan, format!("{} is not a valid stat.", params[1]).as_slice()));
                    try!(bot.send_privmsg(chan, "Options: str dex con wis int cha (or their full names)."));
                }
            }
        } else {
            try!(bot.send_privmsg(chan, format!("{} is not logged in.", user).as_slice()));
        }
    } else {
        try!(bot.send_privmsg(chan, "Invalid format. Use '.roll' or '.roll (stat)'."));
    }
    Ok(())
}

#[cfg(not(test))]
fn main() {
    let mut world = World::new().unwrap();
    let process = |bot: &irc::Bot, source: &str, command: &str, args: &[&str]| {
        match (command, args) {
            ("PRIVMSG", [chan, msg]) => {
                let user = match source.find('!') {
                    Some(i) => source.slice_to(i),
                    None => chan,
                };
                if !chan.starts_with("#") {
                    if msg.starts_with("register") {
                        try!(do_register(bot, user.clone(), msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("login") {
                        try!(do_login(bot, user.clone(), &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("create") {
                        try!(do_create(bot, user.clone(), &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("logout") {
                        try!(do_logout(bot, user.clone(), &mut world));
                    } else if msg.starts_with("addfeat") {
                        try!(do_add_feat(bot, user.clone(), &mut world, msg.clone().split_str(" ").collect()));
                    }
                } else {
                    if msg.starts_with(".roll") {
                        try!(do_roll(bot, user, chan, &mut world, msg.clone().split_str(" ").collect()));
                    }
                }
            },
            _ => (),
        }
        Ok(())
    };
    let mut pickle = irc::Bot::new(process).unwrap();
    pickle.identify().unwrap();
    pickle.output();
}
