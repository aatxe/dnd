#![cfg(not(test))]
extern crate irc;

use std::ascii::AsciiExt;
use std::io::IoResult;
use data::Entity;
use data::player::Player;
use data::utils::{join_from, str_to_u8};
use data::world::World;
use func::incorrect_format;
use irc::Bot;

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
        try!(incorrect_format(bot, user, "register", "username password health str dex con wis int cha"));
    }
    Ok(())
}

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
        try!(incorrect_format(bot, user, "login", "username password channel"));
    }
    Ok(())
}

pub fn logout(bot: &Bot, user: &str, world: &mut World) -> IoResult<()> {
    if world.is_user_logged_in(user) {
        try!(world.remove_user(user));
        try!(bot.send_privmsg(user, "You've been logged out."));
    } else {
        try!(bot.send_privmsg(user, "You're not currently logged in."));
    }
    Ok(())
}

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
        try!(incorrect_format(bot, user, "addfeat", "name of feat"));
    }
    Ok(())
}

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

pub fn look_up(bot: &Bot, resp: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
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
                try!(bot.send_privmsg(resp, s.as_slice()));
            } else if params[2].eq_ignore_ascii_case("feats") || params[2].eq_ignore_ascii_case("feat") {
                try!(bot.send_privmsg(resp, format!("{} ({}): {}", p.username, params[1], p.feats).as_slice()));
            } else {
                let s = match p.stats().get_stat(params[2]) {
                        Some(x) => format!("{} ({}): {}{} {}", p.username, params[1], tmp_msg, x, params[2]),
                        None => format!("{} is not a valid stat.", params[2]),
                };
                try!(bot.send_privmsg(resp, s.as_slice()));
            }
        } else {
            try!(bot.send_privmsg(resp, format!("{} is not logged in.", params[1]).as_slice()));
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

pub fn add_update(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>, update: bool) -> IoResult<()> {
    if params.len() == 3 {
        let res = world.get_user(user);
        if res.is_ok() {
            let p = try!(res);
            if let Some(n) = from_str(params[2]) {
                if update {
                    p.stats.update_stat(params[1], n);
                    try!(bot.send_privmsg(chan, format!("{} ({}) now has {} {}.", p.username, user, n, params[1]).as_slice()));
                } else {
                    p.stats.increase_stat(params[1], n);
                    let k = match p.stats.get_stat(params[1]) {
                        Some(i) => i,
                        None => 0,
                    };
                    try!(bot.send_privmsg(chan, format!("{} ({}) now has {} {}.", p.username, user, k, params[1]).as_slice()));
                }
            } else {
                try!(bot.send_privmsg(chan, format!("{} is not a valid positive integer.", params[2]).as_slice()));
            }
        } else {
            try!(bot.send_privmsg(chan, "You're not logged in."));
        }
    } else if update {
        try!(incorrect_format(bot, chan, ".update", "stat value"));
    } else {
        try!(incorrect_format(bot, chan, ".increase", "stat value"));
    }
    Ok(())
}
