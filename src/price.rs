#[derive(Debug)]
pub struct PricePoint {
    pub price: u128,
    pub timestamp: u64,
    //pair_id: String
}

pub fn calculate_twap(data: &[PricePoint]) -> u128 {
    let mut total_weighted_price = 0;
    let mut total_time = 0;

    for i in 0..data.len() - 1 {
        let duration = data[i + 1].timestamp - data[i].timestamp;
        if duration > 0 {
            total_weighted_price += data[i].price * duration as u128;
            total_time += duration;
        }
    }

    total_weighted_price / total_time as u128
}
