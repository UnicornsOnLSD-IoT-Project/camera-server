use crate::{
    api_error::ApiError,
    camera_tokens,
    config::{self, Config},
    user_tokens,
    users_cameras::{self, check_if_user_has_access_to_camera, InsertableUsersCamera},
    CameraServerDbConn,
};

use super::schema::cameras;
use camera_tokens::{CameraToken, InsertableCameraToken};
use diesel::prelude::*;
use diesel::{self};
use rocket::post;
use rocket::response::Stream;
use rocket::{http::Status, Data};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::time::SystemTime;
use std::{env, fs::create_dir_all, fs::read_dir, fs::DirEntry};

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

/// Returns a sorted image list of the given directory (usually a camera directory in this case)
/// Not to be confused the get_image_list() GET request (couldn't think of a better name).
/// Returns a Vec<DirEntry> if successful, and an ApiError if something goes wrong.
pub fn list_camera_directory(
    camera_directory: &String,
    sort: bool,
) -> Result<Vec<DirEntry>, ApiError> {
    let image_list = read_dir(camera_directory).map_err(|error| {
        println!(
            "Failed to directory {}! The error was {}",
            camera_directory, error
        );
        ApiError {
            error: "Failed to get list of images",
            status: Status::InternalServerError,
        }
    })?;

    let mut sorted_image_list: Vec<DirEntry> = image_list
        .map(|x| x.expect("Failed to map to Vec<DirEntry>"))
        .collect::<Vec<DirEntry>>();

    if sorted_image_list.len() == 0 {
        return Err(ApiError {
            error: "Camera has no images (or doesn't exist)",
            status: Status::NotFound,
        });
    }

    if sort {
        sorted_image_list.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    }

    Ok(sorted_image_list)
}

pub fn images_directory() -> String {
    env::var("IMAGES_DIRECTORY").expect("IMAGES_DIRECTORY environment variable is not set!")
}

pub fn camera_directory(images_directory_path: &String, camera_id_string: &String) -> String {
    format!("{}/{}", images_directory_path, camera_id_string)
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
    let users_camera = users_cameras::insert(
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

    config::insert(
        Config {
            camera_id: new_camera.camera_id,
            interval: 10,
        },
        &conn,
    )
    .map_err(|error| {
        println!(
            "Failed to add config for {}! The error was {}",
            new_camera.camera_id, error
        );
        users_cameras::delete(users_camera.users_cameras_id, &conn)
            .expect("Failed to delete users camera while handling create config error!");
        camera_tokens::delete(new_camera.camera_id, &conn)
            .expect("Failed to delete new camera token while handling pair user to camera error!");
        delete(new_camera.camera_id, &conn)
            .expect("Failed to delete new camera while handling pair user to camera error!");
        return ApiError {
            error: "Failed to create camera config",
            status: Status::InternalServerError,
        };
    })?;

    Ok(Json(new_camera_token))
}

/// Stores a new image. Returns the seconds since epoch used as the image name
#[post("/UploadImage", format = "image/jpeg", data = "<image>")]
pub fn upload_image(image: Data, camera_token: CameraToken) -> Result<String, ApiError> {
    let images_directory = images_directory();

    create_dir_all(format!("{}/{}", &images_directory, camera_token.camera_id))
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

#[get("/Cameras/<camera_id_string>/LatestImage", format = "image/jpeg")]
pub fn get_latest(
    conn: CameraServerDbConn,
    user_token: user_tokens::UserToken,
    camera_id_string: String,
) -> Result<Stream<File>, ApiError> {
    check_if_user_has_access_to_camera(&conn, &user_token, &camera_id_string)?;

    let images_directory_path = images_directory();

    let camera_directory = camera_directory(&images_directory_path, &camera_id_string);

    let sorted_image_list = list_camera_directory(&camera_directory, true)?;

    // It should be OK to do an expect() here since sorted_directory_list() already returns an error if the dir list is empty
    File::open(
        sorted_image_list
            .last()
            .expect("Failed to get the last element of the sorted image list somehow?")
            .path(),
    )
    .map(Stream::from)
    .map_err(|error| {
        println!("Failed to read file! The error was {}", error);
        ApiError {
            error: "Failed to load image",
            status: Status::InternalServerError,
        }
    })
}

#[get("/Cameras/<camera_id_string>/ImageList")]
pub fn get_image_list(
    conn: CameraServerDbConn,
    user_token: user_tokens::UserToken,
    camera_id_string: String,
) -> Result<Json<Vec<String>>, ApiError> {
    let images_directory_path = images_directory();
    let camera_directory = camera_directory(&images_directory_path, &camera_id_string);

    check_if_user_has_access_to_camera(&conn, &user_token, &camera_id_string)?;

    let sorted_directory_list = list_camera_directory(&camera_directory, true)?
        .iter()
        .map(|x| {
            // This horrible thing removes the extension from the file name and effectively converts the DirEntrys to Strings (since returning &strs in Vecs is awkward to do)
            Path::new(&x.file_name()).file_stem().expect("file_stem returned None! This should only happen if a file doesn't have a name somehow").to_str().expect("Failed to convert &OsStr to &str!").to_string()
        })
        .collect();

    Ok(Json(sorted_directory_list))
}

#[get(
    "/Cameras/<camera_id_string>/Image/<image_id_string>",
    format = "image/jpeg"
)]
pub fn get_image(
    conn: CameraServerDbConn,
    user_token: user_tokens::UserToken,
    camera_id_string: String,
    image_id_string: String,
) -> Result<Stream<File>, ApiError> {
    check_if_user_has_access_to_camera(&conn, &user_token, &camera_id_string)?;

    let images_directory_path = images_directory();

    let camera_directory = camera_directory(&images_directory_path, &camera_id_string);

    let image_list = list_camera_directory(&camera_directory, false)?;

    let image_list_basenames: Vec<String> = image_list.iter()
    .map(|x| {
        Path::new(&x.file_name()).file_stem().expect("file_stem returned None! This should only happen if a file doesn't have a name somehow").to_str().expect("Failed to convert &OsStr to &str!").to_string()
    })
    .collect();

    let image_index = image_list_basenames
        .iter()
        .position(|x| x == &image_id_string)
        .ok_or(ApiError {
            error: "Image not found",
            status: Status::NotFound,
        })?;

    File::open(image_list[image_index].path())
        .map(Stream::from)
        .map_err(|error| {
            println!("Failed to read file! The error was {}", error);
            ApiError {
                error: "Failed to load image",
                status: Status::InternalServerError,
            }
        })
}
