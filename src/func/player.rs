extern crate irc;

use std::io::IoResult;
use data::player::Player;
use data::utils::{join_from, str_to_u8};
use data::world::World;
use irc::Bot;

#[cfg(not(test))]
pub fn register(bot: &Bot, user: &str, params: Vec<&str>) -> IoResult<()> {
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
            try!(p.save());
            try!(bot.send_privmsg(user, format!("Your account ({}) has been created.", params[1]).as_slice()));
        } else {
            try!(bot.send_privmsg(user, "Stats must be non-zero positive integers. Format is: "))
            try!(bot.send_privmsg(user, "register username password health str dex con wis int cha"));
        }
    } else {
        try!(bot.send_privmsg(user, "Incorrect format for registration. Format is:"));
        try!(bot.send_privmsg(user, "register username password health str dex con wis int cha"));
    }
    Ok(())
}

#[cfg(not(test))]
pub fn login(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 4 {
        let pr = Player::load(params[1]);
        if pr.is_ok() && !world.is_user_logged_in(user) {
            let p = try!(pr);
            let mut success = false;
            match world.games.find_mut(&String::from_str(params[3])) {
                Some(game) => {
                    let res = try!(game.login(p.clone(), user, params[2]));
                    try!(bot.send_privmsg(user, res));
                    if "Login successful.".eq(&res) {
                        try!(bot.send_invite(user, params[3]));
                        success = true;
                    };
                },
                None => try!(bot.send_privmsg(user, format!("Game not found on channel {}.", params[3]).as_slice())),
            };
            if success {
                try!(world.add_user(user, p));
            }
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

#[cfg(not(test))]
pub fn logout(bot: &Bot, user: &str, world: &mut World) -> IoResult<()> {
    if world.is_user_logged_in(user) {
        try!(world.remove_user(user));
        try!(bot.send_privmsg(user, "You've been logged out."));
    } else {
        try!(bot.send_privmsg(user, "You're not currently logged in."));
    }
    Ok(())
}


#[cfg(not(test))]
pub fn add_feat(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
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


#[cfg(not(test))]
pub fn save(bot: &Bot, user: &str, world: &mut World) -> IoResult<()> {
    let res = world.get_user(user);
    if res.is_ok() {
        let player = try!(res);
        match player.save() {
            Ok(_) => try!(bot.send_privmsg(user, format!("Saved {}.", player.username).as_slice())),
            Err(_) => try!(bot.send_privmsg(user, format!("Failed to save {}.", player.username).as_slice())),
        }
    } else {
        try!(bot.send_privmsg(user, "You must be logged in to save."));
    }
    Ok(())
}
