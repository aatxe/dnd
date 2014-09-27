extern crate irc;
extern crate serialize;

use data::{Game, Player};
use std::collections::HashMap;
use std::io::IoResult;

mod data;

fn str_to_u8(s: &str) -> u8 {
    match from_str(s) {
        Some(n) => n,
        None => 0,
    }
}

fn join_from(words: Vec<&str>, pos: uint) -> String {
    let mut res = String::new();
    for word in words.slice_from(pos).iter() {
        res.push_str(*word);
        res.push_char(' ');
    }
    let len = res.len() - 1;
    res.truncate(len);
    res
}

fn do_create(bot: &irc::Bot, resp: &str, games: &mut HashMap<String, Game>, params: Vec<&str>) -> IoResult<()> {
    if params.len() >= 3 {
        try!(bot.send_join(params[1]));
        let name = join_from(params.clone(), 2);
        try!(bot.send_topic(params[1], name.as_slice()));
        let game = try!(Game::new(name.as_slice()));
        games.insert(String::from_str(params[1]), game);
        try!(bot.send_invite(resp, params[1]));
    } else {
        try!(bot.send_privmsg(resp, "Incorrect format for game creation. Format is:"));
        try!(bot.send_privmsg(resp, "create channel campaign name"));
    }
    Ok(())
}

fn do_register(bot: &irc::Bot, resp: &str, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 9 {
        let mut valid = true;
        for s in params.slice_from(3).iter() {
            if str_to_u8(*s) == 0 {
                valid = false;
            }
        }
        if valid {
            let p = try!(Player::create(params[1], params[2],
                str_to_u8(params[3]), str_to_u8(params[4]),
                str_to_u8(params[5]), str_to_u8(params[6]),
                str_to_u8(params[7]), str_to_u8(params[8])));
            try!(p.save());
            try!(bot.send_privmsg(resp, "Your account has been created."));
        } else {
            try!(bot.send_privmsg(resp, "Stats must be non-zero positive integers. Format is: "))
            try!(bot.send_privmsg(resp, "register username password str dex con wis int cha"));
        }
    } else {
        try!(bot.send_privmsg(resp, "Incorrect format for registration. Format is:"));
        try!(bot.send_privmsg(resp, "register username password str dex con wis int cha"));
    }
    Ok(())
}

fn do_login(bot: &irc::Bot, resp: &str, games: &mut HashMap<String, Game>, params: Vec<&str>) -> IoResult<()> {
    if params.len() == 4 {
        let pr = Player::load(params[1]);
        if pr.is_ok() {
            let p = try!(pr);
            match games.find_mut(&String::from_str(params[3])) {
                Some(game) => {
                    let res = try!(game.login(p, resp, params[2]));
                    try!(bot.send_privmsg(resp, res));
                },
                None => try!(bot.send_privmsg(resp, "Game not found on that channel.")),
            };
        } else {
            try!(bot.send_privmsg(resp, "Account does not exist, or could not be loaded."));
        }
    } else {
        try!(bot.send_privmsg(resp, "Incorrect format for login: Format is:"));
        try!(bot.send_privmsg(resp, "login username password channel"));
    }
    Ok(())
}

#[cfg(not(test))]
fn main() {
    let mut games: HashMap<String, Game> = HashMap::new();
    let process = |bot: &irc::Bot, source: &str, command: &str, args: &[&str]| {
        match (command, args) {
            ("PRIVMSG", [chan, msg]) => {
                let resp = if chan.starts_with("#") {
                    chan
                } else {
                    match source.find('!') {
                        Some(i) => source.slice_to(i),
                        None => chan,
                    }
                };
                if !chan.starts_with("#") {
                    if msg.starts_with("register") {
                        try!(do_register(bot, resp.clone(), msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("login") {
                        try!(do_login(bot, resp.clone(), &mut games, msg.clone().split_str(" ").collect()));
                    } else if msg.starts_with("create") {
                        try!(do_create(bot, resp.clone(), &mut games, msg.clone().split_str(" ").collect()));
                    }
                } else {
                    if msg.starts_with(".list") {
                        let mut s = String::new();
                        match bot.chanlists.find(&String::from_str(chan)) {
                            Some(vec) => {
                                for user in vec.iter() {
                                    s.push_str(user.as_slice());
                                    s.push_char(' ');
                                }
                                let len = s.len() - 1;
                                s.truncate(len);
                            },
                            None => {
                                s.push_str("None.");
                            }
                        }
                        try!(bot.send_privmsg(resp, s.as_slice()));
                    }
                }
            },
            _ => (),
        }
        Ok(())
    };
    let mut pickle = irc::Bot::new(process).unwrap();
    pickle.identify().unwrap();
    pickle.output();
}
