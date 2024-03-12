use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::slice::Iter;

#[derive(Debug, Serialize, Deserialize)]
pub struct Editing;

#[derive(Debug, Serialize, Deserialize)]
pub struct Reading;

pub trait GetCurrentAndIncrementStringToIntMapperId {
    fn get_and_increment(&mut self) -> Self;
    fn to_usize(&self) -> usize;
}

#[derive(Serialize, Deserialize)]
pub struct StringToIntMapper<T: Default + GetCurrentAndIncrementStringToIntMapperId, State = Editing> {
    set_next_id: T,                     // starts at 0
    added_in_order: Vec<String>,        // to retrive string by ID
    keys_mapped_to: HashMap<String, T>, // to get ID from String
    removed_ids_in_order: Vec<T>,
    #[serde(skip)]
    state: PhantomData<State>,
}

impl<T: Default + GetCurrentAndIncrementStringToIntMapperId> StringToIntMapper<T> {
    fn consume_new<StateFrom, StateTo>(c: StringToIntMapper<T, StateFrom>) -> StringToIntMapper<T, StateTo> {
        StringToIntMapper {
            set_next_id: c.set_next_id,
            added_in_order: c.added_in_order,
            keys_mapped_to: c.keys_mapped_to,
            removed_ids_in_order: c.removed_ids_in_order,
            state: PhantomData, 
        }
    }
}

impl<T: Default + GetCurrentAndIncrementStringToIntMapperId> From<StringToIntMapper<T, Reading>> for StringToIntMapper<T, Editing> {
    fn from(item: StringToIntMapper<T, Reading>) -> Self {
        StringToIntMapper::consume_new(item)
    }
}

impl<T: Default + GetCurrentAndIncrementStringToIntMapperId> From<StringToIntMapper<T, Editing>> for StringToIntMapper<T, Reading> {
    fn from(item: StringToIntMapper<T, Editing>) -> Self {
        StringToIntMapper::consume_new(item)
    }
}

impl<T: Default + GetCurrentAndIncrementStringToIntMapperId, State> StringToIntMapper<T, State> {
    pub fn new() -> Self {
        StringToIntMapper {
            set_next_id: Default::default(),
            added_in_order: Vec::new(),
            keys_mapped_to: HashMap::new(),
            removed_ids_in_order: Vec::new(),
            state: PhantomData,
        }
    }

    pub fn get_id(&self, key: &str) -> Option<&T> {
        self.keys_mapped_to.get(key)
    }
}


impl<T: Default + Copy + GetCurrentAndIncrementStringToIntMapperId> StringToIntMapper<T, Editing> {
    pub fn add(&mut self, key: &str) -> Option<T> {
        // Only add keys that do not already exist
        if self.get_id(key).is_none() {
            let id = self.set_next_id.get_and_increment();
            self.added_in_order.push(String::from(key));
            self.keys_mapped_to.insert(String::from(key), id);
            return Some(id);
        }
        None
    }
    // think about remove here?? do we really want to change the hash id. Don't change it for now
    pub fn remove(&mut self, key: &str) {
        match self.get_id(key) {
            Some(v) => self.removed_ids_in_order.push(v.clone()),
            None => (),
        }
    }
    pub fn to_reader(self) -> StringToIntMapper<T, Reading> {
        self.into()
    }

    pub fn new_editing() -> StringToIntMapper<T, Editing> {
        Self::new()
    }
}

impl<T: Default + GetCurrentAndIncrementStringToIntMapperId> StringToIntMapper<T, Reading> {    
    pub fn get_key(&self, id: &T) -> Option<&str> {
        let uid: usize = id.to_usize();
        if self.added_in_order.len() > uid {
            return Some(&self.added_in_order[uid]);
        }
        None
    }
    
    pub fn iter_in_order(&self) -> Iter<String> {
        self.added_in_order.iter()
    }
    
    pub fn to_editer(self) -> StringToIntMapper<T, Editing> {
        self.into()
    }

    pub fn new_reading() -> StringToIntMapper<T, Reading> {
        Self::new()
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    impl GetCurrentAndIncrementStringToIntMapperId for i32 {
        fn get_and_increment(&mut self) -> Self {
            let ret = self.clone();
            *self = ret + 1i32;
            ret
        }
        fn to_usize(&self) -> usize {
            *self as usize
        }
    }

    #[test]
    fn add_and_get_key_test() {
        let mut mapper = StringToIntMapper::<i32, Editing>::new();
        assert_eq!(Some(0), mapper.add("one"));
        assert_eq!(None, mapper.add("one"));
        assert_eq!(Some(1), mapper.add("two"));
        assert_eq!(Some(2), mapper.add("three"));
        let mapper = mapper.to_reader();
        assert_eq!(Some(&0), mapper.get_id("one"));
        assert_eq!(None, mapper.get_id("N/A"));
        assert_eq!(Some(&1), mapper.get_id("two"));
        assert_eq!(Some(&2), mapper.get_id("three"));
    }
}
