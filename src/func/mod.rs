extern crate irc;

use std::io::IoResult;
use data::game::Game;

pub mod player;

#[cfg(not(test))]
pub fn private_roll(bot: &irc::Bot, user: &str) -> IoResult<()> {
    try!(bot.send_privmsg(user, format!("You rolled {}.", Game::roll()).as_slice()));
    Ok(())
}
