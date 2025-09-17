use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Formatter};

use pretty_assertions::assert_eq;
use serde::Deserialize;

/// Test helper macro for roundtrip testing with actual Debug output
macro_rules! test_roundtrip {
    ($test_name:ident, $value:expr, $expected_type:ty) => {
        #[test]
        fn $test_name() {
            let original = $value;
            let debug_output = format!("{:?}", original);
            eprintln!("Debug output: {}", debug_output);

            let deserialized: $expected_type = serde_dbgfmt::from_str(&debug_output)
                .unwrap_or_else(|e| panic!("Failed to deserialize: {}", e));

            assert_eq!(original, deserialized);
        }
    };
}

/// Test helper for debug implementations that use std::fmt helpers
macro_rules! test_debug_helper {
    ($test_name:ident, $debug_impl:expr, $expected_type:ty, $expected_value:expr) => {
        #[test]
        fn $test_name() {
            let debug_output = $debug_impl;
            eprintln!("Debug output: {}", debug_output);

            let deserialized: $expected_type = serde_dbgfmt::from_str(&debug_output)
                .unwrap_or_else(|e| panic!("Failed to deserialize: {}", e));

            assert_eq!($expected_value, deserialized);
        }
    };
}

// ==========================================
// Standard Library Types
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
}

test_roundtrip!(
    test_person_complete,
    Person {
        name: "Alice Johnson".to_string(),
        age: 42,
        email: Some("alice@example.com".to_string()),
    },
    Person
);

test_roundtrip!(
    test_person_no_email,
    Person {
        name: "Bob Smith".to_string(),
        age: 25,
        email: None,
    },
    Person
);

#[derive(Debug, Deserialize, PartialEq)]
struct SimpleConfig {
    host: String,
    port: u16,
    enabled: bool,
}

test_roundtrip!(
    test_simple_config,
    SimpleConfig {
        host: "localhost".to_string(),
        port: 8080,
        enabled: true,
    },
    SimpleConfig
);

// ==========================================
// Collections with Real Data
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
struct ConfigData {
    settings: BTreeMap<String, String>,
    tags: BTreeSet<String>,
    values: Vec<i32>,
}

test_roundtrip!(
    test_config_data,
    ConfigData {
        settings: {
            let mut map = BTreeMap::new();
            map.insert(
                "database_url".to_string(),
                "postgresql://localhost:5432/mydb".to_string(),
            );
            map.insert("log_level".to_string(), "info".to_string());
            map.insert("max_connections".to_string(), "100".to_string());
            map.insert("enable_ssl".to_string(), "true".to_string());
            map
        },
        tags: {
            let mut set = BTreeSet::new();
            set.insert("production".to_string());
            set.insert("web-server".to_string());
            set.insert("high-availability".to_string());
            set
        },
        values: vec![-100, 42, 999, i32::MAX, i32::MIN],
    },
    ConfigData
);

// ==========================================
// Complex Nested Structures
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
enum Status {
    Active,
    Inactive { reason: String },
    Pending { since: u64, priority: u8 },
}

#[derive(Debug, Deserialize, PartialEq)]
struct UserAccount {
    id: u64,
    username: String,
    status: Status,
    metadata: BTreeMap<String, Vec<String>>,
    scores: Option<Vec<f64>>,
}

test_roundtrip!(
    test_user_account_active,
    UserAccount {
        id: 12345,
        username: "power_user_2023".to_string(),
        status: Status::Active,
        metadata: {
            let mut map = BTreeMap::new();
            map.insert(
                "roles".to_string(),
                vec!["admin".to_string(), "moderator".to_string()],
            );
            map.insert(
                "groups".to_string(),
                vec!["developers".to_string(), "beta-testers".to_string()],
            );
            map
        },
        scores: Some(vec![95.5, 87.2, 99.1, 78.8]),
    },
    UserAccount
);

test_roundtrip!(
    test_user_account_inactive,
    UserAccount {
        id: 67890,
        username: "temp_user".to_string(),
        status: Status::Inactive {
            reason: "Account suspended for policy violation".to_string()
        },
        metadata: BTreeMap::new(),
        scores: None,
    },
    UserAccount
);

test_roundtrip!(
    test_user_account_pending,
    UserAccount {
        id: 11111,
        username: "new_user_2024".to_string(),
        status: Status::Pending {
            since: 1704067200,
            priority: 5
        },
        metadata: {
            let mut map = BTreeMap::new();
            map.insert(
                "preferences".to_string(),
                vec!["dark-mode".to_string(), "notifications-off".to_string()],
            );
            map
        },
        scores: Some(vec![]),
    },
    UserAccount
);

// ==========================================
// Non-exhaustive Structs (Advanced Cases)
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
#[non_exhaustive]
struct ApiResponse {
    success: bool,
    data: Option<String>,
    error_code: Option<u32>,
    timestamp: u64,
}

test_roundtrip!(
    test_api_response_success,
    ApiResponse {
        success: true,
        data: Some("simple response data".to_string()),
        error_code: None,
        timestamp: 1704067200,
    },
    ApiResponse
);

test_roundtrip!(
    test_api_response_error,
    ApiResponse {
        success: false,
        data: None,
        error_code: Some(404),
        timestamp: 1704067201,
    },
    ApiResponse
);

#[derive(Debug, Deserialize, PartialEq)]
#[non_exhaustive]
struct DatabaseConfig {
    host: String,
    port: u16,
    database: String,
    pool_size: u32,
    ssl_enabled: bool,
    timeout_seconds: u64,
}

test_roundtrip!(
    test_database_config,
    DatabaseConfig {
        host: "db.example.com".to_string(),
        port: 5432,
        database: "production_db".to_string(),
        pool_size: 50,
        ssl_enabled: true,
        timeout_seconds: 30,
    },
    DatabaseConfig
);

// ==========================================
// Special Values
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
struct SpecialValues {
    max_int: u64,
    min_int: i64,
    float_vals: Vec<f64>,
    bool_vals: (bool, bool),
}

test_roundtrip!(
    test_special_values,
    SpecialValues {
        max_int: u64::MAX,
        min_int: i64::MIN,
        float_vals: vec![1.23456789, -999.999, f64::INFINITY, f64::NEG_INFINITY],
        bool_vals: (true, false),
    },
    SpecialValues
);

#[test]
fn test_nan_handling() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct WithNaN {
        normal: f64,
        nan_val: f64,
    }

    let original = WithNaN {
        normal: 42.0,
        nan_val: f64::NAN,
    };

    let debug_output = format!("{:?}", original);
    eprintln!("Debug output: {}", debug_output);

    let deserialized: WithNaN = serde_dbgfmt::from_str(&debug_output)
        .unwrap_or_else(|e| panic!("Failed to deserialize: {}", e));

    assert_eq!(original.normal, deserialized.normal);
    assert!(deserialized.nan_val.is_nan());
}

// ==========================================
// Custom Debug Implementations
// ==========================================

#[derive(Deserialize, PartialEq)]
struct CustomDebugStruct {
    value: i32,
    name: String,
}

impl Debug for CustomDebugStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomDebugStruct")
            .field("value", &self.value)
            .field("name", &self.name)
            .finish()
    }
}

test_debug_helper!(
    test_custom_debug_struct,
    "CustomDebugStruct { value: 42, name: \"test\" }",
    CustomDebugStruct,
    CustomDebugStruct {
        value: 42,
        name: "test".to_string(),
    }
);

#[derive(Deserialize, PartialEq)]
struct CustomDebugTuple(u32, String, bool);

impl Debug for CustomDebugTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("CustomDebugTuple")
            .field(&self.0)
            .field(&self.1)
            .field(&self.2)
            .finish()
    }
}

test_debug_helper!(
    test_custom_debug_tuple,
    "CustomDebugTuple(123, \"hello\", true)",
    CustomDebugTuple,
    CustomDebugTuple(123, "hello".to_string(), true)
);

#[derive(Deserialize, PartialEq)]
struct CustomDebugList {
    items: Vec<i32>,
}

impl Debug for CustomDebugList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomDebugList")
            .field("items", &self.items)
            .finish()
    }
}

test_debug_helper!(
    test_custom_debug_list,
    "CustomDebugList { items: [1, 2, 3, 4, 5] }",
    CustomDebugList,
    CustomDebugList {
        items: vec![1, 2, 3, 4, 5],
    }
);

// ==========================================
// Real-world Data Structures
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
struct HttpRequest {
    method: String,
    url: String,
    headers: BTreeMap<String, String>,
    body: Option<String>,
    timeout_ms: Option<u64>,
}

test_roundtrip!(
    test_http_request_get,
    HttpRequest {
        method: "GET".to_string(),
        url: "https://api.example.com/users/123".to_string(),
        headers: {
            let mut headers = BTreeMap::new();
            headers.insert("User-Agent".to_string(), "MyApp/1.0".to_string());
            headers.insert("Accept".to_string(), "application/json".to_string());
            headers.insert("Authorization".to_string(), "Bearer token123".to_string());
            headers
        },
        body: None,
        timeout_ms: Some(5000),
    },
    HttpRequest
);

test_roundtrip!(
    test_http_request_post,
    HttpRequest {
        method: "POST".to_string(),
        url: "https://api.example.com/users".to_string(),
        headers: {
            let mut headers = BTreeMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            headers.insert("User-Agent".to_string(), "MyApp/1.0".to_string());
            headers
        },
        body: Some("name=John&email=john@example.com".to_string()),
        timeout_ms: None,
    },
    HttpRequest
);

#[derive(Debug, Deserialize, PartialEq)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Deserialize, PartialEq)]
struct LogEntry {
    level: LogLevel,
    message: String,
    module: Option<String>,
    line: Option<u32>,
    timestamp: u64,
    fields: BTreeMap<String, String>,
}

test_roundtrip!(
    test_log_entry_error,
    LogEntry {
        level: LogLevel::Error,
        message: "Database connection failed".to_string(),
        module: Some("db::connection".to_string()),
        line: Some(42),
        timestamp: 1704067200,
        fields: {
            let mut fields = BTreeMap::new();
            fields.insert("error_code".to_string(), "DB_CONN_TIMEOUT".to_string());
            fields.insert("retry_count".to_string(), "3".to_string());
            fields
        },
    },
    LogEntry
);

test_roundtrip!(
    test_log_entry_info,
    LogEntry {
        level: LogLevel::Info,
        message: "Server started successfully".to_string(),
        module: None,
        line: None,
        timestamp: 1704067201,
        fields: BTreeMap::new(),
    },
    LogEntry
);

// ==========================================
// Complex Nested Options and Results
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
enum Result<T, E> {
    Ok(T),
    Err(E),
}

#[derive(Debug, Deserialize, PartialEq)]
struct ProcessingResult {
    id: u64,
    result: Result<String, String>,
    metadata: Option<BTreeMap<String, Option<String>>>,
}

test_roundtrip!(
    test_processing_result_ok,
    ProcessingResult {
        id: 12345,
        result: Result::Ok("Processing completed successfully".to_string()),
        metadata: Some({
            let mut map = BTreeMap::new();
            map.insert("duration_ms".to_string(), Some("1250".to_string()));
            map.insert("worker_id".to_string(), Some("worker-001".to_string()));
            map.insert("retry_count".to_string(), None);
            map
        }),
    },
    ProcessingResult
);

test_roundtrip!(
    test_processing_result_err,
    ProcessingResult {
        id: 67890,
        result: Result::Err("Invalid input format".to_string()),
        metadata: None,
    },
    ProcessingResult
);

// ==========================================
// Tuples and Complex Combinations
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
struct ComplexTuples {
    coordinates: (f64, f64, f64),
    name_score: (String, u32),
    flags: (bool, bool, bool, bool),
    nested: Option<(String, Vec<u32>, BTreeMap<String, bool>)>,
}

test_roundtrip!(
    test_complex_tuples,
    ComplexTuples {
        coordinates: (12.345, -67.890, 100.0),
        name_score: ("Alice".to_string(), 9500),
        flags: (true, false, true, false),
        nested: Some(("nested_data".to_string(), vec![1, 2, 3, 4, 5], {
            let mut map = BTreeMap::new();
            map.insert("feature_a".to_string(), true);
            map.insert("feature_b".to_string(), false);
            map
        })),
    },
    ComplexTuples
);

// ==========================================
// Array and Vector Edge Cases
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
struct Collections {
    single_item: Vec<u32>,
    mixed_options: Vec<Option<i32>>,
    nested_vecs: Vec<Vec<String>>,
}

test_roundtrip!(
    test_collections_edge_cases,
    Collections {
        single_item: vec![42],
        mixed_options: vec![Some(1), None, Some(-5), None, Some(0)],
        nested_vecs: vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["single".to_string()],
            vec!["x".to_string(), "y".to_string(), "z".to_string()],
        ],
    },
    Collections
);

// ==========================================
// String Cases (Safe)
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
struct StringCases {
    normal: String,
    with_spaces: String,
    with_numbers: String,
}

test_roundtrip!(
    test_string_cases,
    StringCases {
        normal: "Hello World".to_string(),
        with_spaces: "  spaces  at  ends  ".to_string(),
        with_numbers: "version-1.2.3-beta".to_string(),
    },
    StringCases
);

// ==========================================
// Large Numbers and Precision
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
struct NumericPrecision {
    large_positive: u128,
    large_negative: i128,
    high_precision: f64,
    small_float: f32,
}

test_roundtrip!(
    test_numeric_precision,
    NumericPrecision {
        large_positive: u128::MAX,
        large_negative: i128::MIN,
        high_precision: 123.456789012345,
        small_float: 3.14159,
    },
    NumericPrecision
);

// ==========================================
// Integration Tests with std::fmt Debug Helpers
// ==========================================

#[derive(Deserialize, PartialEq)]
struct DebugStructExample {
    field1: String,
    field2: i32,
    field3: bool,
}

impl Debug for DebugStructExample {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugStructExample")
            .field("field1", &self.field1)
            .field("field2", &self.field2)
            .field("field3", &self.field3)
            .finish()
    }
}

#[test]
fn test_debug_struct_helper() {
    let original = DebugStructExample {
        field1: "test".to_string(),
        field2: 42,
        field3: true,
    };

    let debug_output = format!("{:?}", original);
    eprintln!("Debug output: {}", debug_output);

    let deserialized: DebugStructExample = serde_dbgfmt::from_str(&debug_output)
        .unwrap_or_else(|e| panic!("Failed to deserialize: {}", e));

    assert_eq!(original, deserialized);
}

#[derive(Deserialize, PartialEq)]
struct DebugTupleExample(String, i32, bool);

impl Debug for DebugTupleExample {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DebugTupleExample")
            .field(&self.0)
            .field(&self.1)
            .field(&self.2)
            .finish()
    }
}

#[test]
fn test_debug_tuple_helper() {
    let original = DebugTupleExample("test".to_string(), 42, true);

    let debug_output = format!("{:?}", original);
    eprintln!("Debug output: {}", debug_output);

    let deserialized: DebugTupleExample = serde_dbgfmt::from_str(&debug_output)
        .unwrap_or_else(|e| panic!("Failed to deserialize: {}", e));

    assert_eq!(original, deserialized);
}

// ==========================================
// Comprehensive Examples
// ==========================================

#[derive(Debug, Deserialize, PartialEq)]
struct WebApplicationConfig {
    server: ServerConfig,
    database: DatabaseSettings,
    logging: LoggingConfig,
    features: Vec<FeatureFlag>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: u32,
    timeouts: TimeoutConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
struct TimeoutConfig {
    read_timeout_secs: u64,
    write_timeout_secs: u64,
    idle_timeout_secs: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
struct DatabaseSettings {
    url: String,
    pool_size: u32,
    timeout_secs: u64,
    retries: u8,
}

#[derive(Debug, Deserialize, PartialEq)]
struct LoggingConfig {
    level: LogLevel,
    output: String,
    format: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct FeatureFlag {
    name: String,
    enabled: bool,
    rollout_percentage: Option<f32>,
}

test_roundtrip!(
    test_comprehensive_config,
    WebApplicationConfig {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
            timeouts: TimeoutConfig {
                read_timeout_secs: 30,
                write_timeout_secs: 30,
                idle_timeout_secs: 60,
            },
        },
        database: DatabaseSettings {
            url: "postgresql://localhost:5432/myapp".to_string(),
            pool_size: 10,
            timeout_secs: 5,
            retries: 3,
        },
        logging: LoggingConfig {
            level: LogLevel::Info,
            output: "stdout".to_string(),
            format: "json".to_string(),
        },
        features: vec![
            FeatureFlag {
                name: "new_ui".to_string(),
                enabled: true,
                rollout_percentage: Some(50.0),
            },
            FeatureFlag {
                name: "beta_feature".to_string(),
                enabled: false,
                rollout_percentage: None,
            },
        ],
    },
    WebApplicationConfig
);

// ==========================================
// Real-World Debug Output Tests
// ==========================================

/// Test that mimics actual debug output from common Rust patterns
#[test]
fn test_real_world_option_result_patterns() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum ApiResult<T, E> {
        Ok(T),
        Err(E),
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct ApiCall {
        id: u32,
        result: ApiResult<String, String>,
        cached: Option<bool>,
    }

    let data = ApiCall {
        id: 42,
        result: ApiResult::Ok("success".to_string()),
        cached: Some(true),
    };

    let debug_str = format!("{:?}", data);
    eprintln!("Real-world debug: {}", debug_str);

    let parsed: ApiCall = serde_dbgfmt::from_str(&debug_str).unwrap();
    assert_eq!(data, parsed);
}

/// Test collections that appear frequently in real applications
#[test]
fn test_real_world_collections() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct ServiceMetrics {
        request_counts: BTreeMap<String, u64>,
        active_connections: BTreeSet<String>,
        response_times: Vec<f64>,
    }

    let metrics = ServiceMetrics {
        request_counts: {
            let mut map = BTreeMap::new();
            map.insert("GET".to_string(), 1000);
            map.insert("POST".to_string(), 500);
            map.insert("PUT".to_string(), 100);
            map
        },
        active_connections: {
            let mut set = BTreeSet::new();
            set.insert("conn-001".to_string());
            set.insert("conn-002".to_string());
            set.insert("conn-003".to_string());
            set
        },
        response_times: vec![0.1, 0.25, 0.8, 1.2, 0.05],
    };

    let debug_str = format!("{:?}", metrics);
    eprintln!("Service metrics debug: {}", debug_str);

    let parsed: ServiceMetrics = serde_dbgfmt::from_str(&debug_str).unwrap();
    assert_eq!(metrics, parsed);
}

/// Test that shows how the library handles actual enum variants used in
/// applications
#[test]
fn test_real_world_enum_variants() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum TaskStatus {
        Pending,
        Running { worker_id: String, started_at: u64 },
        Completed { duration_ms: u64, exit_code: i32 },
        Failed { error: String, retry_count: u8 },
    }

    let statuses = vec![
        TaskStatus::Pending,
        TaskStatus::Running {
            worker_id: "worker-007".to_string(),
            started_at: 1640995200,
        },
        TaskStatus::Completed {
            duration_ms: 5000,
            exit_code: 0,
        },
        TaskStatus::Failed {
            error: "Connection timeout".to_string(),
            retry_count: 3,
        },
    ];

    for status in statuses {
        let debug_str = format!("{:?}", status);
        eprintln!("Task status debug: {}", debug_str);

        let parsed: TaskStatus = serde_dbgfmt::from_str(&debug_str).unwrap();
        assert_eq!(status, parsed);
    }
}
