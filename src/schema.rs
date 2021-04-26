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
    configs (camera_id) {
        camera_id -> Uuid,
        interval -> Int2,
    }
}

table! {
    user_tokens (user_token) {
        user_token -> Uuid,
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
    configs,
    user_tokens,
    users,
    users_cameras,
);
