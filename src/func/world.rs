#![cfg(not(test))]
use std::io::IoResult;
use data::game::Game;
use data::utils::join_from;
use data::world::World;
use func::incorrect_format;
use irc::Bot;

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
        try!(incorrect_format(bot, user, "create", "channel campaign name"));
    }
    Ok(())
}

pub fn private_roll(bot: &Bot, user: &str) -> IoResult<()> {
    try!(bot.send_privmsg(user, format!("You rolled {}.", Game::roll()).as_slice()));
    Ok(())
}

pub fn save_all(bot: &Bot, user: &str, world: &World) -> IoResult<()> {
    if bot.config.is_owner(user) {
        try!(world.save_all());
        try!(bot.send_privmsg(user, "The world has been saved."));
    } else {
        try!(bot.send_privmsg(user, "You must own the bot to do that!"));
    }
    Ok(())
}
