pub mod datas;
pub mod price;
use datas::get_last_block_number;
use price::{from_event, PricePoint};

use axum::{extract::State, routing::get, Json, Router};

use num_traits::FromPrimitive;
use serde_json::{json, Value};
use starknet::{
    core::types::Felt,
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Url,
    },
    signers::SigningKey,
};

//use starknet::macros::felt_dec;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{self, Duration};

//const STARKNET_URL: &str = "https://free-rpc.nethermind.io/mainnet-juno";
//const STARKNET_URL: &str = "https://free-rpc.nethermind.io/sepolia-juno";
const STARKNET_URL: &str = "https://api.cartridge.gg/x/starknet/sepolia";

#[derive(Clone)]
struct AppState {
    pub provider: JsonRpcClient<HttpTransport>,
    pub prices: Arc<RwLock<Vec<PricePoint>>>,
    pub key: SigningKey,
}

async fn get_twap(State(state): State<AppState>) -> Json<Value> {
    let events = datas::get_events(&state.provider).await;
    let prices: Vec<PricePoint> = from_event(&events);
    if !prices.is_empty() {
        let twap = price::calculate_twap(&prices);
        // sign price
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

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

async fn get_prices(State(state): State<AppState>) -> Json<Value> {
    let prices = state.prices.read().unwrap();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let last_hour = now - 3600;
    let last_hour_prices: Vec<PricePoint> = prices
        .iter()
        .filter(|p| p.timestamp > last_hour)
        .cloned()
        .collect();
    
    //println!("{:?}", last_hour_prices);

    let twap = price::calculate_twap(&last_hour_prices);

    Json(json!({
        "prices": json!(last_hour_prices),
        "twap": twap
    }))
}

async fn update_prices(state: AppState) {
    let mut interval = time::interval(Duration::from_secs(10));

    let mut current_block = 0;
    loop {
        interval.tick().await;

        let last_block = get_last_block_number(&state.provider).await;
        if last_block == current_block {
            continue;
        }
        current_block = last_block;

        let new_events = datas::get_events_from_block(&state.provider, last_block).await;
        let new_prices = from_event(&new_events);

        if new_prices.is_empty() {
            continue;
        }

        if let Ok(mut prices) = state.prices.write() {
            println!("LOG: new prices added: {:?}", new_prices);
            prices.extend(new_prices);
        }
    }
}


async fn get_init_prices(state: &AppState) {
    let events = datas::get_events(&state.provider).await;
    let init_prices: Vec<PricePoint> = from_event(&events);

    if let Ok(mut prices) = state.prices.write() {
        println!("LOG: init state with {:?} prices", init_prices.len());
        prices.extend(init_prices);
    }
}


#[tokio::main]
async fn main() {
    let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(STARKNET_URL).unwrap()));

    let state = AppState {
        provider,
        prices: Arc::new(RwLock::new(vec![])),
        key: SigningKey::from_random(),
    };
    get_init_prices(&state).await;

    tokio::spawn(update_prices(state.clone()));

    let app = Router::new()
        .route("/data", get(get_twap))
        .route("/health", get(health_check))
        .route("/prices", get(get_prices))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
