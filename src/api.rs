use serde::Serialize;
use tauri::Emitter;

use crate::models::{Channel, ChannelAvatarData, ChannelFollowsData, FollowedAtData, GqlResponse, ModStatus, UserAvatarData, UserModStatusData};

const TWITCH_GQL: &str = "https://gql.twitch.tv/gql";
const TWITCH_INTEGRITY: &str = "https://gql.twitch.tv/integrity";

// Used for the follows query
const FOLLOWS_CLIENT_ID: &str = "p9lhq6azjkdl72hs5xnt3amqu7vv8k2";
const FOLLOWS_USER_AGENT: &str = "Mozilla/5.0 (SMART-TV; Linux; Tizen 6.0) AppleWebKit/538.1 (KHTML, like Gecko) Version/6.0 TV Safari/538.1";

// Used for supplementary queries (follower counts, mutuals)
const CLIENT_ID: &str = "kimne78kx3ncx6brgo4mv6wki5h1ko";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";
const ORIGIN: &str = "https://www.twitch.tv";

const GET_USER_AVATAR_QUERY: &str = "
query GetUserAvatar($login: String!) {
  user(login: $login) {
    profileImageURL(width: 300)
  }
}";

const GET_FOLLOWING_QUERY: &str = "
query getFollowing($userLogin: String!, $cursor: Cursor) {
  user(login: $userLogin) {
    id
    follows(first: 100, after: $cursor) {
      totalCount
      pageInfo { hasNextPage }
      edges {
        cursor
        followedAt
        node {
          id
          login
          displayName
          profileImageURL(width: 150)
          stream {
            game { name }
            viewersCount
          }
        }
      }
    }
  }
}";

#[derive(Serialize)]
struct InlineGqlRequest<V: Serialize> {
    query: &'static str,
    variables: V,
}

#[derive(Serialize)]
struct GetUserAvatarVars<'a> {
    login: &'a str,
}

#[derive(Serialize)]
struct UserModStatusVars<'a> {
    #[serde(rename = "channelID")]
    channel_id: &'a str,
    #[serde(rename = "userID")]
    user_id: &'a str,
}

#[derive(Serialize)]
struct GetFollowingVars<'a> {
    #[serde(rename = "userLogin")]
    user_login: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    cursor: Option<&'a str>,
}

#[derive(Serialize)]
struct RawGqlRequest {
    query: String,
}

#[derive(Serialize)]
struct GqlRequest<V: Serialize> {
    #[serde(rename = "operationName")]
    operation_name: &'static str,
    variables: V,
    extensions: Extensions,
}

#[derive(Serialize)]
struct Extensions {
    #[serde(rename = "persistedQuery")]
    persisted_query: PersistedQuery,
}

#[derive(Serialize)]
struct PersistedQuery {
    version: u32,
    #[serde(rename = "sha256Hash")]
    sha256_hash: &'static str,
}

#[derive(Serialize)]
struct ChannelAvatarVars<'a> {
    #[serde(rename = "channelLogin")]
    channel_login: &'a str,
}

fn random_device_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("{:x}{:x}", seed, seed.wrapping_mul(0x9e3779b9))
}

async fn fetch_integrity_token(
    client: &reqwest::Client,
    device_id: &str,
) -> anyhow::Result<String> {
    let resp: serde_json::Value = client
        .post(TWITCH_INTEGRITY)
        .header("Client-Id", CLIENT_ID)
        .header("X-Device-Id", device_id)
        .header("User-Agent", USER_AGENT)
        .header("Origin", ORIGIN)
        .header("Referer", ORIGIN)
        .send()
        .await?
        .json()
        .await?;

    resp["token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("no token in integrity response: {resp}"))
}

pub async fn check_mod_status(
    client: &reqwest::Client,
    channel_id: &str,
    user_id: &str,
) -> anyhow::Result<(bool, bool)> {
    let device_id = random_device_id();
    let integrity_token = fetch_integrity_token(client, &device_id).await?;
    let pairs = [
        (channel_id.to_string(), user_id.to_string()),
        (user_id.to_string(), channel_id.to_string()),
    ];
    let results = fetch_mod_status(client, &integrity_token, &device_id, &pairs).await?;
    Ok((results[0], results[1]))
}

async fn fetch_mod_status(
    client: &reqwest::Client,
    integrity_token: &str,
    device_id: &str,
    pairs: &[(String, String)],
) -> anyhow::Result<Vec<bool>> {
    let mut results = Vec::with_capacity(pairs.len());

    for chunk in pairs.chunks(35) {
        let body: Vec<_> = chunk.iter().map(|(channel_id, user_id)| GqlRequest {
            operation_name: "UserModStatus",
            variables: UserModStatusVars { channel_id, user_id },
            extensions: Extensions {
                persisted_query: PersistedQuery {
                    version: 1,
                    sha256_hash: "511b58faf547070bc95b7d32e7b5cdedf8c289a3aeabfc3c5d3ece2de01ae06f",
                },
            },
        }).collect();

        let resp: Vec<GqlResponse<UserModStatusData>> = client
            .post(TWITCH_GQL)
            .header("Client-Id", CLIENT_ID)
            .header("Client-Integrity", integrity_token)
            .header("X-Device-Id", device_id)
            .header("User-Agent", USER_AGENT)
            .header("Origin", ORIGIN)
            .header("Referer", ORIGIN)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        for r in resp {
            let is_mod = r.data.user
                .map(|u| u.is_moderator)
                .unwrap_or(false);
            results.push(is_mod);
        }
    }

    Ok(results)
}

pub async fn fetch_user_avatar(
    client: &reqwest::Client,
    login: &str,
) -> anyhow::Result<Option<String>> {
    let body = InlineGqlRequest {
        query: GET_USER_AVATAR_QUERY,
        variables: GetUserAvatarVars { login },
    };

    let mut resp: Vec<GqlResponse<UserAvatarData>> = client
        .post(TWITCH_GQL)
        .header("Client-Id", FOLLOWS_CLIENT_ID)
        .header("User-Agent", FOLLOWS_USER_AGENT)
        .json(&[body])
        .send()
        .await?
        .json()
        .await?;

    Ok(resp.remove(0).data.user.and_then(|u| u.profile_image_url))
}

pub async fn fetch_follows(
    client: &reqwest::Client,
    login: &str,
    app: &tauri::AppHandle,
) -> anyhow::Result<Vec<Channel>> {
    fetch_follows_inner(client, login, app).await
}

async fn fetch_follows_inner(
    client: &reqwest::Client,
    login: &str,
    app: &tauri::AppHandle,
) -> anyhow::Result<Vec<Channel>> {
    let mut channels: Vec<Channel> = Vec::new();
    let mut cursor: Option<String> = None;
    let mut searched_user_id: Option<String> = None;

    loop {
        let body = InlineGqlRequest {
            query: GET_FOLLOWING_QUERY,
            variables: GetFollowingVars {
                user_login: login,
                cursor: cursor.as_deref(),
            },
        };

        let mut resp: Vec<GqlResponse<ChannelFollowsData>> = client
            .post(TWITCH_GQL)
            .header("Client-Id", FOLLOWS_CLIENT_ID)
            .header("User-Agent", FOLLOWS_USER_AGENT)
            .json(&[body])
            .send()
            .await?
            .json()
            .await?;

        let user = match resp.remove(0).data.user {
            Some(u) => u,
            None => break,
        };
        if searched_user_id.is_none() {
            searched_user_id = Some(user.id.clone());
        }
        let follows = match user.follows {
            Some(f) => f,
            None => break,
        };

        let has_next = follows.page_info.has_next_page;
        cursor = follows.edges.last().map(|e| e.cursor.clone());

        for edge in follows.edges {
            let mut channel = edge.node;
            channel.followed_at = Some(edge.followed_at);
            channels.push(channel);
        }

        if !has_next {
            break;
        }
    }

    let device_id = random_device_id();
    let integrity_token = fetch_integrity_token(client, &device_id).await?;

    app.emit("loading-details", ()).ok();
    fetch_follower_counts(client, &integrity_token, &device_id, &mut channels).await?;

    app.emit("loading-mutuals", ()).ok();
    fetch_mutuals(client, &integrity_token, &device_id, login, &mut channels).await?;

    if let Some(uid) = searched_user_id {
        app.emit("loading-mods", ()).ok();
        fetch_mod_statuses(client, &integrity_token, &device_id, &uid, &mut channels).await?;
    }

    Ok(channels)
}

async fn fetch_mod_statuses(
    client: &reqwest::Client,
    integrity_token: &str,
    device_id: &str,
    user_id: &str,
    channels: &mut Vec<Channel>,
) -> anyhow::Result<()> {
    // Interleaved pairs: (channel→user, user→channel) for each channel.
    // fetch_mod_status chunks these at 35, results come back in the same order.
    let pairs: Vec<(String, String)> = channels.iter()
        .flat_map(|c| [
            (c.id.clone(), user_id.to_string()),   // is searched user a mod in this channel?
            (user_id.to_string(), c.id.clone()),   // is this channel owner a mod in searched user's channel?
        ])
        .collect();

    let results = fetch_mod_status(client, integrity_token, device_id, &pairs).await?;

    for (i, channel) in channels.iter_mut().enumerate() {
        channel.mod_status = match (results[i * 2], results[i * 2 + 1]) {
            (true,  true)  => ModStatus::Mutual,
            (true,  false) => ModStatus::UserModerates,
            (false, true)  => ModStatus::ChannelModerates,
            _              => ModStatus::None,
        };
    }

    Ok(())
}

async fn fetch_follower_counts(
    client: &reqwest::Client,
    integrity_token: &str,
    device_id: &str,
    channels: &mut Vec<Channel>,
) -> anyhow::Result<()> {
    for chunk in channels.chunks_mut(35) {
        let body: Vec<_> = chunk.iter().map(|c| GqlRequest {
            operation_name: "ChannelAvatar",
            variables: ChannelAvatarVars { channel_login: &c.login },
            extensions: Extensions {
                persisted_query: PersistedQuery {
                    version: 1,
                    sha256_hash: "db0e7b54c5e75fcf7874cafca2dacde646344cbbd1a80a2488a7953176c87a68",
                },
            },
        }).collect();

        let resp: Vec<GqlResponse<ChannelAvatarData>> = client
            .post(TWITCH_GQL)
            .header("Client-Id", CLIENT_ID)
            .header("Client-Integrity", integrity_token)
            .header("X-Device-Id", device_id)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        for (channel, r) in chunk.iter_mut().zip(resp) {
            channel.follower_count = r.data.user.map(|u| u.followers.total_count);
        }
    }

    Ok(())
}

async fn fetch_mutuals(
    client: &reqwest::Client,
    integrity_token: &str,
    device_id: &str,
    login: &str,
    channels: &mut Vec<Channel>,
) -> anyhow::Result<()> {
    for chunk in channels.chunks_mut(35) {
        let body: Vec<_> = chunk.iter().map(|c| RawGqlRequest {
            query: format!(
                r#"{{ user(login: "{}") {{ follow(targetLogin: "{}") {{ followedAt }} }} }}"#,
                c.login, login
            ),
        }).collect();

        let resp: Vec<GqlResponse<FollowedAtData>> = client
            .post(TWITCH_GQL)
            .header("Client-Id", CLIENT_ID)
            .header("Client-Integrity", integrity_token)
            .header("X-Device-Id", device_id)
            .header("User-Agent", USER_AGENT)
            .header("Origin", ORIGIN)
            .header("Referer", ORIGIN)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        for (channel, r) in chunk.iter_mut().zip(resp) {
            channel.is_mutual = r.data.user.and_then(|u| u.follow).is_some();
        }
    }

    Ok(())
}
