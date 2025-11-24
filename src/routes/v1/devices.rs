use crate::{AppState, AuthUser};
use axum::{Extension, Json};
use diesel::prelude::*;
use e2ee_back::schema::devices;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn get_devices(state: Extension<AppState>, auth: AuthUser) -> Json<Value> {
    let mut conn = state.db.get().unwrap();
    let results = devices::table
        .select((
            devices::id,
            devices::name,
        ))
        .filter(devices::user_id.eq(auth.user_id))
        .load::<(Uuid, String)>(&mut conn)
        .expect("Failed to load devices");

    Json(json!({
        "data": results,
    }))
}
