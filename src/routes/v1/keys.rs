use crate::routes::v1::register::Claims;
use crate::{AppState, AuthUserDevice};
use axum::http::StatusCode;
use axum::{Extension, Json};
use base64::Engine;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use e2ee_back::models::Device;
use e2ee_back::schema::{devices, one_time_prekeys};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct UploadKeysRequest {
    identity_key_pub: String,
    signed_prekey_pub: String,
    signed_prekey_signature: String,
    one_time_prekeys: Vec<String>,
    device_name: String,
    push_token: String,
}

pub async fn upload_keys(
    Extension(state): Extension<AppState>,
    auth: AuthUserDevice,
    Json(payload): Json<UploadKeysRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut conn = state.db.get().unwrap();

    let exists: Option<Device> = devices::table
        .filter(devices::id.eq(auth.device_id))
        .first::<Device>(&mut conn)
        .optional()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "message": "Something went wrong",
            "status": 500,
        }))))
        .expect("Failed to check if device already exists");

    if exists.is_some() {
        return (StatusCode::CONFLICT, Json(json!({
            "message": "Keys already uploaded",
            "status": 409,
        })));
    }

    let identity_key_bytes = base64::engine::general_purpose::STANDARD
        .decode(payload.identity_key_pub)
        .expect("Failed to decode base64");
    let signed_prekey_bytes = base64::engine::general_purpose::STANDARD
        .decode(payload.signed_prekey_pub)
        .expect("Failed to decode base64");
    let signed_prekey_sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(payload.signed_prekey_signature)
        .expect("Failed to decode base64");
    let one_time_prekeys_bytes: Vec<Vec<u8>> = payload
        .one_time_prekeys
        .iter()
        .map(|k| base64::engine::general_purpose::STANDARD.decode(k))
        .collect::<Result<_, _>>()
        .expect("Failed to decode base64");

    diesel::insert_into(devices::table)
        .values((
            devices::id.eq(auth.device_id),
            devices::user_id.eq(auth.user_id),
            devices::name.eq(payload.device_name),
            devices::identity_key_pub.eq(identity_key_bytes),
            devices::signed_prekey_pub.eq(signed_prekey_bytes),
            devices::signed_prekey_signature.eq(signed_prekey_sig_bytes),
            devices::created_at.eq(Utc::now()),
            devices::is_revoked.eq(false),
        ))
        .execute(&mut conn)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "message": "Something went wrong",
            "status": 500,
        }))))
        .expect("Failed to insert device");

    let prekeys_to_insert: Vec<_> = one_time_prekeys_bytes
        .into_iter()
        .map(|k| (
            one_time_prekeys::device_id.eq(auth.device_id),
            one_time_prekeys::prekey_pub.eq(k),
            one_time_prekeys::is_consumed.eq(false),
            one_time_prekeys::created_at.eq(Utc::now()),
        ))
        .collect();

    diesel::insert_into(one_time_prekeys::table)
        .values(&prekeys_to_insert)
        .execute(&mut conn)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "message": "Something went wrong",
            "status": 500,
        }))))
        .expect("Failed to insert prekeys");

    let expiry = Utc::now()
        .checked_add_signed(Duration::days(7))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: auth.user_id,
        device: auth.device_id,
        exp: expiry,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ).unwrap();

    (StatusCode::OK, Json(json!({
        "success": true,
        "device_id": auth.device_id,
        "auth_token": token,
    })))
}
