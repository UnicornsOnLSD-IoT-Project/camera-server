table! {
    camera_tokens (camera_token) {
        camera_token -> Uuid,
        camera_id -> Uuid,
    }
}

table! {
    cameras (camera_id) {
        camera_id -> Uuid,
        name -> Text,
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

allow_tables_to_appear_in_same_query!(
    camera_tokens,
    cameras,
    user_tokens,
    users,
    users_cameras,
);
