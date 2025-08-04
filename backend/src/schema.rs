// @generated automatically by Diesel CLI.

diesel::table! {
    folders (id) {
        id -> Uuid,
        user_id -> Uuid,
        parent_folder_id -> Nullable<Uuid>,
        name -> Varchar,
    }
}

diesel::table! {
    passwords (id) {
        id -> Uuid,
        folder_id -> Nullable<Uuid>,
        website -> Varchar,
        username -> Varchar,
        encrypted_password -> Bytea,
        user_id -> Uuid,
        notes -> Nullable<Text>,
        otp_secret -> Nullable<Varchar>,
        attachments -> Nullable<Jsonb>,
    }
}

diesel::table! {
    shares (id) {
        id -> Uuid,
        password_id -> Nullable<Uuid>,
        folder_id -> Nullable<Uuid>,
        user_id -> Uuid,
        shared_with_user_id -> Uuid,
        permission_level -> Varchar,
        expires_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        password_hash -> Varchar,
        salt -> Varchar,
        mfa_secret -> Nullable<Varchar>,
    }
}

diesel::joinable!(folders -> users (user_id));
diesel::joinable!(passwords -> folders (folder_id));
diesel::joinable!(passwords -> users (user_id));
diesel::joinable!(shares -> folders (folder_id));
diesel::joinable!(shares -> passwords (password_id));

diesel::allow_tables_to_appear_in_same_query!(
    folders,
    passwords,
    shares,
    users,
);
