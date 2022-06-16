use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Account {
    #[serde(rename(serialize = "client"))]
    id: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
}

impl Account {
    pub fn new(id: u16) -> Account {
        Account {
            id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }
}
