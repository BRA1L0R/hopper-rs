use std::fmt::Display;

pub struct Uuid(uuid::Uuid);

impl Uuid {
    pub fn offline_player(player_name: &str) -> Uuid {
        let hash = md5::compute(format!("OfflinePlayer:{player_name}"));
        let uuid = uuid::Uuid::from_bytes(hash.0);

        Uuid(uuid)
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
