//! Integration tests for usecases_io_operations
//!
//! Tests the integration between helper functions.

use usecases_io_operations::*;

#[test]
fn test_environment_merging() {
    // Test environment variable merging
    let mut env = Environment::new();
    let global_env: Environment = [
        ("PATH".to_string(), "/usr/bin".to_string()),
        ("HOME".to_string(), "/home/user".to_string()),
    ]
    .iter()
    .cloned()
    .collect();

    let key_value_pairs = vec![
        ("PATH".to_string(), Some("/custom/path".to_string())),
        ("HOME".to_string(), None), // Unset
        ("NEW_VAR".to_string(), Some("new_value".to_string())),
    ];

    merge_global_environment(&mut env, &global_env, &key_value_pairs).unwrap();

    assert_eq!(env.get("PATH"), Some(&"/custom/path".to_string()));
    assert_eq!(env.get("HOME"), None); // Should be unset
    assert_eq!(env.get("NEW_VAR"), Some(&"new_value".to_string()));
}

#[test]
fn test_argument_conversion() {
    // Test argument list conversion
    let args = vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()];
    let result = convert_args(&args).unwrap();
    
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], "default");
    assert_eq!(result[1], "arg1");
    assert_eq!(result[2], "arg2");
    assert_eq!(result[3], "arg3");
}


#[test]
fn test_http_uri_building() {
    // Test HTTP URI building
    let pca = PacketCallbackArgs {
        process_id: 1,
        result: None,
        string_as_bin: false,
        aligned_ptr: std::ptr::null(),
        original: Vec::new(),
        bin_size: 0,
    };

    // Test star URI
    let uri = HttpUri::Star;
    let result = http_bld_uri(&pca, &uri).unwrap();
    assert_eq!(result, HttpUri::Star);

    // Test absolute path
    let uri = HttpUri::AbsPath("/path/to/resource".to_string());
    let result = http_bld_uri(&pca, &uri).unwrap();
    match result {
        HttpUri::AbsPath(path) => assert_eq!(path, "/path/to/resource"),
        _ => panic!("Expected AbsPath"),
    }

    // Test absolute URI
    let uri = HttpUri::AbsoluteUri {
        scheme: "http".to_string(),
        host: "example.com".to_string(),
        port: Some(80),
        path: "/index.html".to_string(),
    };
    let result = http_bld_uri(&pca, &uri).unwrap();
    match result {
        HttpUri::AbsoluteUri {
            scheme,
            host,
            port,
            path,
        } => {
            assert_eq!(scheme, "http");
            assert_eq!(host, "example.com");
            assert_eq!(port, Some(80));
            assert_eq!(path, "/index.html");
        }
        _ => panic!("Expected AbsoluteUri"),
    }
}


