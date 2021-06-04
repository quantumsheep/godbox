use actix_web::{App, HttpServer};
use std::io;

extern crate derive_more;

#[macro_use]
extern crate derive_builder;

mod api_helpers;
mod isolate;
mod routes;
mod runner;

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| App::new().service(routes::run_post::route))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
