extern crate irc;

use std::io::IoResult;
use data::game::Game;
use data::utils::join_from;
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
pub fn create(bot: &Bot, user: &str, world: &mut World, params: Vec<&str>) -> IoResult<()> {
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
pub fn private_roll(bot: &irc::Bot, user: &str) -> IoResult<()> {
    try!(bot.send_privmsg(user, format!("You rolled {}.", Game::roll()).as_slice()));
    Ok(())
}
