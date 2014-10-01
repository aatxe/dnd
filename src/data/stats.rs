use std::ascii::AsciiExt;
use std::io::{IoResult};

#[deriving(Decodable, Encodable, Show, PartialEq, Clone)]
pub struct Stats {
    pub health: u8,
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub wisdom: u8,
    pub intellect: u8,
    pub charisma: u8,
}

impl Stats {
    pub fn new(health: u8, strength: u8, dexterity: u8, constitution: u8, wisdom: u8,
               intellect: u8, charisma: u8) -> IoResult<Stats> {
        Ok(Stats {
            health: health,
            strength: strength,
            dexterity: dexterity,
            constitution: constitution,
            wisdom: wisdom,
            intellect: intellect,
            charisma: charisma,
        })
    }

    fn stat_func(&mut self, stat: &str, f: |&mut u8| -> Option<u8>) -> Option<u8> {
        match stat.to_ascii_lower().as_slice() {
            "health" => f(&mut self.health),
            "hp" => f(&mut self.health),
            "strength" => f(&mut self.strength),
            "str" => f(&mut self.strength),
            "dexterity" => f(&mut self.dexterity),
            "dex" => f(&mut self.dexterity),
            "constitution" => f(&mut self.constitution),
            "con" => f(&mut self.constitution),
            "wisdom" => f(&mut self.wisdom),
            "wis" => f(&mut self.wisdom),
            "intellect" => f(&mut self.intellect),
            "int" => f(&mut self.intellect),
            "charisma" => f(&mut self.charisma),
            "cha" => f(&mut self.charisma),
            _ => None,
        }
    }

    // This should be updated if there's a way to use stat_func(...) without making it mutable.
    pub fn get_stat(&self, stat: &str) -> Option<u8> {
        match stat.to_ascii_lower().as_slice() {
            "heath" => Some(self.health),
            "hp" => Some(self.health),
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
        self.stat_func(stat, |s: &mut u8| { *s = value; None });
    }

    pub fn increase_stat(&mut self, stat: &str, value: u8) {
        self.stat_func(stat, |s: &mut u8| { *s += value; None });
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
        let s = Stats::new(20, 12, 12, 8, 12, 14, 12).unwrap();
        assert_eq!(s.get_stat("str"), Some(12));
        assert_eq!(s.get_stat("constitution"), Some(8));
        assert_eq!(s.get_stat("INTELLECT"), Some(14));
    }

    #[test]
    fn update_stat_test() {
        let mut s = Stats::new(20, 12, 12, 12, 12, 12, 12).unwrap();
        s.update_stat("str", 10);
        assert_eq!(s.get_stat("str"), Some(10));
        s.update_stat("Con", 8);
        assert_eq!(s.get_stat("constitution"), Some(8));
        s.update_stat("InTeLlEcT", 14);
        assert_eq!(s.get_stat("INTELLECT"), Some(14));
    }

    #[test]
    fn increase_stat_test() {
        let mut s = Stats::new(20, 12, 12, 7, 12, 12, 12).unwrap();
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
