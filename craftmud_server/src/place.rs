//! A location in the world.
//!
//! This is all probably temporary until a proper way of storing maps
//! and rooms is figured out.

use std::ops::{Index, IndexMut};

use legion::prelude::*;

pub struct Realm {
    places: Vec<Option<Place>>
}

impl Realm {
    pub fn new() -> Self {
        Self {
            places: vec![]
        }
    }

    pub fn next_id(&mut self) -> PlaceId {
        self.places.push(None);

        PlaceId(self.places.len() - 1)
    }

    pub fn set(&mut self, index: PlaceId, place: Place) {
        self.places[index.0] = Some(place);
    }
}

impl Index<PlaceId> for Realm {
    type Output = Place;

    fn index(&self, index: PlaceId) -> &Self::Output {
        self.places[index.0].as_ref().unwrap()
    }
}

impl IndexMut<PlaceId> for Realm {
    fn index_mut(&mut self, index: PlaceId) -> &mut Self::Output {
        self.places[index.0].as_mut().unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceId(usize);

pub struct Place {
    pub description: String,
    pub exits: Vec<(String, PlaceId)>
}

impl Place {
    pub fn look(&self, string: &mut String) {
        string.push_str(&self.description);
        string.push_str("\r\nExits: ");

        let exits = &(self.exits.iter()
            .map(|(exit_name, _)| -> &str { &*exit_name })
            .collect::<Vec<_>>()
            .join(", ")
        );
        string.push_str(exits);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn look_in_place() {
        let mut s = String::new();

        let p = Place {
            description: "Description".to_string(),
            exits: vec![("exit".to_string(), Realm::new().next_id())],
        };

        p.look(&mut s);

        assert_eq!(s, "Description\r\nExits: exit");
    }
}