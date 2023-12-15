use crate::{CardValue, ValueSetType};
use bitvec::prelude::*;

pub struct ValueSetWindowIterator {
    value_set: ValueSetType,

    //Can be ace for the wheel, starts with this
    stop: CardValue,

    window_start: CardValue,
    window_stop: CardValue,

    //How many values are in the window
    //So 9 T T J in a window from 9 to J is 3
    value_count: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValueSetWindowIteratorItem {
    //note this can be Ace for the wheel
    pub window_start: CardValue,

    pub window_stop: CardValue,

    pub value_count: u8,
}

impl ValueSetWindowIteratorItem {
    pub fn is_set_on_either_side(&self, value_set: ValueSetType) -> bool {
        //here the prev would be king which is never valid
        assert!(self.window_start != CardValue::Ace);

        value_set[self.window_start.prev_card() as usize]
            && value_set[self.window_stop.next_card() as usize]
    }
}

pub fn value_set_iterator(
    value_set: ValueSetType,
    window_length: u8,
    start: CardValue,
    stop: CardValue,
) -> ValueSetWindowIterator {
    assert!(window_length > 1);

    if start != CardValue::Ace {
        assert!(stop as u8 - start as u8 + 1 >= window_length);
    }

    let window_start = start;
    //A, win len 2 ends at 2
    let window_stop: CardValue = (start.next_card() as u8 + (window_length - 2) as u8).into();

    //Add first one seperately because it could be the Ace
    let mut rolling_one_count =
        value_set[window_start.next_card() as usize..=window_stop as usize].count_ones();
    rolling_one_count += if value_set[window_start as usize] {
        1
    } else {
        0
    };

    ValueSetWindowIterator {
        value_set,
        stop,
        window_start,
        window_stop,
        value_count: rolling_one_count as u8,
    }
}

impl Iterator for ValueSetWindowIterator {
    type Item = ValueSetWindowIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.window_stop == self.stop.next_card() {
            return None;
        }

        let ret = ValueSetWindowIteratorItem {
            window_start: self.window_start,
            window_stop: self.window_stop,
            value_count: self.value_count,
        };

        self.value_count -= if self.value_set[self.window_start as usize] {
            1
        } else {
            0
        };

        self.window_start = self.window_start.next_card();
        self.window_stop = self.window_stop.next_card();

        self.value_count += if self.value_set[self.window_stop as usize] {
            1
        } else {
            0
        };

        Some(ret)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_iterator() {
        let mut value_set = ValueSetType::default();
        value_set.set(CardValue::Ace as usize, true);
        value_set.set(CardValue::Two as usize, true);
        value_set.set(CardValue::Five as usize, true);

        let mut it = value_set_iterator(value_set, 2, CardValue::Ace, CardValue::Five);

        assert_eq!(it.window_stop, CardValue::Two);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Ace);
        assert_eq!(item.window_stop, CardValue::Two);
        assert_eq!(item.value_count, 2);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Two);
        assert_eq!(item.window_stop, CardValue::Three);
        assert_eq!(item.value_count, 1);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Three);
        assert_eq!(item.window_stop, CardValue::Four);
        assert_eq!(item.value_count, 0);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Four);
        assert_eq!(item.window_stop, CardValue::Five);
        assert_eq!(item.value_count, 1);

        assert_eq!(None, it.next());
    }

    #[test]
    fn test_2_to_k_iterator() {
        let mut value_set = ValueSetType::default();
        value_set.set(CardValue::Ace as usize, true);
        value_set.set(CardValue::Two as usize, true);
        value_set.set(CardValue::Four as usize, true);
        value_set.set(CardValue::Seven as usize, true);
        value_set.set(CardValue::Nine as usize, true);
        value_set.set(CardValue::Ten as usize, true);
        value_set.set(CardValue::Queen as usize, true);

        let mut it = value_set_iterator(value_set, 4, CardValue::Two, CardValue::King);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Two);
        assert_eq!(item.window_stop, CardValue::Five);
        assert_eq!(item.value_count, 2);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Three);
        assert_eq!(item.window_stop, CardValue::Six);
        assert_eq!(item.value_count, 1);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Four);
        assert_eq!(item.window_stop, CardValue::Seven);
        assert_eq!(item.value_count, 2);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Five);
        assert_eq!(item.window_stop, CardValue::Eight);
        assert_eq!(item.value_count, 1);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Six);
        assert_eq!(item.window_stop, CardValue::Nine);
        assert_eq!(item.value_count, 2);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Seven);
        assert_eq!(item.window_stop, CardValue::Ten);
        assert_eq!(item.value_count, 3);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Eight);
        assert_eq!(item.window_stop, CardValue::Jack);
        assert_eq!(item.value_count, 2);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Nine);
        assert_eq!(item.window_stop, CardValue::Queen);
        assert_eq!(item.value_count, 3);

        let item = it.next().unwrap();
        assert_eq!(item.window_start, CardValue::Ten);
        assert_eq!(item.window_stop, CardValue::King);
        assert_eq!(item.value_count, 2);

        assert_eq!(None, it.next());
    }
}
