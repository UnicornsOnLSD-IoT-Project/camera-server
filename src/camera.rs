use crate::{
    api_error::ApiError,
    camera_tokens, user_tokens,
    users_cameras::{self, InsertableUsersCamera},
    CameraServerDbConn,
};

use super::schema::cameras;
use camera_tokens::{CameraToken, InsertableCameraToken};
use diesel::prelude::*;
use diesel::{self};
use rocket::post;
use rocket::{http::Status, Data};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::{env, fs::create_dir_all};

#[derive(Queryable, AsChangeset, Deserialize, Serialize)]
#[table_name = "cameras"]
pub struct Camera {
    pub camera_id: uuid::Uuid,
    pub name: String,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "cameras"]
pub struct InsertableCamera {
    pub name: String,
}

impl InsertableCamera {
    pub fn from_camera(camera: Camera) -> InsertableCamera {
        InsertableCamera { name: camera.name }
    }
}

pub fn all(connection: &PgConnection) -> QueryResult<Vec<Camera>> {
    cameras::table.load::<Camera>(&*connection)
}

pub fn get(camera_id: uuid::Uuid, connection: &PgConnection) -> QueryResult<Camera> {
    cameras::table
        .find(camera_id)
        .get_result::<Camera>(connection)
}

pub fn insert(camera: InsertableCamera, connection: &PgConnection) -> QueryResult<Camera> {
    diesel::insert_into(cameras::table)
        .values(camera)
        .get_result(connection)
}

pub fn update(
    camera_id: uuid::Uuid,
    camera: Camera,
    connection: &PgConnection,
) -> QueryResult<Camera> {
    diesel::update(cameras::table.find(camera_id))
        .set(&camera)
        .get_result(connection)
}

pub fn delete(camera_id: uuid::Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(cameras::table.find(camera_id)).execute(connection)
}

#[post("/AddCamera", format = "json", data = "<camera_name>")]
pub fn add_new_camera(
    conn: CameraServerDbConn,
    user_token: user_tokens::UserToken,
    camera_name: Json<InsertableCamera>,
) -> Result<Json<CameraToken>, ApiError> {
    // Insert a new camera into the DB. Returns the ID for the new camera.
    let new_camera = insert(camera_name.into_inner(), &conn).map_err(|error| {
        println!("Failed to create new camera! The error was {}", error);
        ApiError {
            error: "Failed to create new camera",
            status: Status::InternalServerError,
        }
    })?;

    // Generate a new token for the camera.
    let new_camera_token = camera_tokens::insert(
        InsertableCameraToken {
            camera_id: new_camera.camera_id,
        },
        &conn,
    )
    .map_err(|error| {
        println!(
            "Failed to add camera token for camera {}! The error was {}",
            new_camera.camera_id, error
        );
        delete(new_camera.camera_id, &conn)
            .expect("Failed to delete new camera while handling camera token error!");
        return ApiError {
            error: "Failed to add camera token",
            status: Status::InternalServerError,
        };
    })?;

    // Create a pair between the current user and the new camera.
    // This basically means the user who made the camera is automatically given access.
    users_cameras::insert(
        InsertableUsersCamera {
            camera_id: new_camera.camera_id,
            user_id: user_token.user_id,
        },
        &conn,
    )
    .map_err(|error| {
        println!(
            "Failed to pair user {} to camera {}! The error was {}",
            user_token.user_id, new_camera.camera_id, error
        );
        camera_tokens::delete(new_camera.camera_id, &conn)
            .expect("Failed to delete new camera token while handling pair user to camera error!");
        delete(new_camera.camera_id, &conn)
            .expect("Failed to delete new camera while handling pair user to camera error!");
        return ApiError {
            error: "Failed to pair user to camera",
            status: Status::InternalServerError,
        };
    })?;

    Ok(Json(new_camera_token))
}

/// Stores a new image. Returns the seconds since epoch used as the image name
#[post("/UploadImage", format = "image/jpeg", data = "<image>")]
pub fn upload_image(image: Data, camera_token: CameraToken) -> Result<String, ApiError> {
    let images_directory =
        env::var("IMAGES_DIRECTORY").expect("IMAGES_DIRECTORY environment variable is not set!");

    create_dir_all(format!(
        "{}/{}",
        images_directory.clone(),
        camera_token.camera_id
    ))
    .expect("Failed to create images directory");

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Failed to get current time somehow?")
        .as_secs();

    image
        .stream_to_file(format!(
            "{}/{}/{}.jpg",
            images_directory, camera_token.camera_id, current_time
        ))
        .map_err(|error| {
            println!("Failed to stream image to file! The error was {}", error);
            ApiError {
                error: "Failed to save image to server",
                status: Status::InternalServerError,
            }
        })?;

    Ok(current_time.to_string())
}
