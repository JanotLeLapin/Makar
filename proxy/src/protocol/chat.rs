#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Chat {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,
}

impl From<makar_protocol::Chat> for Chat {
    fn from(value: makar_protocol::Chat) -> Self {
        let makar_protocol::Chat {
            text,
            color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
        } = value;
        Self {
            text,
            color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
        }
    }
}

impl crate::protocol::Serialize for Chat {
    fn size(&self) -> i32 {
        serde_json::to_string(self).unwrap().size()
    }

    fn serialize(&self, buf: &mut bytes::BytesMut) {
        serde_json::to_string(self).unwrap().serialize(buf);
    }
}
