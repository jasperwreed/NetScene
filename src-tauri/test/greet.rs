#[test]
fn greet_returns_expected_string() {
    let result = netscene_lib::greet("tester");
    assert!(result.contains("tester"));
}
