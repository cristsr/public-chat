use uuid::Uuid;

use crate::message::Room;

pub fn generate_rooms() -> Vec<Room> {
    let rooms = Vec::from([
        Room {
            id: Uuid::new_v4().to_string(),
            name: "Amistad".to_string(),
            people: 0,
            sockets: Vec::new(),
        },
        Room {
            id: Uuid::new_v4().to_string(),
            name: "Porno".to_string(),
            people: 0,
            sockets: Vec::new(),
        },
        Room {
            id: Uuid::new_v4().to_string(),
            name: "Maduritas".to_string(),
            people: 0,
            sockets: Vec::new(),
        },
        Room {
            id: Uuid::new_v4().to_string(),
            name: "Colombia".to_string(),
            people: 0,
            sockets: Vec::new(),
        },
        Room {
            id: Uuid::new_v4().to_string(),
            name: "Latinos".to_string(),
            people: 0,
            sockets: Vec::new(),
        },
    ]);

    rooms
}
