extern crate irc;

use std::io::IoResult;
use data::game::Game;
use data::world::World;
use irc::Bot;

pub mod monster;
pub mod player;

#[cfg(not(test))]
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

#[cfg(not(test))]
pub fn private_roll(bot: &irc::Bot, user: &str) -> IoResult<()> {
    try!(bot.send_privmsg(user, format!("You rolled {}.", Game::roll()).as_slice()));
    Ok(())
}
