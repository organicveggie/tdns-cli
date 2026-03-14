use rstest::rstest;
use serde::Serialize;
use std::path::PathBuf;

use super::*;

#[rstest]
#[case(Some("1.2.3.4".to_string()), None, None, None, vec![("address", "1.2.3.4")])]
#[case(None, Some(true), None, None, vec![("ptr", "true")])]
#[case(None, None, Some(true), None, vec![("ptr_zone", "true")])]
#[case(None, None, None, Some(true), vec![("update_svcb_hints", "true")])]
#[case(Some("1.2.3.4".to_string()), Some(true), None, None, vec![("address", "1.2.3.4"), ("ptr", "true")])]
fn make_address_params_test_v2(
    #[case] addr: Option<String>,
    #[case] ptr: Option<bool>,
    #[case] ptr_zone: Option<bool>,
    #[case] update_svcb_hints: Option<bool>,
    #[case] expected_params: Vec<(&str, &str)>,
) {
    let expected = QueryBuilder::from(expected_params);
    assert_eq!(make_address_params(addr, ptr, ptr_zone, update_svcb_hints), expected);
}

#[derive(Debug, Deserialize, Serialize)]
struct QueryParamsExtraTestOptions {
    pub ttl: Option<u32>,
    pub comments: Option<String>,
    pub expiry_ttl: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct QueryParamsTestCase {
    domain: String,
    record_type_command: RecordTypeCommand,
    extra_options: Option<QueryParamsExtraTestOptions>,
    expected_params: QueryBuilder,
}

#[rstest]
fn make_query_params_test(
    #[files("src/zone/update/fixtures/make_query_params_*.toml")] path: PathBuf,
) {
    const TOKEN: &str = "token1";
    const ZONE: &str = "example.com";
    let cfg = config::Config::new("host1", TOKEN);

    let file_content = std::fs::read_to_string(path).unwrap();
    let test_case: QueryParamsTestCase = toml::from_str(&file_content).unwrap();
    let extras = match test_case.extra_options.as_ref() {
        Some(options) => options,
        None => &QueryParamsExtraTestOptions { ttl: None, comments: None, expiry_ttl: None },
    };

    let domain_name = helpers::make_domain_name(&test_case.domain, ZONE);
    let qb = test_case.record_type_command.make_query_params(
        &cfg,
        &domain_name,
        ZONE,
        extras.comments.clone(),
        extras.ttl,
        extras.expiry_ttl,
    );
    assert_eq!(qb, test_case.expected_params);
}
