#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate validator;
#[macro_use]
extern crate validator_derive;

mod rest;
mod stupiddb;

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                rest::repository::post,
                rest::repository::get,
                rest::repository::push,
                rest::data::post,
            ],
        )
        .launch();
}
