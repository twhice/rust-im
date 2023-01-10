use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use json::JsonValue;

use crate::string;

#[derive(Debug, Clone)]
pub enum Target {
    User {
        name: String,
        id: usize,
    },
    Group {
        members: Vec<Self>,
        group_name: String,
        id: usize,
    },
}
impl Target {
    pub fn set_id(&mut self, new_id: usize) {
        match self {
            Target::User { name, id: _ } => {
                *self = Target::User {
                    name: name.to_owned(),
                    id: new_id,
                }
            }
            Target::Group {
                members,
                group_name,
                id: _,
            } => {
                *self = Target::Group {
                    members: members.to_owned(),
                    group_name: group_name.to_owned(),
                    id: new_id,
                }
            }
        }
    }
    pub fn get_id(&self) -> usize {
        *(match self {
            Target::User { name: _, id } => id,
            Target::Group {
                members: _,
                group_name: _,
                id,
            } => id,
        })
    }
    pub fn get_name(&self) -> String {
        match self {
            Target::User { name, id: _ } => name.clone(),
            Target::Group {
                members: _,
                group_name,
                id: _,
            } => group_name.clone(),
        }
    }
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        match self {
            Target::User { name, id } => {
                map.insert(string!("name"), name.to_string());
                map.insert(string!("id"), id.to_string());
            }
            Target::Group {
                members: _,
                group_name: gname,
                id,
            } => {
                map.insert(string!("id"), id.to_string());
                map.insert(string!("gname"), gname.to_string());
                todo!("之后吧");
            }
        }
        map
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // match self {
        //     // Target::User { name, id } => write!(f, "N:{name} I:{id}"),
        //     // Target::Group {
        //     //     members,
        //     //     group_name,
        //     //     id,
        //     // } => {
        //     //     let mut members_s = String::from("[");
        //     //     for member in members {
        //     //         members_s += " ";
        //     //         members_s += member.to_string().as_str();
        //     //     }
        //     //     write!(f, "N:{} M:{} I:{id}", group_name, members_s)
        //     // }

        // }
        write!(f, "{}", json::stringify(self.to_map()).to_string())
    }
}
impl Default for Target {
    fn default() -> Self {
        Self::User {
            name: "unknow".to_owned(),
            id: 0,
        }
    }
}
impl From<usize> for Target {
    fn from(id: usize) -> Self {
        let mut target = Self::default();
        target.set_id(id);
        target
    }
}
impl TryFrom<String> for Target {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(match json::parse(&value) {
            Ok(o) => o,
            Err(_) => return Err(()),
        })
    }
}
impl TryFrom<JsonValue> for Target {
    type Error = ();

    fn try_from(mut value: JsonValue) -> Result<Self, Self::Error> {
        let name = value["name"].take_string().ok_or(())?;
        let id = value["id"]
            .take_string()
            .ok_or(())?
            .parse::<usize>()
            .unwrap();
        Ok(Self::User { name, id })
    }
}
