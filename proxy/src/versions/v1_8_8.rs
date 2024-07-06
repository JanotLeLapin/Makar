use crate::protocol::{Chat, TitleAction, VarInt};
use makar_protocol::{Difficulty, Gamemode};

crate::define_client_bound! {
    StatusResponse, 0x00 => {
        status: String,
    },
    EncryptionRequest, 0x01 => {
        server_id: String,
        public_key: Vec<u8>,
        verify_token: Vec<u8>,
    },
    LoginSuccess, 0x02 => {
        uuid: String,
        username: String,
    },
    JoinGame, 0x01 => {
        entity_id: i32,
        gamemode: Gamemode,
        dimension: i8,
        difficulty: Difficulty,
        max_players: u8,
        level_type: String,
        reduced_debug_info: u8,
    },
    PlayerPositionAndLook, 0x08 => {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        flags: u8,
    },
    ChatMessage, 0x02 => {
        json: Chat,
        position: u8,
    },
    Title, 0x45 => {
        action: TitleAction,
    },
}

crate::define_proxy_bound! {
    Handshake, Handshake, 0x00 => {
        protocol: VarInt,
        address: String,
        port: u16,
        next_state: u8,
    },
    StatusRequest, Status, 0x00 => {},
    StatusPing, Status, 0x01 => {
        payload: u64,
    },
    LoginStart, Login, 0x00 => {
        name: String,
    },
    ChatMessage, Play, 0x01 => {
        message: String,
    },
    PlayerIsOnGround, Play, 0x03 => {
        on_ground: u8,
    },
    PlayerPosition, Play, 0x04 => {
        x: f64,
        y: f64,
        z: f64,
        on_ground: u8,
    },
    PlayerPositionAndLook, Play, 0x06 => {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: u8,
    },
    ClientSettings, Play, 0x15 => {
        locale: String,
        view_distance: u8,
        chat_mode: u8,
        chat_colors: u8,
        displayed_skin_parts: u8,
    },
    PluginMessage, Play, 0x17 => {
        channel: String,
        // data: Vec<u8>,
    },
}
