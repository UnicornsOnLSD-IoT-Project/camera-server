table! {
    cameras (camera_id) {
        camera_id -> Uuid,
    }
}

table! {
    user_tokens (token) {
        token -> Uuid,
        user_id -> Uuid,
    }
}

table! {
    users (user_id) {
        user_id -> Uuid,
        username -> Text,
        password -> Text,
    }
}

table! {
    users_cameras (users_cameras_id) {
        users_cameras_id -> Int4,
        camera_id -> Uuid,
        user_id -> Uuid,
    }
}

joinable!(users_cameras -> cameras (camera_id));

allow_tables_to_appear_in_same_query!(
    cameras,
    user_tokens,
    users,
    users_cameras,
);
