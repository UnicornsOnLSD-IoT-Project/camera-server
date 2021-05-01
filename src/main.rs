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
mod config;
mod schema;
mod user;
mod user_tokens;
mod users_cameras;

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
                camera::get_latest,
                camera::get_image_list,
                camera::get_image,
                users_cameras::list_cameras,
                config::get_config_user,
                config::get_config_camera,
                config::update_config,
            ],
        )
        .launch();
}
