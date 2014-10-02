use std::ascii::AsciiExt;

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

pub trait Entity {
    fn identifier(&self) -> &str;
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
    use super::utils::{join_from, str_to_u8};

    #[test]
    fn to_roll_type_test() {
        assert_eq!(RollType::to_roll_type("str"), Some(Strength));
        assert_eq!(RollType::to_roll_type("WISDOM"), Some(Wisdom));
        assert_eq!(RollType::to_roll_type("Intellect"), Some(Intellect));
        assert_eq!(RollType::to_roll_type("test"), None);
    }

    #[test]
    fn str_to_u8_test() {
        assert_eq!(str_to_u8("4"), 4);
        assert_eq!(str_to_u8("-4"), 0);
        assert_eq!(str_to_u8("x"), 0);
    }

    #[test]
    fn join_from_test() {
        assert_eq!(join_from(vec!["hi","there","friend"], 0).as_slice(), "hi there friend");
            assert_eq!(join_from(vec!["hi","their","friend"], 1).as_slice(), "their friend");
    }
}
