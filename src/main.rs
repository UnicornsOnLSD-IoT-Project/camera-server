#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

use rocket_contrib::databases::diesel;

#[database("camera-server-db")]
struct CameraServerDbConn(diesel::PgConnection);

mod user;

#[get("/")]
fn index() -> &'static str {
    "Hello World!"
}

fn main() {
    rocket::ignite()
        .attach(CameraServerDbConn::fairing())
        .mount("/", routes![index])
        .launch();
}
