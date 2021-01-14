use crate::{user_tokens, ApiResponse};

use super::schema::users;
use super::CameraServerDbConn;
use bcrypt::{hash, verify, DEFAULT_COST};
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

pub fn does_username_exist(username_to_search: String, connection: &PgConnection) -> bool {
    let result = users::table
        .filter(users::username.eq_all(username_to_search))
        .first::<User>(connection);

    match result {
        Ok(_) => return true,
        Err(_) => return false,
    }
}

#[post("/addUser", format = "json", data = "<new_user>")]
pub fn add_user(conn: CameraServerDbConn, new_user: Json<InsertableUser>) -> ApiResponse {
    let new_user_decoded = new_user.into_inner();

    if new_user_decoded.password.chars().count() < 8 {
        return ApiResponse {
            json: json!({"error": "Password must be at least 8 characters long"}),
            status: Status::UnprocessableEntity,
        };
    }

    if does_username_exist(new_user_decoded.username.clone(), &conn) {
        return ApiResponse {
            json: json!({"error": "Username already exists"}),
            status: Status::Conflict,
        };
    }

    let new_user_insertable = InsertableUser {
        username: new_user_decoded.username,
        password: hash(new_user_decoded.password, DEFAULT_COST).unwrap(),
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
