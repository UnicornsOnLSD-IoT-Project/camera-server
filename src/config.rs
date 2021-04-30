use crate::camera_tokens::CameraToken;
use crate::user_tokens::UserToken;
use crate::CameraServerDbConn;
use crate::{api_error::ApiError, users_cameras::check_if_user_has_access_to_camera};

use super::schema::configs;
use diesel::prelude::*;
use diesel::{self};
use rocket::http::Status;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Queryable, AsChangeset, Insertable, Deserialize, Serialize)]
#[table_name = "configs"]
pub struct Config {
    pub camera_id: uuid::Uuid,
    pub interval: i16,
}

pub fn all(connection: &PgConnection) -> QueryResult<Vec<Config>> {
    configs::table.load::<Config>(&*connection)
}

pub fn get(camera_id: uuid::Uuid, connection: &PgConnection) -> QueryResult<Config> {
    configs::table
        .find(camera_id)
        .get_result::<Config>(connection)
}

pub fn insert(config: Config, connection: &PgConnection) -> QueryResult<Config> {
    diesel::insert_into(configs::table)
        .values(config)
        .get_result(connection)
}

pub fn update(
    camera_id: uuid::Uuid,
    config: Config,
    connection: &PgConnection,
) -> QueryResult<Config> {
    diesel::update(configs::table.find(camera_id))
        .set(&config)
        .get_result(connection)
}

pub fn delete(camera_id: uuid::Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(configs::table.find(camera_id)).execute(connection)
}

#[get("/Cameras/<camera_id_string>/GetConfigUser")]
/// Retrieves a camera's config, authenticates with a user token.
pub fn get_config_user(
    conn: CameraServerDbConn,
    camera_id_string: String,
    user_token: UserToken,
) -> Result<Json<Config>, ApiError> {
    check_if_user_has_access_to_camera(&conn, &user_token, &camera_id_string)?;

    let camera_id = uuid::Uuid::parse_str(&camera_id_string).map_err(|error| {
        println!(
            "Failed to parse camera id into UUID: Input was {}, error was {}",
            camera_id_string, error
        );
        ApiError {
            error: "Failed to parse camera ID string",
            status: Status::UnprocessableEntity,
        }
    })?;

    let config = get(camera_id, &conn).map_err(|error| {
        println!("Failed to read camera config! The error was {}", error);
        return ApiError {
            error: "Failed to read config",
            status: Status::InternalServerError,
        };
    })?;

    Ok(Json(config))
}

#[get("/Cameras/GetConfigCamera")]
/// Retrieves a camera's config, authenticates with a camera token.
pub fn get_config_camera(
    conn: CameraServerDbConn,
    camera_token: CameraToken,
) -> Result<Json<Config>, ApiError> {
    let config = get(camera_token.camera_id, &conn).map_err(|error| {
        println!("Failed to read camera config! The error was {}", error);
        return ApiError {
            error: "Failed to read config",
            status: Status::InternalServerError,
        };
    })?;

    Ok(Json(config))
}
