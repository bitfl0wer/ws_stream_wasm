use crate::{import::*, WsErr};

/// Generic trait for different WebSocket message implementations.
/// No matter which WebSocket message type is being used, the user can always access the
/// underlying data with this common interface.
pub trait Message: TryFrom<MessageEvent, Error = WsErr> + Into<Vec<u8>> + From<Vec<u8>> {
    /// Returns the message as a string slice, if the message is in text format.
    fn as_text(&self) -> Option<&str>;
    /// Returns the message as a string, if the message is in text format.
    fn as_string(&self) -> Option<String>;
    /// Returns the message as a byte slice, if the message is binary.
    fn as_binary(&self) -> Option<Vec<u8>>;
    /// Returns true if the message is a string.
    fn is_text(&self) -> bool;
    /// Returns true if the message is binary.
    fn is_binary(&self) -> bool;
    /// Returns true if the message is empty.
    fn is_empty(&self) -> bool;
    /// Returns the length of the message. For binary messages, this is the length of the vector.
    fn len(&self) -> usize;
}

/// Represents a WebSocket Message, after converting from JavaScript type.
//
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
//
pub enum WsMessage {
    /// The data of the message is a string.
    ///
    Text(String),

    /// The message contains binary data.
    ///
    Binary(Vec<u8>),
}

impl Message for WsMessage {
    fn as_text(&self) -> Option<&str> {
        match *self {
            WsMessage::Text(ref string) => Some(string),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<String> {
        match *self {
            WsMessage::Text(ref string) => Some(string.clone()),
            _ => None,
        }
    }

    fn as_binary(&self) -> Option<Vec<u8>> {
        match *self {
            WsMessage::Binary(ref vec) => Some(vec.clone()),
            _ => None,
        }
    }

    fn is_text(&self) -> bool {
        matches!(self, WsMessage::Text(_))
    }

    fn is_binary(&self) -> bool {
        matches!(self, WsMessage::Binary(_))
    }

    fn is_empty(&self) -> bool {
        match self {
            WsMessage::Text(string) => string.is_empty(),
            WsMessage::Binary(vec) => vec.is_empty(),
        }
    }

    fn len(&self) -> usize {
        match self {
            WsMessage::Text(string) => string.len(),
            WsMessage::Binary(vec) => vec.len(),
        }
    }
}

/// This will convert the JavaScript event into a WsMessage. Note that this
/// will only work if the connection is set to use the binary type ArrayBuffer.
/// On binary type Blob, this will panic.
//
impl TryFrom<MessageEvent> for WsMessage {
    type Error = WsErr;

    fn try_from(evt: MessageEvent) -> Result<Self, Self::Error> {
        match evt.data() {
            d if d.is_instance_of::<ArrayBuffer>() => {
                let buffy = Uint8Array::new(d.unchecked_ref());
                let mut v = vec![0; buffy.length() as usize];

                buffy.copy_to(&mut v); // FIXME: get rid of this copy

                Ok(WsMessage::Binary(v))
            }

            // We don't allow invalid encodings. In principle if needed,
            // we could add a variant to WsMessage with a CString or an OsString
            // to allow the user to access this data. However until there is a usecase,
            // I'm not inclined, amongst other things because the conversion from Js isn't very
            // clear and it would require a bunch of testing for something that's a rather bad
            // idea to begin with. If you need data that is not a valid string, use a binary
            // message.
            //
            d if d.is_string() => match d.as_string() {
                Some(text) => Ok(WsMessage::Text(text)),
                None => Err(WsErr::InvalidEncoding),
            },

            // We have set the binary mode to array buffer (WsMeta::connect), so normally this shouldn't happen.
            // That is as long as this is used within the context of the WsMeta constructor.
            //
            d if d.is_instance_of::<Blob>() => Err(WsErr::CantDecodeBlob),

            // should never happen.
            //
            _ => Err(WsErr::UnknownDataType),
        }
    }
}

impl From<WsMessage> for Vec<u8> {
    fn from(msg: WsMessage) -> Self {
        match msg {
            WsMessage::Text(string) => string.into(),
            WsMessage::Binary(vec) => vec,
        }
    }
}

impl From<Vec<u8>> for WsMessage {
    fn from(vec: Vec<u8>) -> Self {
        WsMessage::Binary(vec)
    }
}

impl From<String> for WsMessage {
    fn from(s: String) -> Self {
        WsMessage::Text(s)
    }
}

impl AsRef<[u8]> for WsMessage {
    fn as_ref(&self) -> &[u8] {
        match self {
            WsMessage::Text(string) => string.as_ref(),
            WsMessage::Binary(vec) => vec.as_ref(),
        }
    }
}
