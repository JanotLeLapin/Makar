use bytes::BufMut;

use crate::protocol::{Chat, Serialize};

#[derive(Debug)]
pub enum TitleAction {
    SetTitle(Chat),
    SetSubtitle(Chat),
    SetTimes {
        fade_in: u32,
        stay: u32,
        fade_out: u32,
    },
    Hide,
    Reset,
}

impl Serialize for TitleAction {
    fn size(&self) -> i32 {
        match self {
            TitleAction::SetTitle(chat) | TitleAction::SetSubtitle(chat) => chat.size() + 1,
            TitleAction::SetTimes { .. } => 13,
            TitleAction::Hide | TitleAction::Reset => 1,
        }
    }

    fn serialize(&self, buf: &mut bytes::BytesMut) {
        match self {
            TitleAction::SetTitle(chat) => {
                buf.put_u8(0);
                chat.serialize(buf);
            }
            TitleAction::SetSubtitle(chat) => {
                buf.put_u8(1);
                chat.serialize(buf);
            }
            TitleAction::SetTimes {
                fade_in,
                stay,
                fade_out,
            } => {
                buf.put_u8(2);
                fade_in.serialize(buf);
                stay.serialize(buf);
                fade_out.serialize(buf);
            }
            TitleAction::Hide => {
                buf.put_u8(3);
            }
            TitleAction::Reset => {
                buf.put_u8(4);
            }
        }
    }
}
