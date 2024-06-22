use atom_syndication::Feed;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Feeds {
    #[serde(rename = "feed")]
    pub feeds: Vec<Feed>,
}
