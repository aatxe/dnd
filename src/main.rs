extern crate irc;

use std::string::String;

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
                if msg.contains("pickles") && msg.contains("hi") {
                    try!(bot.send_privmsg(resp, "hi"))
                } else if msg.starts_with(". ") {
                    try!(bot.send_privmsg(resp, msg.slice_from(2)));
                } else if msg.starts_with(".list") {
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
                };
            },
            _ => ()
        }
        Ok(())
    };
    let mut pickle = irc::Bot::new(process).unwrap();
    pickle.identify().unwrap();
    pickle.output();
}
