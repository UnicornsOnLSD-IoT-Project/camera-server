use crate::{
    user_tokens::{self, UserToken},
    ApiResponse,
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

#[derive(Debug)]
pub enum LoginError {
    UsernameNotFound,
    IncorrectUsernameOrPassword,
    UserLoginNotProvided,
    ParseError,
}

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
    pub token: UserToken,
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
pub fn add_user(conn: CameraServerDbConn, new_user: Json<InsertableUser>) -> ApiResponse {
    let new_user_decoded = new_user.into_inner();

    if new_user_decoded.password.chars().count() < 8 {
        return ApiResponse {
            json: json!({"error": "Password must be at least 8 characters long"}),
            status: Status::UnprocessableEntity,
        };
    }

    // if get_by_username(new_user_decoded.username.clone(), &conn) {
    //     return ApiResponse {
    //         json: json!({"error": "Username already exists"}),
    //         status: Status::Conflict,
    //     };
    // }
    match get_by_username(new_user_decoded.username.clone(), &conn) {
        Ok(_) => {
            return ApiResponse {
                json: json!({"error": "Username already exists"}),
                status: Status::Conflict,
            };
        }
        Err(_) => {}
    }

    let new_user_insertable = InsertableUser {
        username: new_user_decoded.username,
        password: bcrypt::hash(new_user_decoded.password, bcrypt::DEFAULT_COST).unwrap(),
    };

    let new_user_inserted = insert(new_user_insertable, &conn).unwrap();
    let new_user_token = user_tokens::insert(
        InsertableUserToken {
            user_id: new_user_inserted.user_id,
        },
        &conn,
    )
    .unwrap();
    return ApiResponse {
        json: json!(new_user_token),
        status: Status::Created,
    };
}

/// Generates a new token for the given user. Actual login checking is handled in the UserLogin request guard.
#[post("/Login", format = "json", data = "<user_login>")]
pub fn login(conn: CameraServerDbConn, user_login: Json<InsertableUser>) -> ApiResponse {
    if !is_login_valid(
        user_login.username.clone(),
        user_login.password.clone(),
        &conn,
    ) {
        return ApiResponse {
            json: json!({"error": "Incorrect username or password"}),
            status: Status::Unauthorized,
        };
    }

    let user = match get_by_username(user_login.username.clone(), &conn) {
        Ok(result) => result,
        Err(_) => {
            return ApiResponse {
                json: json!({"error": "Failed to get user id from username"}),
                status: Status::InternalServerError,
            }
        }
    };

    let token = match user_tokens::insert(
        user_tokens::InsertableUserToken {
            user_id: user.user_id,
        },
        &conn,
    ) {
        Ok(result) => result,
        Err(_) => {
            return ApiResponse {
                json: json!({"error": "Failed to create token"}),
                status: Status::InternalServerError,
            }
        }
    };

    return ApiResponse {
        json: json!(AuthentiationResult {
            user_info: UserInfo {
                user_id: user.user_id,
                username: user.username,
            },
            token: token,
        }),
        status: Status::Ok,
    };
}
