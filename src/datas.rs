use core::panic;
use starknet::{
    core::types::{
        BlockId, BlockTag, EmittedEvent, EventFilter, Felt, MaybePendingBlockWithTxHashes,
    },
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider,
    },
};

use crate::price::PricePoint;

//const PRAGMA_MAIN_ADDRESS: &str = "0x2a85bd616f912537c50a49a4076db02c00b29b2cdc8a197ce92ed1837fa875b";
const PRAGMA_TEST_CONTRACT_ADDRESS: &str =
    "0x36031daa264c24520b11d93af622c848b2499b66b41d611bac95e13cfca131a";
const SUBMITTED_SPOT_ENTRY: &str =
    "0x0280bb2099800026f90c334a3a23888ffe718a2920ffbbf4f44c6d3d5efb613c";

fn pragma_filter() -> EventFilter {
    let pragma_address = Felt::from_hex(PRAGMA_TEST_CONTRACT_ADDRESS).unwrap();
    let event_keys = vec![vec![Felt::from_hex(SUBMITTED_SPOT_ENTRY).unwrap()]];

    EventFilter {
        from_block: None,
        to_block: None,
        address: Some(pragma_address),
        keys: Some(event_keys),
    }
}

pub async fn get_last_block(provider: &JsonRpcClient<HttpTransport>) -> u64 {
    let block = provider
        .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest))
        .await;

    match block.unwrap() {
        MaybePendingBlockWithTxHashes::Block(b) => b.block_number,
        _ => panic!(),
    }
}

pub async fn get_last_block_number(provider: &JsonRpcClient<HttpTransport>) -> u64 {
    provider.block_number().await.unwrap()
}

pub async fn get_events(provider: &JsonRpcClient<HttpTransport>) -> Vec<EmittedEvent> {
    let mut filter: EventFilter = pragma_filter();
    let block_number = get_last_block_number(provider).await;
    filter.from_block = Some(BlockId::Number(block_number - 130));
    filter.to_block = Some(BlockId::Number(block_number));

    let page = provider.get_events(filter.clone(), None, 999).await;
    // loop maybe
    // let page = provider.get_events(filter, page.unwrap().continuation_token, 1).await;
    page.unwrap().events
}

pub async fn get_last_block_prices() -> Vec<PricePoint> {

    vec![]
}

pub async fn get_events_from_block(provider: &JsonRpcClient<HttpTransport>, block: u64) -> Vec<EmittedEvent> {
    let mut filter: EventFilter = pragma_filter();
    filter.from_block = Some(BlockId::Number(block));
    //filter.to_block = Some(BlockId::Number(block));

    let mut page = provider.get_events(filter.clone(), None, 64).await.unwrap();
    let mut events = page.events;
    while page.continuation_token.is_some() {
        page = provider.get_events(filter.clone(), page.continuation_token, 64).await.unwrap();
        events.extend(page.events);
    }

    events
}