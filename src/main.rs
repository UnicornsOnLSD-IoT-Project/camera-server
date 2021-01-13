#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_contrib;

mod schema;
mod user;

use rocket_contrib::json::Json;

#[database("camera-server-db")]
struct CameraServerDbConn(diesel::PgConnection);

#[get("/")]
fn index(conn: CameraServerDbConn) -> Json<Vec<user::User>> {
    Json(user::all(&conn).unwrap())
}

#[get("/add/<username>/<password>")]
fn add(conn: CameraServerDbConn, username: String, password: String) -> &'static str {
    user::insert(user::InsertableUser { username, password }, &conn).expect("Something went wrong");
    "Did it work?"
}

fn main() {
    rocket::ignite()
        .attach(CameraServerDbConn::fairing())
        .mount("/", routes![index, add])
        .launch();
}
