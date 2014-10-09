#![feature(if_let)]
extern crate crypto;
extern crate irc;
extern crate serialize;

use std::io::{BufferedReader, BufferedWriter, TcpStream};
use data::world::World;
use irc::{Bot};
use irc::bot::IrcBot;

mod data;
mod func;

#[cfg(not(test))]
fn main() {
    let mut world = World::new().unwrap();
    let process = |bot: &IrcBot<BufferedWriter<TcpStream>, BufferedReader<TcpStream>>, source: &str, command: &str, args: &[&str]| {
        match (command, args) {
            ("PRIVMSG", [chan, msg]) => {
                let user = match source.find('!') {
                    Some(i) => source.slice_to(i),
                    None => chan,
                };
                let tokens: Vec<&str> = msg.split_str(" ").collect();
                try!(if !chan.starts_with("#") {
                    match tokens[0] {
                        "register" => func::player::register(bot, user, tokens),
                        "login" => func::player::login(bot, user, &mut world, tokens),
                        "create" => func::world::create(bot, user, &mut world, tokens),
                        "logout" => func::player::logout(bot, user, &mut world),
                        "addfeat" => func::player::add_feat(bot, user, &mut world, tokens),
                        "roll" => func::world::private_roll(bot, user),
                        "saveall" => func::world::save_all(bot, user, &world),
                        "save" => func::player::save(bot, user, &mut world),
                        "lookup" => func::player::look_up(bot, user, &mut world, tokens),
                        "mlookup" => func::monster::look_up(bot, user, &mut world, tokens),
                        "addmonster" => func::monster::add(bot, user, &mut world, tokens),
                        _ => Ok(())
                    }
                } else {
                    if tokens[0].starts_with(".") {
                        match tokens[0].slice_from(1) {
                            "roll" => func::entity::roll(bot, user, chan, &mut world, tokens),
                            "lookup" => func::player::look_up(bot, user, &mut world, tokens),
                            "update" => func::player::add_update(bot, user, chan, &mut world, tokens, true),
                            "increase" => func::player::add_update(bot, user, chan, &mut world, tokens, false),
                            "temp" => func::entity::set_temp_stats(bot, user, chan, &mut world, tokens),
                            "cleartemp" => func::entity::clear_temp_stats(bot, user, chan, &mut world, tokens),
                            "damage" => func::entity::damage(bot, user, chan, &mut world, tokens),
                            _ => Ok(())
                        }
                    } else {
                        Ok(())
                    }
                });
            },
            _ => (),
        }
        Ok(())
    };
    let mut pickle = IrcBot::new(process).unwrap();
    pickle.identify().unwrap();
    pickle.output().unwrap();
}
