use actix_web::{HttpRequest, HttpResponse, get, web};

use crate::tracker::{
    announce::{AnnounceRequest, AnnounceResponse, Event},
    swarm::{Peer, Swarm},
};

#[get("/")]
async fn hai() -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::PermanentRedirect()
        .insert_header(("Location", "https://github.com/fawni/rosemary"))
        .finish())
}

macro_rules! fail {
    ($e:literal ) => {
        ::twink::hiss!($e);

        return Ok(HttpResponse::BadRequest()
            .body(serde_bencode::to_string(&AnnounceResponse::fail($e)).expect("huh?")));
    };
}

#[get("/announce")]
async fn announce(
    request: HttpRequest,
    params: web::Query<AnnounceRequest>,
    swarm: web::Data<Swarm>,
) -> actix_web::Result<HttpResponse> {
    let announce = params.0;

    let info_hash = announce.info_hash;
    let is_seeder = announce.left == 0;

    if let Some(event) = announce.event {
        let ip = match announce.ip {
            Some(ip) => ip,
            None => {
                if let Some(socket) = request.peer_addr() {
                    socket.ip()
                } else {
                    fail!("Could not determine peer's ip address");
                }
            }
        };

        let peer = Peer::new(announce.peer_id, ip, announce.port);

        match event {
            Event::Started => {
                if (swarm.add_peer(&info_hash, peer, is_seeder).await).is_err() {
                    fail!("Failed to add peer to swarm");
                }
            }
            Event::Stopped => {
                if (swarm.remove_peer(&info_hash, peer).await).is_err() {
                    fail!("Failed to remove peer from swarm");
                }
            }
            Event::Completed => {
                if (swarm.promote_peer(&info_hash, peer).await).is_err() {
                    fail!("Failed to promote peer to seeder");
                }
            }
        }
    }

    let numwant = announce.numwant.unwrap_or(30);

    let Ok((peers, peers6)) = swarm.peers(&info_hash, is_seeder, numwant).await else {
        fail!("Failed to get peers");
    };

    let (seeders, leechers): (usize, usize) =
        if let Ok((seeders, leechers)) = swarm.peer_stats(&info_hash).await {
            (seeders, leechers)
        } else {
            fail!("Failed to get peer stats");
        };

    let response = AnnounceResponse::Success {
        interval: 30 * 60,
        complete: seeders,
        incomplete: leechers,
        peers,
        peers6,
    };

    let Ok(bencode) = serde_bencode::to_string(&response) else {
        fail!("Failed to serialize response to bencode");
    };

    Ok(HttpResponse::Ok().body(bencode))
}
