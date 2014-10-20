#![feature(if_let)]
extern crate crypto;
extern crate irc;
extern crate serialize;

use data::world::World;
use irc::Bot;
use irc::bot::IrcBot;

mod data;
mod func;

#[cfg(not(test))]
fn main() {
    let mut world = World::new();
    let mut bot = IrcBot::new(|bot, source, command, args| {
        func::process_world(bot, source, command, args, &mut world)
    }).unwrap();
    bot.identify().unwrap();
    bot.output().unwrap();
}
