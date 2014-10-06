#![feature(if_let)]
extern crate crypto;
extern crate irc;
extern crate serialize;

use std::io::IoResult;
use data::{Basic, Entity, RollType};
use data::stats::Stats;
use data::utils::{join_from, str_to_u8};
use data::world::World;
#[cfg(not(test))] use func::permissions_test;
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
            if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
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
            if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
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
    if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
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
fn do_set_temp_stats(bot: &Bot, user: &str, chan: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
    if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
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
    if !try!(permissions_test(bot, user, chan, world)) { return Ok(()); }
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
                        try!(func::monster::look_up(bot, user, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("addmonster") {
                        try!(func::monster::add(bot, user, &mut world, msg.clone().split_str(" ").collect()));
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
