use std::ascii::AsciiExt;
use std::fmt::{FormatError, Formatter, Show};
use std::io::{IoError, IoResult};

pub mod game;
pub mod monster;
pub mod player;
pub mod stats;
pub mod world;

pub mod utils {
    use super::{BotResult};
    use super::BotError::InvalidInput;
    use std::num::{Float, from_f32, pow};

    #[deriving(Decodable, Encodable, Show, PartialEq, Clone)]
    pub struct Position(pub int, pub int);

    impl Position {
        pub fn distance_sq(&self, rhs: &Position) -> int {
            let Position(x1, y1) = *self;
            let Position(x2, y2) = *rhs;
            pow(y2 - y1, 2) + pow(x2 - x1, 2)
        }

        // FIXME: distance doesn't work the way we'd want it to.
        // e.g. (0, 0).distance(5, 5) is 7, when we want it to be 5.
        pub fn distance(&self, rhs: &Position) -> BotResult<int> {
            if let Some(n) = self.distance_sq(rhs).to_f32() {
                if let Some(x) = from_f32(n.sqrt().floor()) {
                    return Ok(x)
                }
            }
            Err(InvalidInput("Something went wrong calculating the distance.".into_string()))
        }
    }

    impl Add<Position, Position> for Position {
        fn add(&self, rhs: &Position) -> Position {
            let Position(x1, y1) = *self;
            let Position(x2, y2) = *rhs;
            Position(x1 + x2, y1 + y2)
        }
    }

    impl Sub<Position, Position> for Position {
        fn sub(&self, rhs: &Position) -> Position {
            let Position(x1, y1) = *self;
            let Position(x2, y2) = *rhs;
            Position(x1 - x2, y1 - y2)
        }
    }

    pub fn join_from(words: Vec<&str>, pos: uint) -> String {
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
        from_str(s).unwrap_or(0)
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
            &BotError::InvalidInput(ref s) => s.fmt(fmt),
            &BotError::Io(ref io_err) => io_err.fmt(fmt),
            &BotError::NotFound(ref s) => s.fmt(fmt),
            &BotError::PasswordIncorrect => "Password incorrect.".fmt(fmt),
            &BotError::Propagated(ref s, ref v) => {
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
        match roll_type.to_ascii_lower()[] {
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
    use std::io::{InvalidInput, IoError, IoResult};

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
        assert_eq!(Position(0, 0).distance(&Position(5, 4)), Ok(6));
        assert_eq!(Position(0, 0).distance(&Position(6, 0)), Ok(6));
        assert_eq!(Position(0, 0).distance(&Position(0, 6)), Ok(6));
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
        assert_eq!(utils::join_from(vec!["hi","there","friend"], 0)[], "hi there friend");
        assert_eq!(utils::join_from(vec!["hi","their","friend"], 1)[], "their friend");
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
