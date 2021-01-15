use crate::CameraServerDbConn;

use super::schema::user_tokens;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{self};
use rocket::{
    http::Status,
    request::{self, FromRequest},
    Outcome, Request,
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum TokenError {
    ParseError,
    NotFound,
    NoTokenProvided,
}

#[derive(Queryable, AsChangeset, Deserialize, Serialize, Debug)]
#[table_name = "user_tokens"]
pub struct UserToken {
    pub token: uuid::Uuid,
    pub user_id: uuid::Uuid,
}

impl<'a, 'r> FromRequest<'a, 'r> for UserToken {
    type Error = TokenError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let token = request.headers().get_one("token");
        match token {
            Some(token) => {
                let parsed_token = match uuid::Uuid::parse_str(token) {
                    Ok(parsed_token_ok) => parsed_token_ok,
                    // Token cannot be parsed into a UUID
                    Err(_) => {
                        return Outcome::Failure((Status::BadRequest, TokenError::ParseError))
                    }
                };
                let connection = CameraServerDbConn::from_request(&request).unwrap();
                match get(parsed_token, &connection) {
                    Ok(user_token) => return Outcome::Success(user_token),

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
#[table_name = "user_tokens"]
pub struct InsertableUserToken {
    pub user_id: uuid::Uuid,
}

impl InsertableUserToken {
    pub fn from_user_token(user_token: UserToken) -> InsertableUserToken {
        InsertableUserToken {
            user_id: user_token.user_id,
        }
    }
}

pub fn all(connection: &PgConnection) -> QueryResult<Vec<UserToken>> {
    user_tokens::table.load::<UserToken>(&*connection)
}

pub fn get(token: uuid::Uuid, connection: &PgConnection) -> QueryResult<UserToken> {
    user_tokens::table
        .find(token)
        .get_result::<UserToken>(connection)
}

pub fn insert(
    user_token: InsertableUserToken,
    connection: &PgConnection,
) -> QueryResult<UserToken> {
    diesel::insert_into(user_tokens::table)
        .values(user_token)
        .get_result(connection)
}

pub fn update(
    token: uuid::Uuid,
    user_token: UserToken,
    connection: &PgConnection,
) -> QueryResult<UserToken> {
    diesel::update(user_tokens::table.find(token))
        .set(&user_token)
        .get_result(connection)
}

pub fn delete(token: uuid::Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(user_tokens::table.find(token)).execute(connection)
}
