// @generated automatically by Diesel CLI.

diesel::table! {
    audit_logs (id) {
        id -> Uuid,
        event_type -> Varchar,
        user_id -> Nullable<Uuid>,
        resource_id -> Nullable<Uuid>,
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        details -> Nullable<Text>,
        timestamp -> Timestamp,
        integrity_hash -> Varchar,
    }
}

diesel::table! {
    folders (id) {
        id -> Uuid,
        user_id -> Uuid,
        parent_folder_id -> Nullable<Uuid>,
        name -> Varchar,
    }
}

diesel::table! {
    login_history (id) {
        id -> Uuid,
        user_id -> Uuid,
        ip_address -> Inet,
        #[max_length = 100]
        country -> Nullable<Varchar>,
        #[max_length = 100]
        region -> Nullable<Varchar>,
        #[max_length = 100]
        city -> Nullable<Varchar>,
        latitude -> Nullable<Numeric>,
        longitude -> Nullable<Numeric>,
        #[max_length = 50]
        timezone -> Nullable<Varchar>,
        #[max_length = 255]
        isp -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        login_time -> Timestamptz,
        is_suspicious -> Bool,
        created_at -> Timestamptz,
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
        encrypted_website -> Nullable<Bytea>,
        encrypted_username -> Nullable<Bytea>,
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
        reset_token -> Nullable<Varchar>,
        reset_token_expires_at -> Nullable<Timestamp>,
        email -> Varchar,
        auth_method -> Nullable<Varchar>,
        is_sso_user -> Nullable<Bool>,
        sso_display_name -> Nullable<Varchar>,
        sso_avatar_url -> Nullable<Varchar>,
    }
}

diesel::joinable!(audit_logs -> users (user_id));
diesel::joinable!(folders -> users (user_id));
diesel::joinable!(login_history -> users (user_id));
diesel::joinable!(passwords -> folders (folder_id));
diesel::joinable!(passwords -> users (user_id));
diesel::joinable!(shares -> folders (folder_id));
diesel::joinable!(shares -> passwords (password_id));

diesel::table! {
    oauth_accounts (id) {
        id -> Uuid,
        user_id -> Uuid,
        provider -> Varchar,
        provider_user_id -> Varchar,
        email -> Varchar,
        access_token_hash -> Nullable<Varchar>,
        refresh_token_hash -> Nullable<Varchar>,
        token_expires_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(oauth_accounts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    audit_logs,
    folders,
    login_history,
    oauth_accounts,
    passwords,
    shares,
    users,
);
