#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_contrib;

extern crate bcrypt;

mod camera;
mod camera_tokens;
mod enums {
    pub mod token_error;
}
mod api_error;
mod schema;
mod user;
mod user_tokens;
mod users_cameras;

use rocket_contrib::json::Json;

#[database("camera-server-db")]
pub struct CameraServerDbConn(diesel::PgConnection);

fn main() {
    rocket::ignite()
        .attach(CameraServerDbConn::fairing())
        .mount(
            "/",
            routes![
                user::add_user,
                user::login,
                camera::add_new_camera,
                camera::upload_image,
            ],
        )
        .launch();
}
