use std::net::IpAddr;

use serde::{Deserialize, Serialize};

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
    pub info_hash: String,
    pub port: u16,
    pub left: u64,
    pub event: Option<Event>,
    pub peer_id: String,
    pub ip: Option<IpAddr>,
    pub numwant: Option<usize>,
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
