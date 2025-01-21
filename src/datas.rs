use core::panic;
use starknet::{
    core::types::{
        BlockId, BlockTag, EmittedEvent, EventFilter, Felt, MaybePendingBlockWithTxHashes,
    },
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider, Url,
    },
};

pub async fn get_events() -> Vec<EmittedEvent> {
    let provider = JsonRpcClient::new(HttpTransport::new(
        //Url::parse("https://free-rpc.nethermind.io/mainnet-juno").unwrap(),
        Url::parse("https://free-rpc.nethermind.io/sepolia-juno").unwrap(),
    ));

    let block = provider
        .get_block_with_tx_hashes(BlockId::Tag(BlockTag::Latest))
        .await;

    let block_number = match block.unwrap() {
        MaybePendingBlockWithTxHashes::Block(b) => b.block_number,
        _ => panic!(),
    };

    let filter = EventFilter {
        from_block: Some(BlockId::Number(block_number - 120)),
        to_block: Some(BlockId::Number(block_number)),
        //address: Some(Felt::from_hex("0x36031daa264c24520b11d93af622c848b2499b66b41d611bac95e13cfca131a").unwrap()),
        address: Some(
            Felt::from_hex("0x36031daa264c24520b11d93af622c848b2499b66b41d611bac95e13cfca131a")
                .unwrap(),
        ),
        keys: Some(vec![vec![Felt::from_hex(
            "0x0280bb2099800026f90c334a3a23888ffe718a2920ffbbf4f44c6d3d5efb613c",
        )
        .unwrap()]]),
    };

    let page = provider.get_events(filter.clone(), None, 9999).await;
    // loop maybe
    // let page = provider.get_events(filter, page.unwrap().continuation_token, 1).await;
    page.unwrap().events
}
