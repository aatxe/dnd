extern crate irc;

use std::io::IoResult;
use data::world::World;
use irc::Bot;
use irc::bot::IrcBot;
use irc::data::{IrcReader, IrcWriter};

pub mod entity;
pub mod monster;
pub mod player;
pub mod world;

pub fn process_world<T, U>(bot: &IrcBot<T, U>, source: &str, command: &str, args: &[&str], world: &mut World) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    match (command, args) {
        ("PRIVMSG", [chan, msg]) => {
            let user = match source.find('!') {
                Some(i) => source.slice_to(i),
                None => chan,
            };
            let tokens: Vec<&str> = msg.split_str(" ").collect();
            try!(if !chan.starts_with("#") {
                match tokens[0] {
                    "register" => player::register(bot, user, tokens),
                    "login" => player::login(bot, user, world, tokens),
                    "create" => world::create(bot, user, world, tokens),
                    "logout" => player::logout(bot, user, world),
                    "addfeat" => player::add_feat(bot, user, world, tokens),
                    "roll" => world::private_roll(bot, user),
                    "saveall" => world::save_all(bot, user, world),
                    "save" => player::save(bot, user, world),
                    "lookup" => player::look_up(bot, user, world, tokens),
                    "mlookup" => monster::look_up(bot, user, world, tokens),
                    "addmonster" => monster::add(bot, user, world, tokens),
                    _ => Ok(())
                }
            } else {
                if tokens[0].starts_with(".") {
                    match tokens[0].slice_from(1) {
                        "roll" => entity::roll(bot, user, chan, world, tokens),
                        "lookup" => player::look_up(bot, user, world, tokens),
                        "update" => player::add_update(bot, user, chan, world, tokens, true),
                        "increase" => player::add_update(bot, user, chan, world, tokens, false),
                        "temp" => entity::set_temp_stats(bot, user, chan, world, tokens),
                        "cleartemp" => entity::clear_temp_stats(bot, user, chan, world, tokens),
                        "damage" => entity::damage(bot, user, chan, world, tokens),
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
}

pub fn permissions_test(bot: &Bot, user: &str, chan: &str, world: &mut World) -> IoResult<bool> {
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

pub fn incorrect_format(bot: &Bot, resp: &str, cmd: &str, format: &str) -> IoResult<()> {
    try!(bot.send_privmsg(resp, format!("Incorrect format for {}. Format is:", cmd).as_slice()));
    try!(bot.send_privmsg(resp, format!("{} {}", cmd, format).as_slice()));
    Ok(())
}
