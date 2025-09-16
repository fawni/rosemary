use actix_web::{App, HttpServer, web};

const HOST: &str = "0.0.0.0";
const PORT: u16 = 6969;

mod redis;
mod routes;
mod tracker;

#[actix_web::main]
async fn main() -> eyre::Result<()> {
    twink::log::setup();

    let swarm = redis::open()?;

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(swarm.clone()))
        .wrap(actix_logger::Logger::new(twink::fmt!(
            "<green>%s <purple>%r</> took <yellow>%Dms</> | <cyan><b>%{X-Forwarded-For}i</> <i>%{User-Agent}i</>"
        )))
        .service(routes::hai)
        .service(routes::announce)
    })
    .bind((HOST, PORT))?
    .run()
    .await?;

    Ok(())
}
