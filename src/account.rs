use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Account {
    #[serde(rename(serialize = "client"))]
    id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
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

    pub fn is_locked(&self) -> bool {
        self.locked
    }
}
