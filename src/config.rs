use super::schema::configs;
use diesel::prelude::*;
use diesel::{self};
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
