extern crate irc;

fn main() {
    let mut pickle = irc::Bot::new().unwrap();
    pickle.identify().unwrap();
    pickle.output();
}
