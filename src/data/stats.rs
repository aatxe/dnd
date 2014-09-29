use std::ascii::AsciiExt;
use std::io::{IoResult};

#[deriving(Decodable, Encodable, Show, PartialEq, Clone)]
pub struct Stats {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub wisdom: u8,
    pub intellect: u8,
    pub charisma: u8,
}

impl Stats {
    pub fn new(strength: u8, dexterity: u8, constitution: u8, wisdom: u8,
               intellect: u8, charisma: u8) -> IoResult<Stats> {
        Ok(Stats {
            strength: strength,
            dexterity: dexterity,
            constitution: constitution,
            wisdom: wisdom,
            intellect: intellect,
            charisma: charisma,
        })
    }

    pub fn get_stat(&self, stat: &str) -> Option<u8> {
        match stat.to_ascii_lower().as_slice() {
            "strength" => Some(self.strength),
            "str" => Some(self.strength),
            "dexterity" => Some(self.dexterity),
            "dex" => Some(self.dexterity),
            "constitution" => Some(self.constitution),
            "con" => Some(self.constitution),
            "wisdom" => Some(self.wisdom),
            "wis" => Some(self.wisdom),
            "intellect" => Some(self.intellect),
            "int" => Some(self.intellect),
            "charisma" => Some(self.charisma),
            "cha" => Some(self.charisma),
            _ => None,
        }
    }

    pub fn update_stat(&mut self, stat: &str, value: u8) {
        match stat.to_ascii_lower().as_slice() {
            "strength" => self.strength = value,
            "str" => self.strength = value,
            "dexterity" => self.dexterity = value,
            "dex" => self.dexterity = value,
            "constitution" => self.constitution = value,
            "con" => self.constitution = value,
            "wisdom" => self.wisdom = value,
            "wis" => self.wisdom = value,
            "intellect" => self.intellect = value,
            "int" => self.intellect = value,
            "charisma" => self.charisma = value,
            "cha" => self.charisma = value,
            _ => (),
        }
    }

    pub fn increase_stat(&mut self, stat: &str, value: u8) {
        match stat.to_ascii_lower().as_slice() {
            "strength" => self.strength += value,
            "str" => self.strength += value,
            "dexterity" => self.dexterity += value,
            "dex" => self.dexterity += value,
            "constitution" => self.constitution += value,
            "con" => self.constitution += value,
            "wisdom" => self.wisdom += value,
            "wis" => self.wisdom += value,
            "intellect" => self.intellect += value,
            "int" => self.intellect += value,
            "charisma" => self.charisma += value,
            "cha" => self.charisma += value,
            _ => (),
        }
    }

    pub fn calc_bonus(stat: u8) -> i8 {
        let st = stat as i8;
        (st - 10) / 2
    }
}

#[cfg(test)]
mod test {
    use super::Stats;

    #[test]
    fn get_stat_test() {
        let s = Stats::new(12, 12, 8, 12, 14, 12).unwrap();
        assert_eq!(s.get_stat("str"), Some(12));
        assert_eq!(s.get_stat("constitution"), Some(8));
        assert_eq!(s.get_stat("INTELLECT"), Some(14));
    }

    #[test]
    fn update_stat_test() {
        let mut s = Stats::new(12, 12, 12, 12, 12, 12).unwrap();
        s.update_stat("str", 10);
        assert_eq!(s.get_stat("str"), Some(10));
        s.update_stat("Con", 8);
        assert_eq!(s.get_stat("constitution"), Some(8));
        s.update_stat("InTeLlEcT", 14);
        assert_eq!(s.get_stat("INTELLECT"), Some(14));
    }

    #[test]
    fn increase_stat_test() {
        let mut s = Stats::new(12, 12, 7, 12, 12, 12).unwrap();
        s.increase_stat("str", 2);
        assert_eq!(s.get_stat("str"), Some(14));
        s.increase_stat("Con", 1);
        assert_eq!(s.get_stat("constitution"), Some(8));
        s.increase_stat("InTeLlEcT", 6);
        assert_eq!(s.get_stat("INTELLECT"), Some(18));
    }

    #[test]
    fn calc_bonus_test() {
        assert_eq!(Stats::calc_bonus(14), 2i8);
        assert_eq!(Stats::calc_bonus(11), 0i8);
        assert_eq!(Stats::calc_bonus(8), -1i8);
    }
}
