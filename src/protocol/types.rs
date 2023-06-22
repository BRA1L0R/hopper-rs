use std::fmt::Display;

pub struct PlayerUuid(uuid::Uuid);

impl PlayerUuid {
    pub fn offline_player(player_name: &str) -> PlayerUuid {
        let mut hash = md5::compute(format!("OfflinePlayer:{player_name}"));

        // set UUID version to 3
        hash[6] = hash[6] & 0x0f | 0x30;
        // IETF variant
        hash[8] = hash[8] & 0x3f | 0x80;

        let uuid = uuid::Uuid::from_bytes(hash.0);

        PlayerUuid(uuid)
    }
}

impl Display for PlayerUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
