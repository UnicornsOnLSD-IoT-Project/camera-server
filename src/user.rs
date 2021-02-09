use crate::{
    api_error::ApiError,
    user_tokens::{self, UserToken},
};

use super::schema::users;
use super::CameraServerDbConn;
use bcrypt;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{self};
use rocket::http::Status;
use rocket::post;

use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use user_tokens::InsertableUserToken;

#[derive(Queryable, AsChangeset, Deserialize, Serialize)]
#[table_name = "users"]
pub struct User {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub password: String,
}

#[derive(Insertable, Deserialize, Serialize)]
#[table_name = "users"]
pub struct InsertableUser {
    pub username: String,
    pub password: String,
}

impl InsertableUser {
    pub fn from_user(user: User) -> InsertableUser {
        InsertableUser {
            username: user.username,
            password: user.password,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub user_id: uuid::Uuid,
}

impl UserInfo {
    pub fn from_user(user: User) -> UserInfo {
        UserInfo {
            username: user.username,
            user_id: user.user_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthentiationResult {
    pub user_info: UserInfo,
    pub user_token: uuid::Uuid,
}

pub fn all(connection: &PgConnection) -> QueryResult<Vec<User>> {
    users::table.load::<User>(&*connection)
}

pub fn get(id: uuid::Uuid, connection: &PgConnection) -> QueryResult<User> {
    users::table.find(id).get_result::<User>(connection)
}

pub fn insert(user: InsertableUser, connection: &PgConnection) -> QueryResult<User> {
    diesel::insert_into(users::table)
        .values(user)
        .get_result(connection)
}

pub fn update(id: uuid::Uuid, user: User, connection: &PgConnection) -> QueryResult<User> {
    diesel::update(users::table.find(id))
        .set(&user)
        .get_result(connection)
}

pub fn delete(id: uuid::Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(users::table.find(id)).execute(connection)
}

pub fn get_by_username(username: String, connection: &PgConnection) -> QueryResult<User> {
    return users::table
        .filter(users::username.eq_all(username))
        .first::<User>(connection);
}

pub fn is_login_valid(username: String, password: String, connection: &PgConnection) -> bool {
    let query = users::table
        .filter(users::username.eq(username))
        .first::<User>(connection);
    match query {
        Ok(result) => match bcrypt::verify(password, &result.password) {
            Ok(is_password_true) => return is_password_true,
            Err(_) => return false,
        },
        Err(_) => return false,
    }
}

#[post("/AddUser", format = "json", data = "<new_user>")]
pub fn add_user(
    conn: CameraServerDbConn,
    new_user: Json<InsertableUser>,
) -> Result<Json<AuthentiationResult>, ApiError> {
    if new_user.password.chars().count() < 8 {
        return Err(ApiError {
            error: "Password must be at least 8 characters long",
            status: Status::UnprocessableEntity,
        });
    }

    // Tries a DB request with the new username. If something comes back, return an error saying the username already exists
    match get_by_username(new_user.username.clone(), &conn) {
        Ok(_) => {
            return Err(ApiError {
                error: "Username already exists",
                status: Status::Conflict,
            });
        }
        Err(_) => {}
    }

    let new_user_insertable = InsertableUser {
        username: new_user.username.clone(),
        password: bcrypt::hash(new_user.password.clone(), bcrypt::DEFAULT_COST).unwrap(),
    };

    // Inserts the new username/pass into the db. Returns a User object, which included the new UUID.
    let new_user_inserted = insert(new_user_insertable, &conn).map_err(|error| {
        println!("Failed to insert user into table! The error was: {}", error);
        ApiError {
            error: "Failed to insert user into table",
            status: Status::InternalServerError,
        }
    })?;

    // Inserts the new user into the token table in order to get a token. If this fails, try to undo what we've done.
    let new_user_token = user_tokens::insert(
        InsertableUserToken {
            user_id: new_user_inserted.user_id,
        },
        &conn,
    )
    .map_err(|error| {
        println!(
            "Failed to get new token for user {} (id: {}). The error was {}",
            new_user.username, new_user_inserted.user_id, error
        );
        delete(new_user_inserted.user_id, &conn)
            .expect("Failed to delete user id while handling token insert error!");
        ApiError {
            error: "Failed to generate token",
            status: Status::InternalServerError,
        }
    })?;

    Ok(Json(AuthentiationResult {
        user_info: UserInfo {
            user_id: new_user_inserted.user_id,
            username: new_user_inserted.username,
        },
        user_token: new_user_token.user_token,
    }))
}

/// Generates a new token for the given user. Actual login checking is handled in the UserLogin request guard.
#[post("/Login", format = "json", data = "<user_login>")]
pub fn login(
    conn: CameraServerDbConn,
    user_login: Json<InsertableUser>,
) -> Result<Json<AuthentiationResult>, ApiError> {
    if !is_login_valid(
        user_login.username.clone(),
        user_login.password.clone(),
        &conn,
    ) {
        return Err(ApiError {
            error: "Invalid username or password",
            status: Status::Unauthorized,
        });
    }

    let user = get_by_username(user_login.username.clone(), &conn).map_err(|error| {
        println!(
            "Failed to get user id from username {}. The error was: {}",
            user_login.username, error
        );
        ApiError {
            error: "Failed to get user id from username",
            status: Status::InternalServerError,
        }
    })?;

    let token = user_tokens::insert(
        user_tokens::InsertableUserToken {
            user_id: user.user_id,
        },
        &conn,
    )
    .map_err(|error| {
        println!(
            "Failed to create token for user {}. The error was {}",
            user_login.username, error
        );
        ApiError {
            error: "Failed to create token",
            status: Status::InternalServerError,
        }
    })?;

    Ok(Json(AuthentiationResult {
        user_info: UserInfo {
            user_id: user.user_id,
            username: user.username,
        },
        user_token: token.user_token,
    }))
}
