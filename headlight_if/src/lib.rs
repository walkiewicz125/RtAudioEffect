use serializer::ByteMessage;

#[derive(ByteMessage, Default, Debug)]
pub struct EchoMessage {
    pub message: String,
}

#[derive(ByteMessage, Default, Debug)]
pub struct IdentityMessage {
    pub info: String,
    pub effect_id: u8,
}

#[derive(ByteMessage, Default, Debug)]
pub struct IdentityRequestMessage {}

#[derive(ByteMessage, Default, Debug)]
pub struct SetColorMessage {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(ByteMessage, Default, Debug)]
pub struct SetServo {
    pub id: u8,
    pub position: f32,
}

#[derive(ByteMessage, Debug, Default)]
pub enum Message {
    #[default]
    Invalid,
    Ack,
    Echo(EchoMessage),
    EchoReply(EchoMessage),
    IdentityRequest(IdentityRequestMessage),
    Identity(IdentityMessage),
    SetColor(SetColorMessage),
    SetServo(SetServo),
}

impl TryFrom<Vec<u8>> for Message {
    type Error = String;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let mut message: Message = Message::default();
        let got_bytes = bytes.len();
        let size = message.from_bytes(bytes)?;
        if size != got_bytes as u32 {
            return Err(format!(
                "Invalid message size. Expected: {}, got: {}",
                size, got_bytes
            ));
        }
        Ok(message)
    }
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        self.to_bytes()
    }
}
