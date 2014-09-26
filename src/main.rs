extern crate irc;
extern crate serialize;

use data::Player;

mod data;

fn str_to_u8(s: &str) -> u8 {
    match from_str(s) {
        Some(n) => n,
        None => 0,
    }
}

#[cfg(not(test))]
fn main() {
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
                    if msg.starts_with("create") {
                        let params: Vec<&str> = msg.split_str(" ").collect();
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
                                try!(bot.send_privmsg(resp, "create username password str dex con wis int cha"));
                            }
                        } else {
                            try!(bot.send_privmsg(resp, "Incorrect format for creation. Format is:"));
                            try!(bot.send_privmsg(resp, "create username password str dex con wis int cha"));
                        }
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
