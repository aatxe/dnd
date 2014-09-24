extern crate irc;

fn main() {
    let process = |bot: &irc::Bot, source: &str, command: &str, args: &[&str]| {
        match (command, args) {
            ("PRIVMSG", [chan, msg]) => {
                if msg.contains("pickles") && msg.contains("hi") {
                    try!(bot.send_privmsg(chan, "hi"))
                } else if msg.starts_with(". ") {
                    try!(bot.send_privmsg(chan, msg.slice_from(2)));
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
