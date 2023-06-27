use ogmios_client::{OgmiosClient, OgmiosLocalTxSubmission};

fn tx_bytes() -> Vec<u8> {
    // claim 110c88e91ea4812293949f31aee468998e52e97d6fcff64f54c1ab77badacd35 0
    let tx_hex = "84a50081825820110c88e91ea4812293949f31aee468998e52e97d6fcff64f54c1ab77badacd3500\
    0181825839007cd00bc8d4c88791e7986aaec8fbea30e6a067cf75b243bec953a020f76a223b3ba0b009004dc0827e\
    a804d520f2023bba5d3684ad9a3453821a0095fb37a0021a00029b490b582000000000000000000000000000000000\
    000000000000000000000000000000000d8182582096a865201466a840d1e0717f53b4734c4481013252920c37049c\
    9135c9e2bb1a00a40081825820000000007cd00bc8d4c88791e7986aaec8fbea30e6a067cf75b243bec953a0205840\
    00f899d39b17fd5d66c192c4b50d343e42f7235b30504c8ae7619f93c828dc6dce4568dd69177c551828492d777a67\
    27fd66c2fbccbda8c2aeed92032c99790a049fd87980ff0581840000d87980821b00000001000000001b0000000100\
    000000068151500100003222253330044a22930b2b9a01f5f6";

    hex::decode(tx_hex).unwrap()
}

#[tokio::main]
async fn main() {
    let ip = "192.168.0.143".to_string();
    let port = "1337".to_string();

    let client = OgmiosClient::new(ip, port);
    let bytes = tx_bytes();
    let res = client.evaluate_tx(&bytes, vec![]).await.unwrap();
    println!("happy res: {:?}", res);
    let bytes = vec![1, 2, 3, 4];
    let res = client.evaluate_tx(&bytes, vec![]).await.unwrap();
    println!("sad res: {:?}", res);
}
