use num_traits::ToPrimitive;
use serde::Serialize;
use starknet::core::{types::EmittedEvent, utils::parse_cairo_short_string};

#[derive(Debug, Clone, Serialize)]
pub struct PricePoint {
    pub price: u128,
    pub timestamp: u64,
    //pair_id: String
}

pub fn from_event(events: &[EmittedEvent]) -> Vec<PricePoint> {
    let mut prices = vec![];

    events.iter().for_each(|event| {
        let pair_id = parse_cairo_short_string(&event.data[4]).unwrap();

        if pair_id == "WBTC/USD"
        //if event.data[4] == felt_dec!("18669995996566340")
        {
            let timestamp = event.data[0].to_u64().unwrap();
            let price = event.data[3].to_u128().unwrap();
            assert!(timestamp > 0 && price > 0, "Invalid price or timestamp");
            prices.push(PricePoint { price, timestamp })
        }
    });

    prices.sort_by_key(|p| p.timestamp);
    prices
}

pub fn calculate_twap(data: &[PricePoint]) -> u128 {
    if data.is_empty() {
        return 0;
    }

    if data.len() == 1 {
        return data[0].price;
    }

    let mut total_weighted_price = 0;
    let mut total_time = 0;
    for i in 0..data.len() - 1 {
        let duration = data[i + 1].timestamp - data[i].timestamp;
        if duration > 0 {
            total_weighted_price += data[i].price * duration as u128;
            total_time += duration;
        }
    }

    assert!(total_time > 0, "Invalid timestamps");
    total_weighted_price / total_time as u128
}
