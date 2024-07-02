use crate::protocol::VarInt;

crate::define_protocol! {
    Handshake, 0x00, ServerBound => {
        protocol: VarInt,
        address: String,
        port: u16,
        next_state: u8,
    },
    StatusResponse, 0x00, ClientBound => {
        status: String,
    },
    LoginStart, 0x00, ServerBound => {
        name: String,
    },
    EncryptionRequest, 0x01, ClientBound => {
        server_id: String,
        public_key: Vec<u8>,
        verify_token: Vec<u8>,
    },
    LoginSuccess, 0x02, ClientBound => {
        uuid: String,
        username: String,
    },
    JoinGame, 0x01, ClientBound => {
        entity_id: i32,
        gamemode: u8,
        dimension: i8,
        difficulty: u8,
        max_players: u8,
        level_type: String,
        reduced_debug_info: u8,
    },
}
