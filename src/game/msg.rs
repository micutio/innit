use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum MsgClass {
    Info,
    Action,
    Alert,
    Story(Option<String>),
}

#[derive(Serialize, Deserialize, Default)]
pub struct Log {
    pub is_changed: bool,
    pub messages: Vec<(String, MsgClass)>,
}

impl Log {
    pub fn new() -> Self {
        Log {
            is_changed: false,
            messages: Vec::new(),
        }
    }
}

/// The message log can add text from any string collection.
pub trait MessageLog {
    fn add<T: Into<String>>(&mut self, message: T, class: MsgClass);
}

impl MessageLog for Log {
    /// Push a message into the log under two conditions:
    /// - either the log is empty
    /// - or the last message is not identical to the new message
    fn add<T: Into<String>>(&mut self, msg: T, class: MsgClass) {
        if self.messages.is_empty() {
            self.messages.push((msg.into(), class));
            self.is_changed = true;
            return;
        }

        if let Some(recent_msg) = self.messages.last() {
            let msg_str = msg.into();
            if !recent_msg.0.eq(&msg_str) {
                self.messages.push((msg_str, class));
                self.is_changed = true;
            }
        }
    }
}
