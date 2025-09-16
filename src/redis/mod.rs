use std::sync::Arc;

use deadpool_redis::{Config as RedisConfig, Runtime};

use crate::tracker::swarm::{Swarm, SwarmState};

pub fn open() -> eyre::Result<Swarm> {
    let host = "redis://127.0.0.1:6379";
    let redis_config = RedisConfig::from_url(host);
    let redis_pool = redis_config.create_pool(Some(Runtime::Tokio1))?;

    let swarm = Arc::new(SwarmState::new(redis_pool));

    Ok(swarm)
}
