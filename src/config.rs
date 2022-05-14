use std::collections::{BTreeSet, HashSet};
use uuid::Uuid;

use crate::message::Room;

pub fn generate_rooms() -> HashSet<Room> {
    let mut rooms = HashSet::new();

    rooms.insert(Room {
        id: Uuid::new_v4().to_string(),
        name: "Amistad".to_string(),
        people: 0,
        sockets: BTreeSet::new(),
    });

    rooms.insert(Room {
        id: Uuid::new_v4().to_string(),
        name: "Porno".to_string(),
        people: 0,
        sockets: BTreeSet::new(),
    });

    rooms.insert(Room {
        id: Uuid::new_v4().to_string(),
        name: "Maduritas".to_string(),
        people: 0,
        sockets: BTreeSet::new(),
    });

    rooms.insert(Room {
        id: Uuid::new_v4().to_string(),
        name: "Colombia".to_string(),
        people: 0,
        sockets: BTreeSet::new(),
    });

    rooms.insert(Room {
        id: Uuid::new_v4().to_string(),
        name: "Latinos".to_string(),
        people: 0,
        sockets: BTreeSet::new(),
    });

    return rooms;
}
