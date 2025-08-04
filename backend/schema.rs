table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        password_hash -> Varchar,
        salt -> Varchar,
        mfa_secret -> Nullable<Varchar>,
    }
}

table! {
    folders (id) {
        id -> Uuid,
        user_id -> Uuid,
        parent_folder_id -> Nullable<Uuid>,
        name -> Varchar,
    }
}

table! {
    passwords (id) {
        id -> Uuid,
        folder_id -> Nullable<Uuid>,
        website -> Varchar,
        username -> Varchar,
        encrypted_password -> Bytea,
    }
}

table! {
    shares (id) {
        id -> Uuid,
        password_id -> Nullable<Uuid>,
        folder_id -> Nullable<Uuid>,
        user_id -> Uuid,
    }
}