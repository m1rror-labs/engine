use mockchain_engine::rpc::get_version::get_version;

#[test]
fn test_get_version() {
    let res = get_version();
    assert_eq!(
        res,
        Ok(serde_json::json!({ "feature-set": 2891131721u32, "solana-core": "2.1.11" }))
    );
}
