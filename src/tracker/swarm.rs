use std::{net::IpAddr, sync::Arc};

use bincode::{Decode, Encode};
use deadpool_redis::{Config as RedisConfig, Pool, Runtime, redis::AsyncCommands};
use serde::{Deserialize, Serialize};

pub type Swarm = Arc<SwarmState>;

#[derive(Clone)]
pub struct SwarmState {
    pub pool: Pool,
}

impl SwarmState {
    pub const fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn add_peer(&self, info_hash: &str, peer: Peer, is_seeder: bool) -> eyre::Result<()> {
        let mut conn = self.pool.get().await?;

        if is_seeder {
            conn.sadd::<_, _, ()>(Peer::key_seeders(info_hash), peer.bin_encode()?)
                .await?;
        } else {
            conn.sadd::<_, _, ()>(Peer::key_leechers(info_hash), peer.bin_encode()?)
                .await?;
        }

        Ok(())
    }

    pub async fn remove_peer(&self, info_hash: &str, peer: Peer) -> eyre::Result<()> {
        let mut conn = self.pool.get().await?;

        conn.srem::<_, _, ()>(Peer::key_seeders(info_hash), peer.bin_encode()?)
            .await?;
        conn.srem::<_, _, ()>(Peer::key_leechers(info_hash), peer.bin_encode()?)
            .await?;

        Ok(())
    }

    pub async fn promote_peer(&self, info_hash: &str, peer: Peer) -> eyre::Result<()> {
        let mut conn = self.pool.get().await?;

        conn.srem::<_, _, ()>(Peer::key_leechers(info_hash), peer.bin_encode()?)
            .await?;
        conn.sadd::<_, _, ()>(Peer::key_seeders(info_hash), peer.bin_encode()?)
            .await?;

        Ok(())
    }

    pub async fn peers(
        &self,
        info_hash: &str,
        is_seeder: bool,
        numwant: usize,
    ) -> eyre::Result<(Vec<Peer>, Vec<Peer>)> {
        let mut conn = self.pool.get().await?;
        let (mut peers, mut peers6): (Vec<Peer>, Vec<Peer>) = (Vec::new(), Vec::new());

        let leechers: Vec<Vec<u8>> = conn.smembers(Peer::key_leechers(info_hash)).await?;

        if !is_seeder {
            let seeders: Vec<Vec<u8>> = conn.smembers(Peer::key_seeders(info_hash)).await?;

            for raw in seeders {
                let peer = Peer::bin_decode(&raw)?;

                if peer.ip.is_ipv4() {
                    peers.push(peer);
                } else {
                    peers6.push(peer);
                }
            }
        }

        for raw in leechers {
            let peer = Peer::bin_decode(&raw)?;

            if peer.ip.is_ipv4() {
                peers.push(peer);
            } else {
                peers6.push(peer);
            }
        }

        if peers.len() > numwant {
            peers.truncate(numwant);
        }

        if peers6.len() > numwant {
            peers6.truncate(numwant);
        }

        Ok((peers, peers6))
    }

    pub async fn peer_stats(&self, info_hash: &str) -> eyre::Result<(usize, usize)> {
        let mut conn = self.pool.get().await?;

        let seeders: usize = conn.scard(Peer::key_seeders(info_hash)).await?;
        let leechers: usize = conn.scard(Peer::key_leechers(info_hash)).await?;

        Ok((seeders, leechers))
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct Peer {
    id: String,
    ip: IpAddr,
    port: u16,
}

impl Peer {
    pub const fn new(id: String, ip: IpAddr, port: u16) -> Self {
        Self { id, ip, port }
    }

    pub fn bin_encode(&self) -> eyre::Result<Vec<u8>> {
        Ok(bincode::serde::encode_to_vec(
            self,
            bincode::config::standard(),
        )?)
    }

    pub fn bin_decode(data: &[u8]) -> eyre::Result<Self> {
        let (peer, _) = bincode::decode_from_slice(data, bincode::config::standard())?;

        Ok(peer)
    }

    pub fn key_seeders(info_hash: &str) -> String {
        format!("info:{info_hash}:seeders")
    }

    pub fn key_leechers(info_hash: &str) -> String {
        format!("info:{info_hash}:leechers")
    }
}

pub fn open() -> eyre::Result<Swarm> {
    let host = "redis://127.0.0.1:6379";
    let redis_config = RedisConfig::from_url(host);
    let redis_pool = redis_config.create_pool(Some(Runtime::Tokio1))?;

    let swarm = Arc::new(SwarmState::new(redis_pool));

    Ok(swarm)
}
