use std::ascii::AsciiExt;
use std::fmt::{FormatError, Formatter, Show};
use std::io::{IoError, IoResult};

pub mod game;
pub mod monster;
pub mod player;
pub mod stats;
pub mod world;

pub mod utils {
    pub fn join_from(words: Vec<&str>, pos: uint) -> String {
        let mut res = String::new();
        for word in words.slice_from(pos).iter() {
            res.push_str(*word);
            res.push(' ');
        }
        let len = res.len() - 1;
        res.truncate(len);
        res
    }

    pub fn str_to_u8(s: &str) -> u8 {
        match from_str(s) {
            Some(n) => n,
            None => 0,
        }
    }
}

pub type BotResult<T> = Result<T, BotError>;

#[deriving(PartialEq)]
pub enum BotError {
    InvalidInput(String),
    Io(IoError),
    NotFound(String),
    PasswordIncorrect,
    Propagated(String, String),
}

impl Show for BotError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &InvalidInput(ref s) => s.fmt(fmt),
            &Io(ref io_err) => io_err.fmt(fmt),
            &NotFound(ref s) => s.fmt(fmt),
            &PasswordIncorrect => "Password incorrect.".fmt(fmt),
            &Propagated(ref s, ref v) => {
                try!(s.fmt(fmt));
                v.fmt(fmt)
            }
        }
    }
}

pub fn as_io<T>(res: IoResult<T>) -> BotResult<T> {
    if res.is_ok() {
        Ok(res.unwrap())
    } else {
        Err(Io(res.err().unwrap()))
    }
}

pub trait Entity {
    fn identifier(&self) -> &str;
    fn damage(&mut self, amount: u8) -> bool;
    fn roll(&self, roll_type: RollType) -> u8;
    fn stats(&self) -> stats::Stats;
    fn has_temp_stats(&self) -> bool;
    fn set_temp_stats(&mut self, stats: stats::Stats);
    fn clear_temp_stats(&mut self);
}

#[deriving(Show, PartialEq)]
pub enum RollType {
    Basic,
    Strength,
    Dexterity,
    Constitution,
    Wisdom,
    Intellect,
    Charisma
}

impl RollType {
    pub fn to_roll_type(roll_type: &str) -> Option<RollType> {
        match roll_type.to_ascii_lower().as_slice() {
            "strength" => Some(Strength),
            "str" => Some(Strength),
            "dexterity" => Some(Dexterity),
            "dex" => Some(Dexterity),
            "constitution" => Some(Constitution),
            "con" => Some(Constitution),
            "wisdom" => Some(Wisdom),
            "wis" => Some(Wisdom),
            "intellect" => Some(Intellect),
            "int" => Some(Intellect),
            "charisma" => Some(Charisma),
            "cha" => Some(Charisma),
            _ => None,
        }
    }
}


#[cfg(test)]
mod test {
    use super::RollType;
    use super::{Strength, Wisdom, Intellect};
    use super::utils;
    use std::io::{InvalidInput, IoError, IoResult};

    #[test]
    fn to_roll_type() {
        assert_eq!(RollType::to_roll_type("str"), Some(Strength));
        assert_eq!(RollType::to_roll_type("WISDOM"), Some(Wisdom));
        assert_eq!(RollType::to_roll_type("Intellect"), Some(Intellect));
        assert_eq!(RollType::to_roll_type("test"), None);
    }

    #[test]
    fn str_to_u8() {
        assert_eq!(utils::str_to_u8("4"), 4);
        assert_eq!(utils::str_to_u8("-4"), 0);
        assert_eq!(utils::str_to_u8("x"), 0);
    }

    #[test]
    fn join_from() {
        assert_eq!(utils::join_from(vec!["hi","there","friend"], 0).as_slice(), "hi there friend");
        assert_eq!(utils::join_from(vec!["hi","their","friend"], 1).as_slice(), "their friend");
    }

    #[test]
    fn as_io() {
        assert!(super::as_io(Ok("This is okay!")).is_ok());
        let e: IoResult<RollType> = Err(IoError {
            kind: InvalidInput,
            desc: "This is not okay.",
            detail: None,
        });
        assert!(super::as_io(e).is_err());
    }
}
