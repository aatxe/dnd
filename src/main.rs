extern crate irc;

use irc::data::Message;

fn main() {
    let process = |bot: &irc::Bot, source: &str, command: &str, args: &[&str]| {
        match (command, args) {
            ("PRIVMSG", [chan, msg]) => {
                if msg.contains("pickles") && msg.contains("hi") {
                    try!(irc::conn::send(&bot.conn, Message::new(None, "PRIVMSG", [chan.as_slice(), "hi"])));
                } else if msg.starts_with(". ") {
                    try!(irc::conn::send(&bot.conn, Message::new(None, "PRIVMSG", [chan.as_slice(), msg.slice_from(2)])));
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
