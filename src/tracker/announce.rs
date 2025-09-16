use std::net::IpAddr;

use serde::{Deserialize, Deserializer, Serialize, de};

use super::swarm::Peer;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all_fields = "lowercase")] // doesn't work?
pub enum Event {
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "completed")]
    Completed,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AnnounceRequest {
    #[serde(deserialize_with = "deserialize_url_encode")]
    pub info_hash: String,
    #[serde(deserialize_with = "deserialize_url_encode")]
    pub peer_id: String,
    pub port: u16,
    pub left: u64,
    pub event: Option<Event>,
    pub ip: Option<IpAddr>,
    pub numwant: Option<usize>,
}

fn deserialize_url_encode<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let buf: &str = de::Deserialize::deserialize(deserializer)?;
    let decoded = urlencoding::decode(buf).unwrap().to_string();
    if decoded.len() == 20 {
        Ok(decoded)
    } else {
        Err(de::Error::custom(
            "URL-encoded parameters should be 20 bytes in length",
        ))
    }
}

#[derive(Debug, Serialize)]
pub enum AnnounceResponse {
    Failure {
        failure_reason: String,
    },
    Success {
        interval: u64,
        complete: usize,
        incomplete: usize,
        peers: Vec<Peer>,
        peers6: Vec<Peer>,
    },
}

impl AnnounceResponse {
    pub fn fail(reason: &str) -> Self {
        Self::Failure {
            failure_reason: reason.to_owned(),
        }
    }
}
