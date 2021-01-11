use super::schema::users;

pub mod handler;
pub mod repository;
pub mod router;

#[derive(Queryable, AsChangeset, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct InsertableUser {
    username: String,
    password: String,
}

pub impl InsertableUser {
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

pub fn get(id: i32, connection: &PgConnection) -> QueryResult<User> {
    users::table.find(id).get_result::<User>(connection)
}

pub fn insert(user: User, connection: &PgConnection) -> QueryResult<User> {
    diesel::insert_into(users::table)
        .values(&InsertableUser::from_user(user))
        .get_result(connection)
}

pub fn update(id: i32, user: User, connection: &PgConnection) -> QueryResult<User> {
    diesel::update(users::table.find(id))
        .set(&user)
        .get_result(connection)
}

pub fn delete(id: i32, connection: &PgConnection) -> QueryResult<usize> {
    diesel::delete(users::table.find(id)).execute(connection)
}
