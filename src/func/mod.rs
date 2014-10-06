#![cfg(not(test))]
extern crate irc;

use std::io::IoResult;
use data::world::World;
use irc::Bot;

pub mod entity;
pub mod monster;
pub mod player;
pub mod world;

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
