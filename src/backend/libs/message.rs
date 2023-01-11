use json::JsonValue;

use crate::string;

use super::usergroup::*;

use std::{collections::HashMap, fmt::Display};
#[derive(Debug, Clone)]
pub struct Message {
    pub head: MessageHead,
    pub from: Target,
    pub target: Target,
    pub msg: String,
}
#[derive(Debug, Clone, Copy)]
pub enum MessageHead {
    Login,
    Message,
    Display,
    SighUp,
    UserList,
}

impl Message {
    pub fn new(head: MessageHead, from: Target, target: Target, msg: String) -> Self {
        Self {
            head,
            from,
            target,
            msg,
        }
    }

    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(string!("head"), self.head.to_string());
        map.insert(string!("from"), self.from.to_string());
        map.insert(string!("target"), self.target.to_string());
        map.insert(string!("msg"), self.msg.clone());
        map
    }
}

impl Default for Message {
    fn default() -> Self {
        Self {
            head: Default::default(),
            msg: Default::default(),
            from: Target::default(),
            target: Target::default(),
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "H:[{}] F:[{}] T:[{}] M:{}",
            self.head.to_string(),
            self.from.get_name(),
            self.target.get_name(),
            self.msg.to_string()
        )
    }
}

impl TryFrom<JsonValue> for Message {
    type Error = ();

    fn try_from(mut value: JsonValue) -> Result<Self, Self::Error> {
        let shead = value["head"].take_string().ok_or(())?;
        let head;
        if shead == string!("l") {
            head = MessageHead::Login;
        } else if shead == string!("d") {
            head = MessageHead::Display;
        } else if shead == string!("s") {
            head = MessageHead::Message;
        } else if shead == string!("n") {
            head = MessageHead::SighUp;
        } else if shead == string!("u") {
            head = MessageHead::UserList;
        } else {
            return Err(());
        }

        let from = value["from"]
            .take_string()
            .ok_or(())?
            // .parse::<usize>()
            // .expect("无效id")
            .try_into()
            .expect("在从字符串转化到Target的过程中失败了");

        let target = value["target"]
            .take_string()
            .ok_or(())?
            // .parse::<usize>()
            // .expect("无效id")
            .try_into()
            .expect("在从字符串转化到Target的过程中失败了");
        let msg = value["msg"].take_string().ok_or(())?;

        Ok(Self::new(head, from, target, msg))
    }
}

impl Default for MessageHead {
    fn default() -> Self {
        Self::Display
    }
}

impl Display for MessageHead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MessageHead::Login => "l",
                MessageHead::Message => "s",
                MessageHead::Display => "d",
                MessageHead::SighUp => "n",
                MessageHead::UserList => "u",
            }
        )
    }
}
