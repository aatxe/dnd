#![feature(if_let)]
extern crate crypto;
extern crate irc;
extern crate serialize;

use std::io::IoResult;
use data::{Basic, Entity, RollType};
use data::monster::Monster;
use data::stats::Stats;
use data::utils::{join_from, str_to_u8};
use data::world::World;
use irc::Bot;

mod data;
mod func;

#[cfg(not(test))]
fn do_create(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
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

#[cfg(not(test))]
fn do_roll(bot: &Bot, user: &str, chan: &str,
           world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 1 || (params.len() == 2 && params[1].starts_with("@")) {
        let res = if params.len() == 2 {
            if !try!(do_permissions_test(bot, user, chan, world)) { return Ok(()); }
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
            if !try!(do_permissions_test(bot, user, chan, world)) { return Ok(()); }
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

#[cfg(not(test))]
fn do_damage(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if !try!(do_permissions_test(bot, user, chan, world)) { return Ok(()); }
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
        try!(bot.send_privmsg(chan, "Invalid format for damage. Format is:"));
        try!(bot.send_privmsg(chan, ".damage target value"));
    }
    Ok(())
}

#[cfg(not(test))]
fn do_monster_look_up(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if (params.len() == 3 || params.len() == 4) && params[2].starts_with("@") {
        if !try!(do_permissions_test(bot, user, params[1], world)) { return Ok(()); }
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
        try!(bot.send_privmsg(user, "Invalid format for mlookup. Format is:"));
        try!(bot.send_privmsg(user, "mlookup channel target [stat]"));
    }
    Ok(())
}

#[cfg(not(test))]
fn do_set_temp_stats(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if !try!(do_permissions_test(bot, user, chan, world)) { return Ok(()); }
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
            try!(bot.send_privmsg(chan, format!("{} is not logged in or does not exist.", params[1]).as_slice()));
        }
    } else {
        try!(bot.send_privmsg(chan, "Invalid format for setting temporary stats. Format is:"));
        try!(bot.send_privmsg(chan, ".temp target health str dex con wis int cha"));
    }
    Ok(())
}

#[cfg(not(test))]
fn do_clear_temp_stats(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if !try!(do_permissions_test(bot, user, chan, world)) { return Ok(()); }
    if params.len() == 2 {
        let res = world.get_entity(params[1], Some(chan));
        if res.is_ok() {
            let p = try!(res);
            p.clear_temp_stats();
            try!(bot.send_privmsg(chan, format!("{} ({}) has reverted to {}.", p.identifier(), params[1], p.stats()).as_slice()));
        } else {
            try!(bot.send_privmsg(chan, format!("{} is not logged in or does not exist.", user).as_slice()));
        }
    } else {
        try!(bot.send_privmsg(chan, "Invalid format for setting temporary stats. Format is:"));
        try!(bot.send_privmsg(chan, ".cleartemp target"));
    }
    Ok(())
}

#[cfg(not(test))]
fn do_add_monster(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 10 {
        if !try!(do_permissions_test(bot, user, params[1], world)) { return Ok(()); }
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
        try!(bot.send_privmsg(user, "Incorrect format for monster creation. Format is:"));
        try!(bot.send_privmsg(user, "addmonster chan name health str dex con wis int cha"));
    }
    Ok(())
}

#[cfg(not(test))]
fn do_permissions_test(bot: &Bot, user: &str, chan: &str, world: &mut World) -> IoResult<bool> {
    let mut ret = true;
    let res = world.get_game(chan);
    if res.is_err() {
        try!(bot.send_privmsg(user, format!("There is no game in {}.", chan).as_slice()));
        ret = false;
    } else if !try!(res).is_dm(user) {
        try!(bot.send_privmsg(user, "You must be the DM to do that!"));
        ret = false;
    };
    Ok(ret)
}

#[cfg(not(test))]
fn main() {
    let mut world = World::new().unwrap();
    let process = |bot: &Bot, source: &str, command: &str, args: &[&str]| {
        match (command, args) {
            ("PRIVMSG", [chan, msg]) => {
                let user = match source.find('!') {
                    Some(i) => source.slice_to(i),
                    None => chan,
                };
                if !chan.starts_with("#") {
                    if msg.starts_with("register") {
                        try!(func::player::register(bot, user, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("login") {
                        try!(func::player::login(bot, user, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("create") {
                        try!(do_create(bot, user, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("logout") {
                        try!(func::player::logout(bot, user, &mut world));
                    } else if msg.starts_with("addfeat") {
                        try!(func::player::add_feat(bot, user, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("roll") {
                        try!(func::private_roll(bot, user));
                    } else if msg.starts_with("save") {
                        try!(func::player::save(bot, user, &mut world));
                    } else if msg.starts_with("lookup") {
                        try!(func::player::look_up(bot, user, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("mlookup") {
                        try!(do_monster_look_up(bot, user, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("addmonster") {
                        try!(do_add_monster(bot, user, &mut world, msg.clone().split_str(" ").collect()));
                    }
                } else {
                    if msg.starts_with(".roll") {
                        try!(do_roll(bot, user, chan, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with(".lookup") {
                        try!(func::player::look_up(bot, chan, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with(".update") {
                        try!(func::player::add_update(bot, user, chan, &mut world, msg.clone().split_str(" ").collect(), true));
                    } else if msg.starts_with(".increase") {
                        try!(func::player::add_update(bot, user, chan, &mut world, msg.clone().split_str(" ").collect(), false));
                    } else if msg.starts_with(".temp") {
                        try!(do_set_temp_stats(bot, user, chan, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with(".cleartemp") {
                        try!(do_clear_temp_stats(bot, user, chan, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with(".damage") {
                        try!(do_damage(bot, user, chan, &mut world, msg.clone().split_str(" ").collect()));
                    }
                }
            },
            _ => (),
        }
        Ok(())
    };
    let mut pickle = Bot::new(process).unwrap();
    pickle.identify().unwrap();
    pickle.output();
}
