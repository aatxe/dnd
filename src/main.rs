#![feature(if_let)]
extern crate crypto;
extern crate irc;
extern crate serialize;

use data::world::World;
use irc::Bot;

mod data;
mod func;

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
                        try!(func::create(bot, user, &mut world, msg.clone().split_str(" ").collect()));
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
                        try!(func::entity::roll(bot, user, chan, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with(".lookup") {
                        try!(func::player::look_up(bot, chan, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with(".update") {
                        try!(func::player::add_update(bot, user, chan, &mut world, msg.clone().split_str(" ").collect(), true));
                    } else if msg.starts_with(".increase") {
                        try!(func::player::add_update(bot, user, chan, &mut world, msg.clone().split_str(" ").collect(), false));
                    } else if msg.starts_with(".temp") {
                        try!(func::entity::set_temp_stats(bot, user, chan, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with(".cleartemp") {
                        try!(func::entity::clear_temp_stats(bot, user, chan, &mut world, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with(".damage") {
                        try!(func::entity::damage(bot, user, chan, &mut world, msg.clone().split_str(" ").collect()));
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
