use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct GqlResponse<T> {
    pub data: T,
    pub errors: Option<Vec<GqlError>>,
}

#[derive(Deserialize, Debug)]
pub struct GqlError {
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct ChannelFollowsData {
    pub user: Option<User>,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub follows: Option<FollowConnection>,
}

#[derive(Deserialize, Debug)]
pub struct FollowConnection {
    #[serde(rename = "totalCount")]
    pub total_count: u32,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
    pub edges: Vec<FollowEdge>,
}

#[derive(Deserialize, Debug)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

#[derive(Deserialize, Debug)]
pub struct FollowEdge {
    pub cursor: String,
    #[serde(rename = "followedAt")]
    pub followed_at: String,
    pub node: Channel,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Channel {
    pub login: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "profileImageURL")]
    pub profile_image_url: Option<String>,
    pub stream: Option<Stream>,
    #[serde(default)]
    pub follower_count: Option<u32>,
    #[serde(default)]
    pub followed_at: Option<String>,
    #[serde(default)]
    pub is_mutual: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Stream {
    pub game: Option<Game>,
    #[serde(rename = "viewersCount")]
    pub viewers_count: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct FollowedAtData {
    pub user: Option<FollowedAtUser>,
}

#[derive(Deserialize, Debug)]
pub struct FollowedAtUser {
    pub follow: Option<FollowedAt>,
}

#[derive(Deserialize, Debug)]
pub struct FollowedAt {
    #[serde(rename = "followedAt")]
    pub followed_at: String,
}

#[derive(Deserialize, Debug)]
pub struct ChannelAvatarData {
    pub user: Option<ChannelAvatarUser>,
}

#[derive(Deserialize, Debug)]
pub struct ChannelAvatarUser {
    pub followers: FollowerCount,
}

#[derive(Deserialize, Debug)]
pub struct FollowerCount {
    #[serde(rename = "totalCount")]
    pub total_count: u32,
}
