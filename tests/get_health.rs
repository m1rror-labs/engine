use mockchain_engine::rpc::get_health::get_health;

#[test]
fn test_get_health() {
    let res = get_health();
    assert_eq!(res, Ok(serde_json::json!("ok")));
}
