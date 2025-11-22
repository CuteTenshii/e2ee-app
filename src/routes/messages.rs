use axum::{Extension, Json};
use diesel::prelude::*;
use serde_json::{Value, json};
use e2ee_back::{models::*, schema::messages::dsl::*};
use crate::AppState;

pub async fn get_messages(state: Extension<AppState>) -> Json<Value> {
    let mut conn = state.db.get().unwrap();
    let results = messages
        .limit(10)
        .load::<Message>(&mut conn)
        .expect("Failed to load messages");

    Json(json!(results))
}
