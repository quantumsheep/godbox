#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate derive_builder;

mod api_helpers;
mod isolate;
mod routes;
mod runner;

fn main() {
    rocket::ignite()
        .mount("/", routes![routes::run_post::route])
        .register(api_helpers::generate_catchers())
        .launch();
}
