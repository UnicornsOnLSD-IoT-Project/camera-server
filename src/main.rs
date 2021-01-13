#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_contrib;

extern crate bcrypt;

mod schema;
mod user;

use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response;
use rocket::response::{Responder, Response};
use rocket_contrib::json::{Json, JsonValue};

#[database("camera-server-db")]
pub struct CameraServerDbConn(diesel::PgConnection);

#[derive(Debug)]
pub struct ApiResponse {
    json: JsonValue,
    status: Status,
}

impl<'r> Responder<'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[get("/")]
fn index(conn: CameraServerDbConn) -> Json<Vec<user::User>> {
    Json(user::all(&conn).unwrap())
}

fn main() {
    rocket::ignite()
        .attach(CameraServerDbConn::fairing())
        .mount("/", routes![index, user::add_user])
        .launch();
}
