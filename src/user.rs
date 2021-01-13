use crate::ApiResponse;

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

#[derive(Queryable, AsChangeset, Deserialize, Serialize)]
#[table_name = "users"]
pub struct User {
    pub id: uuid::Uuid,
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

#[post("/addUser", format = "json", data = "<new_user>")]
pub fn add_user(conn: CameraServerDbConn, new_user: Json<InsertableUser>) -> ApiResponse {
    let new_user_decoded = new_user.into_inner();

    if new_user_decoded.password.chars().count() < 8 {
        return ApiResponse {
            json: json!({"error": "Password must be at least 8 characters long"}),
            status: Status::UnprocessableEntity,
        };
    }

    let new_user_ready = InsertableUser {
        username: new_user_decoded.username,
        password: hash(new_user_decoded.password, DEFAULT_COST).unwrap(),
    };

    insert(new_user_ready, &conn).unwrap();
    return ApiResponse {
        json: json!({}),
        status: Status::Created,
    };
}
