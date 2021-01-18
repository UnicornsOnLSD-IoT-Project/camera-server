use super::schema::cameras;
use diesel::prelude::*;
use diesel::{self};
use serde::{Deserialize, Serialize};

#[derive(Queryable, AsChangeset, Deserialize, Serialize)]
#[table_name = "cameras"]
pub struct Camera {
    pub camera_id: uuid::Uuid,
}

pub fn all(connection: &PgConnection) -> QueryResult<Vec<Camera>> {
    cameras::table.load::<Camera>(&*connection)
}

pub fn get(id: uuid::Uuid, connection: &PgConnection) -> QueryResult<Camera> {
    cameras::table.find(id).get_result::<Camera>(connection)
}

pub fn insert(connection: &PgConnection) -> QueryResult<Camera> {
    diesel::insert_into(cameras::table)
        .default_values()
        .get_result(connection)
}

pub fn update(id: uuid::Uuid, camera: Camera, connection: &PgConnection) -> QueryResult<Camera> {
    diesel::update(cameras::table.find(id))
        .set(&camera)
        .get_result(connection)
}

pub fn delete(id: uuid::Uuid, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(cameras::table.find(id)).execute(connection)
}
