use crate::AppState;
use argon2::{Argon2, PasswordHasher};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use e2ee_back::models::{User, VerificationCode};
use e2ee_back::schema::users;
use e2ee_back::schema::verification_codes;
use jsonwebtoken::{encode, EncodingKey, Header};
use password_hash::{PasswordHash, PasswordVerifier, SaltString};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

fn normalize_phone(phone: &str) -> Option<String> {
    let p = phone.trim().replace(" ", "");

    if !p.starts_with('+') {
        return None;
    }
    if p.len() < 8 {
        return None;
    }
    Some(p)
}

#[derive(Deserialize)]
pub struct PhoneRegister {
    phone_number: String,
}

pub async fn register_phone(
    state: Extension<AppState>,
    Json(payload): Json<PhoneRegister>
) -> (StatusCode, Json<serde_json::Value>) {
    let phone = match normalize_phone(&payload.phone_number) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, Json(json!({
            "message": "Invalid phone number",
            "status": 400,
        }))),
    };

    let mut conn = state.db.get().unwrap();
    let existing = verification_codes::table
        .filter(verification_codes::phone_number.eq(&phone))
        .first::<VerificationCode>(&mut conn)
        .optional()
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "message": "Something went wrong",
            "status": 500,
        }))));

    if let Ok(Some(record)) = existing {
        let now = Utc::now();

        if now.signed_duration_since(record.expires_at - Duration::minutes(4)) < Duration::seconds(60) {
            return (StatusCode::TOO_MANY_REQUESTS, Json(json!({
                "message": "Please wait a bit for another code",
                "status": 429,
            })));
        }
    }

    let otp = rand::rng().random_range(100000..999999);
    #[cfg(debug_assertions)]
    print!("Generated OTP code: {otp}\n");

    let hasher = Argon2::default();
    let hashed = hasher
        .hash_password(otp.to_string().as_bytes(), &SaltString::generate())
        .unwrap()
        .to_string();
    let exp = Utc::now() + Duration::minutes(5);

    diesel::insert_into(verification_codes::table)
        .values((
            verification_codes::phone_number.eq(&phone),
            verification_codes::code_hash.eq(&hashed),
            verification_codes::expires_at.eq(exp),
            verification_codes::attempt_count.eq(0),
        ))
        .on_conflict(verification_codes::phone_number)
        .do_update()
        .set((
            verification_codes::code_hash.eq(&hashed),
            verification_codes::expires_at.eq(exp),
            verification_codes::attempt_count.eq(0),
        ))
        .execute(&mut conn)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "message": "Something went wrong",
            "status": 500,
        }))))
        .expect("Failed to insert verification code");

    (StatusCode::ACCEPTED, Json(json!({"success": true})))
}

#[derive(Deserialize)]
pub struct ConfirmRegister {
    phone_number: String,
    otp: String,
}

pub async fn register_confirm(state: Extension<AppState>, Json(payload): Json<ConfirmRegister>) -> (StatusCode, Json<serde_json::Value>) {
    let phone = match normalize_phone(&payload.phone_number) {
        Some(p) => p,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "message": "Invalid phone number",
                "status": 400,
            })))
        }
    };

    let mut conn = state.db.get().unwrap();
    let entry = match verification_codes::table
        .filter(verification_codes::phone_number.eq(&phone))
        .first::<VerificationCode>(&mut conn)
        .optional()
    {
        Ok(e) => e,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "message": "Something went wrong",
                "status": 500,
            })))
        }
    };

    let VerificationCode {
        phone_number: phone_db,
        code_hash: code_hash_db,
        expires_at: expires_at_db,
        attempt_count: attempt_count_db
    } = match entry {
        Some(e) => e,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({
            "message": "Unauthorized",
            "status": 401,
        }))),
    };

    if Utc::now() > expires_at_db {
        return (StatusCode::UNAUTHORIZED, Json(json!({
            "message": "Unauthorized",
            "status": 401,
        })));
    }

    let attempts = attempt_count_db.unwrap_or(0);
    if attempts >= 5 {
        return (StatusCode::FORBIDDEN, Json(json!({
            "message": "Your account has been blocked for security reasons, please retry later.",
            "status": 403,
        })));
    }

    let parsed = match PasswordHash::new(&code_hash_db) {
        Ok(p) => p,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "message": "Internal Server Error",
                "status": 500,
            })))
        }
    };

    if Argon2::default()
        .verify_password(payload.otp.as_bytes(), &parsed)
        .is_err() {
        diesel::update(verification_codes::table.filter(verification_codes::phone_number.eq(&phone)))
            // Incrémenter tentatives
            .set(verification_codes::attempt_count.eq(attempts + 1))
            .execute(&mut conn)
            .unwrap();

        return (StatusCode::UNAUTHORIZED, Json(json!({
            "message": "Unauthorized",
            "status": 401,
        })));
    }

    let existing_user = users::table
        .filter(users::phone_number.eq(&phone))
        .first::<User>(&mut conn)
        .optional()
        .unwrap();

    let user_id_val = match existing_user {
        Some(user) => user.id,
        None => {
            let new_user_id = Uuid::new_v4();
            diesel::insert_into(users::table)
                .values((
                    users::id.eq(new_user_id),
                    users::name.eq("New user"),
                    users::phone_number.eq(&phone),
                    users::created_at.eq(Utc::now()),
                ))
                .execute(&mut conn)
                .unwrap();

            new_user_id
        }
    };

    diesel::delete(verification_codes::table.filter(verification_codes::phone_number.eq(&phone)))
        .execute(&mut conn)
        .unwrap();

    let device_id_val = Uuid::new_v4();
    let expiry = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;
    let claims = Claims {
        sub: user_id_val,
        device: device_id_val,
        exp: expiry,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
        .expect("JWT generation failed");

    (StatusCode::OK, Json(json!({
        "success": true,
        "user_id": user_id_val,
        "device_id": device_id_val,
        "auth_token": token,
    })))
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user_id
    pub device: Uuid, // device_id
    pub exp: usize, // expiration UNIX timestamp
}
