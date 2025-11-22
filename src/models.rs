use diesel::prelude::*;
use chrono::NaiveDateTime;
use uuid::Uuid;
use serde::Serialize;
use crate::schema::{devices, messages, one_time_prekeys, users, verification_codes};

#[derive(Debug, Queryable, Identifiable, Associations)]
#[diesel(table_name = devices)]
#[diesel(belongs_to(User))]
pub struct Device {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub last_seen: Option<NaiveDateTime>,
    pub is_revoked: Option<bool>,
    pub identity_key_pub: Vec<u8>,
    pub signed_prekey_pub: Vec<u8>,
    pub signed_prekey_signature: Vec<u8>,
    pub push_token: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = devices)]
pub struct NewDevice<'a> {
    pub user_id: Option<Uuid>,
    pub name: Option<&'a str>,
    pub identity_key_pub: &'a [u8],
    pub signed_prekey_pub: &'a [u8],
    pub signed_prekey_signature: &'a [u8],
    pub push_token: Option<&'a str>,
}

#[derive(Debug, Queryable, Identifiable, Associations, Serialize)]
#[diesel(table_name = messages)]
#[diesel(belongs_to(User, foreign_key = sender_user_id))]
#[diesel(belongs_to(Device, foreign_key = sender_device_id))]
pub struct Message {
    pub id: i64,
    pub sender_user_id: Option<Uuid>,
    pub sender_device_id: Option<Uuid>,
    pub recipient_user_id: Option<Uuid>,
    pub recipient_device_id: Option<Uuid>,
    pub ciphertext: Vec<u8>,
    pub message_type: i16,
    pub protocol_version: i16,
    pub delivered_at: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = messages)]
pub struct NewMessage<'a> {
    pub sender_user_id: Option<Uuid>,
    pub sender_device_id: Option<Uuid>,
    pub recipient_user_id: Option<Uuid>,
    pub recipient_device_id: Option<Uuid>,
    pub ciphertext: &'a [u8],
    pub message_type: i16,
    pub protocol_version: i16,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[diesel(table_name = one_time_prekeys)]
#[diesel(belongs_to(Device))]
pub struct OneTimePrekey {
    pub id: i64,
    pub device_id: Option<Uuid>,
    pub prekey_pub: Vec<u8>,
    pub is_consumed: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = one_time_prekeys)]
pub struct NewOneTimePrekey<'a> {
    pub device_id: Option<Uuid>,
    pub prekey_pub: &'a [u8],
}

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub phone_number: String,
    pub avatar_hash: Option<String>,
    pub last_seen: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub phone_number: &'a str,
    pub avatar_hash: Option<&'a str>,
}

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = verification_codes)]
#[diesel(primary_key(phone_number))]
pub struct VerificationCode {
    pub phone_number: String,
    pub code_hash: String,
    pub expires_at: NaiveDateTime,
    pub attempt_count: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = verification_codes)]
pub struct NewVerificationCode<'a> {
    pub phone_number: &'a str,
    pub code_hash: &'a str,
    pub expires_at: NaiveDateTime,
    pub attempt_count: Option<i32>,
}
