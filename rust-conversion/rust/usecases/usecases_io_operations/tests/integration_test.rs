//! Integration tests for usecases_io_operations
//!
//! Tests the integration between port BIFs, port control, and helper functions.

use usecases_io_operations::*;

#[test]
fn test_port_bif_and_control_integration() {
    // Test that port BIF operations can work with control driver
    let driver = ControlDriver::new();
    let port_id = 123;
    
    // Start the driver
    let result = driver.start(port_id);
    assert!(result.is_ok());
    
    // Control operations should work
    let (response, size) = driver.control(port_id, 'e' as u32, b"test", 10).unwrap();
    assert_eq!(size, 4);
    assert_eq!(response, b"test");
    
    // Stop the driver
    driver.stop(port_id);
}

#[test]
fn test_port_settings_parsing() {
    // Test port settings creation and defaults
    let mut settings = PortSettings::default();
    assert_eq!(settings.packet_bytes, 0);
    assert_eq!(settings.use_stdio, true);
    
    // Modify settings
    settings.packet_bytes = 4;
    settings.binary_io = true;
    settings.read = true;
    settings.write = true;
    
    assert_eq!(settings.packet_bytes, 4);
    assert!(settings.binary_io);
    assert!(settings.read);
    assert!(settings.write);
}

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
fn test_packet_options() {
    // Test packet options creation
    let mut options = PacketOptions::default();
    assert_eq!(options.max_packet_length, None);
    assert_eq!(options.line_length, None);
    assert_eq!(options.line_delimiter, None);
    
    // Set options
    options.max_packet_length = Some(1024);
    options.line_length = Some(512);
    options.line_delimiter = Some(b'\n');
    
    assert_eq!(options.max_packet_length, Some(1024));
    assert_eq!(options.line_length, Some(512));
    assert_eq!(options.line_delimiter, Some(b'\n'));
}

#[test]
fn test_port_identifier() {
    // Test port identifier creation
    let id = PortIdentifier::Id(12345);
    let name = PortIdentifier::Name("test_port".to_string());
    
    match id {
        PortIdentifier::Id(n) => assert_eq!(n, 12345),
        _ => panic!("Expected Id"),
    }
    
    match name {
        PortIdentifier::Name(n) => assert_eq!(n, "test_port"),
        _ => panic!("Expected Name"),
    }
}

#[test]
fn test_port_name_variants() {
    // Test all port name variants
    let spawn = PortName::Spawn {
        command: "ls".to_string(),
    };
    let spawn_driver = PortName::SpawnDriver {
        driver: "test_driver".to_string(),
    };
    let spawn_exec = PortName::SpawnExecutable {
        executable: "/usr/bin/test".to_string(),
    };
    let fd = PortName::Fd {
        input_fd: 0,
        output_fd: 1,
    };
    
    match spawn {
        PortName::Spawn { command } => assert_eq!(command, "ls"),
        _ => panic!("Expected Spawn"),
    }
    
    match spawn_driver {
        PortName::SpawnDriver { driver } => assert_eq!(driver, "test_driver"),
        _ => panic!("Expected SpawnDriver"),
    }
    
    match spawn_exec {
        PortName::SpawnExecutable { executable } => assert_eq!(executable, "/usr/bin/test"),
        _ => panic!("Expected SpawnExecutable"),
    }
    
    match fd {
        PortName::Fd { input_fd, output_fd } => {
            assert_eq!(input_fd, 0);
            assert_eq!(output_fd, 1);
        }
        _ => panic!("Expected Fd"),
    }
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

#[test]
fn test_packet_type_variants() {
    // Test all packet type variants
    let types = vec![
        PacketType::Raw,
        PacketType::One,
        PacketType::Two,
        PacketType::Four,
        PacketType::Asn1,
        PacketType::SunRm,
        PacketType::Cdr,
        PacketType::Fcgi,
        PacketType::Line,
        PacketType::Tpkt,
        PacketType::Http,
        PacketType::HttpH,
        PacketType::HttpBin,
        PacketType::HttpHBin,
        PacketType::SslTls,
    ];
    
    // Just verify they can be created and compared
    assert_eq!(types.len(), 15);
    assert_eq!(types[0], PacketType::Raw);
    assert_eq!(types[14], PacketType::SslTls);
}

#[test]
fn test_port_info_items() {
    // Test all port info item variants
    let items = vec![
        PortInfoItem::Id,
        PortInfoItem::Name,
        PortInfoItem::Connected,
        PortInfoItem::Links,
        PortInfoItem::Input,
        PortInfoItem::Output,
        PortInfoItem::QueueSize,
        PortInfoItem::QueueData,
        PortInfoItem::ExitStatus,
    ];
    
    assert_eq!(items.len(), 9);
}

#[test]
fn test_port_data_variants() {
    // Test port data variants
    let immediate = PortData::Immediate(42);
    let heap = PortData::Heap(vec![1, 2, 3, 4]);
    let undefined = PortData::Undefined;
    
    match immediate {
        PortData::Immediate(val) => assert_eq!(val, 42),
        _ => panic!("Expected Immediate"),
    }
    
    match heap {
        PortData::Heap(data) => assert_eq!(data, vec![1, 2, 3, 4]),
        _ => panic!("Expected Heap"),
    }
    
    assert_eq!(undefined, PortData::Undefined);
}

