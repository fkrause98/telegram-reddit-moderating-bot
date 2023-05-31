use anyhow::Result;
use core::fmt;
use serde::{Deserialize, Serialize};
// This is what redit calls a "thing"
#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    // TODO:
    // This breaks when a post is a video.
    pub link_permalink: String,
    // This field is actually "",
    // is one of the many identifiers for a "thing",
    // (see https://www.reddit.com/dev/api/#fullnames)
    // but I'm going to use it as an identifier
    // for each post.
    // Can't use 'id' because serde complains.
    #[serde(alias = "link_id")]
    pub post_id: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct PostData {
    pub data: Post,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ModQueueJson {
    pub children: Vec<PostData>,
}

pub enum ModAction {
    Approve,
    Remove,
}

impl fmt::Display for ModAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModAction::Approve => write!(f, "approve"),
            ModAction::Remove => write!(f, "remove"),
        }
    }
}
impl TryFrom<&str> for ModAction {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Self> {
        match s {
            "approve" => Ok(ModAction::Approve),
            "remove" => Ok(ModAction::Remove),
            _ => Err(anyhow::anyhow!("Wrong variant: {s}")),
        }
    }
}
