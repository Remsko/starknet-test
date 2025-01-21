pub mod datas;
pub mod price;
use price::PricePoint;

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};

use num_traits::{FromPrimitive, ToPrimitive};
use serde_json::json;
use starknet::{core::types::Felt, signers::SigningKey};

use starknet::core::utils::parse_cairo_short_string;
//use starknet::macros::felt_dec;

#[derive(Clone)]
struct AppState {
    pub key: SigningKey,
}

async fn get_twap(State(state): State<AppState>) -> impl IntoResponse {
    let events = datas::get_events().await;
    let mut prices: Vec<PricePoint> = vec![];

    events.iter().for_each(|event| {
        if parse_cairo_short_string(&event.data[4]).unwrap() == "WBTC/USD"
        //if event.data[4] == felt_dec!("18669995996566340")
        {
            prices.push(PricePoint {
                price: event.data[3].to_u128().unwrap(),
                timestamp: event.data[0].to_u64().unwrap(),
            })
        }
    });
    if !prices.is_empty() {
        // sign price
        let twap = price::calculate_twap(&prices);
        let signature = state.key.sign(&Felt::from_u128(twap).unwrap()).unwrap();
        let signer = state.key.verifying_key().scalar().to_fixed_hex_string();
        //println!("{0:#?}", state.key.verifying_key().verify(&Felt::from_u128(twap).unwrap(), &signature).unwrap());

        return Json(json!({
            "twap": twap,
            "signer": signer,
            "signature": json!({
                "r": signature.r.to_fixed_hex_string(),
                "s": signature.s.to_fixed_hex_string()
            })
        }));
    }
    Json(json!({
        "twap": "ERROR"
    }))
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok"
    }))
}

#[tokio::main]
async fn main() {
    let state = AppState {
        key: SigningKey::from_random(),
    };

    let app = Router::new()
        .route("/data", get(get_twap))
        .route("/health", get(health_check))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
