table! {
    user_tokens (token) {
        token -> Uuid,
        user_id -> Uuid,
    }
}

table! {
    users (user_id) {
        user_id -> Uuid,
        username -> Varchar,
        password -> Varchar,
    }
}

joinable!(user_tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(
    user_tokens,
    users,
);
