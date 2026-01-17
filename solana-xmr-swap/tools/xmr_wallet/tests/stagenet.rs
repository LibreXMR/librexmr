use xmr_wallet::XmrWallet;

#[tokio::test]
#[ignore = "requires monero-wallet-rpc on stagenet"]
async fn connect_and_get_balance() {
    let wallet = XmrWallet::connect("http://127.0.0.1:18083")
        .await
        .expect("connect wallet RPC");
    let balance = wallet.get_balance().await.expect("get balance");
    println!("Stagenet balance: {}", balance);
}
