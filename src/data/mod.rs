use std::ascii::AsciiExt;
use std::error::Error as StdError;
use std::fmt::Error as FmtError;
use std::fmt::{Display, Formatter};
use std::io::{Error, Result};
use std::result::Result as StdResult;

pub mod game;
pub mod monster;
pub mod player;
pub mod stats;
pub mod world;

pub mod utils {
    use super::{BotResult};
    use super::BotError::InvalidInput;
    use std::borrow::ToOwned;
    use std::ops::{Add, Sub};
    use std::num::{Float, Int, ToPrimitive, from_f32};

    #[derive(Clone, Copy, RustcDecodable, RustcEncodable, Debug, PartialEq)]
    pub struct Position(pub i32, pub i32);

    impl Position {
        pub fn distance_sq(&self, rhs: &Position) -> i32 {
            let Position(x1, y1) = *self;
            let Position(x2, y2) = *rhs;
            (y2 - y1).pow(2) + (x2 - x1).pow(2)
        }

        // FIXME: distance doesn't work the way we'd want it to.
        // e.g. (0, 0).distance(5, 5) is 7, when we want it to be 5.
        pub fn distance(&self, rhs: &Position) -> BotResult<i32> {
            if let Some(n) = self.distance_sq(rhs).to_f32() {
                if let Some(x) = from_f32(n.sqrt().floor()) {
                    return Ok(x)
                }
            }
            Err(InvalidInput("Something went wrong calculating the distance.".to_owned()))
        }
    }

    impl Add for Position {
        type Output = Position;
        fn add(self, rhs: Position) -> Position {
            let Position(x1, y1) = self;
            let Position(x2, y2) = rhs;
            Position(x1 + x2, y1 + y2)
        }
    }

    impl Sub for Position {
        type Output = Position;
        fn sub(self, rhs: Position) -> Position {
            let Position(x1, y1) = self;
            let Position(x2, y2) = rhs;
            Position(x1 - x2, y1 - y2)
        }
    }

    pub fn join_from(words: Vec<&str>, pos: usize) -> String {
        let mut res = String::new();
        for word in words[pos..].iter() {
            res.push_str(*word);
            res.push(' ');
        }
        let len = res.len() - 1;
        res.truncate(len);
        res
    }

    pub fn str_to_u8(s: &str) -> u8 {
        s.parse().unwrap_or(0)
    }
}

pub type BotResult<T> = StdResult<T, BotError>;

#[derive(Debug)]
pub enum BotError {
    InvalidInput(String),
    Io(Error),
    NotFound(String),
    PasswordIncorrect,
    Propagated(String, String),
}

impl PartialEq<BotError> for BotError {
    fn eq(&self, other: &BotError) -> bool {
        match self {
            &BotError::InvalidInput(ref a) => match other {
                &BotError::InvalidInput(ref b) => a == b,
                _ => false,
            },
            &BotError::Io(ref a) => match other {
                &BotError::Io(ref b) => a.kind() == b.kind() && a.description() == b.description(),
                _ => false,
            },
            &BotError::NotFound(ref a) => match other {
                &BotError::NotFound(ref b) => a == b,
                _ => false,
            },
            &BotError::PasswordIncorrect => match other {
                &BotError::PasswordIncorrect => true,
                _ => false,
            },
            &BotError::Propagated(ref a, ref b) => match other {
                &BotError::Propagated(ref c, ref d) => a == c && b == d,
                _ => false,
            },
        }
    }
}

impl Display for BotError {
    fn fmt(&self, fmt: &mut Formatter) -> StdResult<(), FmtError> {
        match self {
            &BotError::InvalidInput(ref s) => write!(fmt, "{}", s),
            &BotError::Io(ref io_err) => write!(fmt, "{:?}", io_err),
            &BotError::NotFound(ref s) => write!(fmt, "{}", s),
            &BotError::PasswordIncorrect => write!(fmt, "Password incorrect."),
            &BotError::Propagated(ref s, ref v) => write!(fmt, "{}\r\n{}", s, v),
        }
    }
}

pub fn as_io<T>(res: Result<T>) -> BotResult<T> {
    if res.is_ok() {
        Ok(res.unwrap())
    } else {
        Err(BotError::Io(res.err().unwrap()))
    }
}

pub trait Entity {
    fn identifier(&self) -> &str;
    fn position(&self) -> &utils::Position;
    fn damage(&mut self, amount: u8) -> bool;
    fn roll(&self, roll_type: RollType) -> u8;
    fn do_move(&mut self, pos: utils::Position) -> BotResult<()>;
    fn stats(&self) -> stats::Stats;
    fn has_temp_stats(&self) -> bool;
    fn set_temp_stats(&mut self, stats: stats::Stats);
    fn clear_temp_stats(&mut self);
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
        match &roll_type.to_ascii_lowercase()[..] {
            "strength" => Some(RollType::Strength),
            "str" => Some(RollType::Strength),
            "dexterity" => Some(RollType::Dexterity),
            "dex" => Some(RollType::Dexterity),
            "constitution" => Some(RollType::Constitution),
            "con" => Some(RollType::Constitution),
            "wisdom" => Some(RollType::Wisdom),
            "wis" => Some(RollType::Wisdom),
            "intellect" => Some(RollType::Intellect),
            "int" => Some(RollType::Intellect),
            "charisma" => Some(RollType::Charisma),
            "cha" => Some(RollType::Charisma),
            _ => None,
        }
    }
}


#[cfg(test)]
mod test {
    use super::RollType;
    use super::RollType::{Strength, Wisdom, Intellect};
    use super::utils;
    use super::utils::Position;
    use std::io::{Error, ErrorKind, Result};

    #[test]
    fn new_position() {
        let pos = Position(3, -4);
        let Position(x, y) = pos;
        assert_eq!(x, 3);
        assert_eq!(y, -4);
    }

    #[test]
    fn add_position() {
        let Position(x1, y1) = Position(1, 2) + Position(3, 1);
        assert_eq!(x1, 4);
        assert_eq!(y1, 3);
        let Position(x2, y2) = Position(-1, 2) + Position(3, -1);
        assert_eq!(x2, 2);
        assert_eq!(y2, 1);
    }

    #[test]
    fn subtract_position() {
        let Position(x1, y1) = Position(1, 2) - Position(3, 1);
        assert_eq!(x1, -2);
        assert_eq!(y1, 1);
        let Position(x2, y2) = Position(-1, 2) - Position(3, -1);
        assert_eq!(x2, -4);
        assert_eq!(y2, 3);
    }

    #[test]
    fn distance() {
        assert_eq!(Position(0, 0).distance(&Position(5, 4)).unwrap(), 6);
        assert_eq!(Position(0, 0).distance(&Position(6, 0)).unwrap(), 6);
        assert_eq!(Position(0, 0).distance(&Position(0, 6)).unwrap(), 6);
    }

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
        assert_eq!(&utils::join_from(vec!["hi","there","friend"], 0)[..], "hi there friend");
        assert_eq!(&utils::join_from(vec!["hi","their","friend"], 1)[..], "their friend");
    }

    #[test]
    fn as_io() {
        assert!(super::as_io(Ok("This is okay!")).is_ok());
        let e: Result<RollType> = Err(Error::new(
            ErrorKind::InvalidInput, "This is not okay." 
        ));
        assert!(super::as_io(e).is_err());
    }
}
