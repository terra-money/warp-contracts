use json_codec_wasm::Json as CodecJson;
use controller::condition::Json;

use std::str::FromStr;

#[test]
fn test_json_display_and_from_str() {
    let test_cases = vec![
        ("null", CodecJson::Null),
        ("true", CodecJson::Bool(true)),
        ("false", CodecJson::Bool(false)),
        ("123", CodecJson::I128(123)),
        ("\"hello\"", CodecJson::String("hello".to_string())),
        (
            "[\"hello\",123]",
            CodecJson::Array(vec![
                CodecJson::String("hello".to_string()),
                CodecJson::I128(123),
            ]),
        ),
        (
            "{\"key1\":\"value1\",\"key2\":123}",
            CodecJson::Object({
                let mut map = std::collections::HashMap::new();
                map.insert("key1".to_string(), CodecJson::String("value1".to_string()));
                map.insert("key2".to_string(), CodecJson::I128(123));
                map
            }),
        ),
        (
            "{\"key1\":[\"value1\",123],\"key2\":{\"key3\":true}}",
            CodecJson::Object({
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "key1".to_string(),
                    CodecJson::Array(vec![
                        CodecJson::String("value1".to_string()),
                        CodecJson::I128(123),
                    ]),
                );
                map.insert(
                    "key2".to_string(),
                    CodecJson::Object({
                        let mut inner_map = std::collections::HashMap::new();
                        inner_map.insert("key3".to_string(), CodecJson::Bool(true));
                        inner_map
                    }),
                );
                map
            }),
        ),
    ];

    for (json_str, expected_json) in test_cases {
        // Test FromStr
        let parsed_json = Json::from_str(json_str).unwrap();
        assert_eq!(parsed_json.value, expected_json);

        // Test Display
        let displayed_json = format!(
            "{}",
            Json {
                value: expected_json.clone()
            }
        );
        let reparsed_displayed_json = Json::from_str(&displayed_json).unwrap();
        assert_eq!(reparsed_displayed_json.value, expected_json);
    }
}
