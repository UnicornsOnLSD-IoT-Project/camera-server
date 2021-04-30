use crate::{enums::token_error::TokenError, CameraServerDbConn};

use super::schema::camera_tokens;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{self};
use rocket::{http::Status, request, request::FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};

#[derive(Queryable, AsChangeset, Deserialize, Serialize)]
#[table_name = "camera_tokens"]
pub struct CameraToken {
    pub camera_token: uuid::Uuid,
    pub camera_id: uuid::Uuid,
}

impl<'a, 'r> FromRequest<'a, 'r> for CameraToken {
    type Error = TokenError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let token = request.headers().get_one("camera_token");
        match token {
            Some(token) => {
                let parsed_token = match uuid::Uuid::parse_str(token) {
                    Ok(parsed_token_ok) => parsed_token_ok,
                    // Token cannot be parsed into a UUID
                    Err(_) => {
                        return Outcome::Failure((Status::BadRequest, TokenError::ParseError))
                    }
                };
                let connection = CameraServerDbConn::from_request(&request)
                    .expect("Failed to get DB connection on CameraToken request guard");
                match get(parsed_token, &connection) {
                    Ok(camera_token) => return Outcome::Success(camera_token),

                    Err(_) => {
                        return Outcome::Failure((Status::Unauthorized, TokenError::NotFound))
                    }
                }
            }
            // Token does not exist
            None => Outcome::Failure((Status::Unauthorized, TokenError::NoTokenProvided)),
        }
    }
}

#[derive(Insertable, Deserialize, Serialize)]
#[table_name = "camera_tokens"]
pub struct InsertableCameraToken {
    pub camera_id: uuid::Uuid,
}

impl InsertableCameraToken {
    pub fn from_camera_token(camera_token: CameraToken) -> InsertableCameraToken {
        InsertableCameraToken {
            camera_id: camera_token.camera_id,
        }
    }
}

pub fn all(connection: &PgConnection) -> QueryResult<Vec<CameraToken>> {
    camera_tokens::table.load::<CameraToken>(&*connection)
}

pub fn get(camera_id: uuid::Uuid, connection: &PgConnection) -> QueryResult<CameraToken> {
    camera_tokens::table
        .find(camera_id)
        .get_result::<CameraToken>(connection)
}

pub fn insert(
    camera_token: InsertableCameraToken,
    connection: &PgConnection,
) -> QueryResult<CameraToken> {
    diesel::insert_into(camera_tokens::table)
        .values(camera_token)
        .get_result(connection)
}

pub fn update(
    camera_id: uuid::Uuid,
    camera_token: CameraToken,
    connection: &PgConnection,
) -> QueryResult<CameraToken> {
    diesel::update(camera_tokens::table.find(camera_id))
        .set(&camera_token)
        .get_result(connection)
}

pub fn delete(camera_id: uuid::Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(camera_tokens::table.find(camera_id)).execute(connection)
}
