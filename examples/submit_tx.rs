use ogmios_client::{OgmiosClient, OgmiosLocalTxSubmission};

fn tx_bytes() -> Vec<u8> {
    // claim 110c88e91ea4812293949f31aee468998e52e97d6fcff64f54c1ab77badacd35 0
    let tx_hex = "84a50081825820110c88e91ea4812293949f31aee468998e52e97d6fcff64f54c1ab77badacd350\
    00181825839007cd00bc8d4c88791e7986aaec8fbea30e6a067cf75b243bec953a020f76a223b3ba0b009004dc082\
    7ea804d520f2023bba5d3684ad9a3453821a0095fc3fa0021a00029a410b5820b30210e09efbe6b148bd31b071050\
    90f361465cbbd9f8eece4afebb32d424f760d8182582042e36b9709741fcec9b88fc7e0d1af6ada1ec61934550ee5\
    58b77ceda24f10e600a40081825820ed6acd2c4d14a0c8f2991bb3f02219e2ced37c23490fc346d8c06dcb85899b8\
    55840ea23ea29c0a6385601482664a1ffbec3b64a7df8ce72c64ffd8ee52d9c7dc65a3479802688eeeffe6257cf60\
    8ede39f2e0c3aa26b0f7c8dff794af4d2fceb605049fd87980ff0581840000d87980821908fd1a0008f3a00681515\
    00100003222253330044a22930b2b9a01f5f6";

    hex::decode(tx_hex).unwrap()
}

#[tokio::main]
async fn main() {
    let ip = "192.168.0.143".to_string();
    let port = "1337".to_string();

    let client = OgmiosClient::new(ip, port);

    // let bytes = vec![1, 2, 3, 4];
    // let res = client.submit_tx(&bytes).await.unwrap();
    // println!("sad res: {:?}", res);

    let bytes = tx_bytes();
    let res = client.submit_tx(&bytes).await.unwrap();
    println!("happy res: {:?}", res);
}
