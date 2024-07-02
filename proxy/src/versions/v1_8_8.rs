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
}
