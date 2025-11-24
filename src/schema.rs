// @generated automatically by Diesel CLI.

diesel::table! {
    devices (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        name -> Text,
        created_at -> Nullable<Timestamptz>,
        last_seen -> Nullable<Timestamptz>,
        is_revoked -> Nullable<Bool>,
        identity_key_pub -> Bytea,
        signed_prekey_pub -> Bytea,
        signed_prekey_signature -> Bytea,
        push_token -> Nullable<Text>,
    }
}

diesel::table! {
    messages (id) {
        id -> Int8,
        sender_user_id -> Nullable<Uuid>,
        sender_device_id -> Nullable<Uuid>,
        recipient_user_id -> Nullable<Uuid>,
        recipient_device_id -> Nullable<Uuid>,
        ciphertext -> Bytea,
        message_type -> Int2,
        protocol_version -> Int2,
        delivered_at -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    one_time_prekeys (id) {
        id -> Int8,
        device_id -> Nullable<Uuid>,
        prekey_pub -> Bytea,
        is_consumed -> Nullable<Bool>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        name -> Text,
        #[max_length = 100]
        phone_number -> Varchar,
        #[max_length = 64]
        avatar_hash -> Nullable<Varchar>,
        last_seen -> Nullable<Timestamptz>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    verification_codes (phone_number) {
        phone_number -> Text,
        code_hash -> Text,
        expires_at -> Timestamptz,
        attempt_count -> Nullable<Int4>,
    }
}

diesel::joinable!(devices -> users (user_id));
diesel::joinable!(one_time_prekeys -> devices (device_id));

diesel::allow_tables_to_appear_in_same_query!(
    devices,
    messages,
    one_time_prekeys,
    users,
    verification_codes,
);
