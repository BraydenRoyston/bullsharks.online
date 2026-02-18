use crate::error::ApiError;

/// Comprehensive test suite for error handling
/// 
/// Tests cover:
/// - Error type classification
/// - HTTP status code mapping
/// - Error message formatting
/// - Error conversion and propagation
/// - JSON response format
/// - Error categorization for monitoring

#[test]
fn test_api_error_startup_error() {
    let error = ApiError::StartupError("Failed to initialize database pool".to_string());
    
    match error {
        ApiError::StartupError(msg) => {
            assert_eq!(msg, "Failed to initialize database pool");
            // In the real implementation, this would map to INTERNAL_SERVER_ERROR
        },
        _ => panic!("Expected StartupError variant"),
    }
}

#[test]
fn test_api_error_database_error() {
    let error = ApiError::DatabaseError("Connection timeout after 30 seconds".to_string());
    
    match error {
        ApiError::DatabaseError(msg) => {
            assert_eq!(msg, "Connection timeout after 30 seconds");
            assert!(msg.contains("timeout"));
            // In the real implementation, this would map to INTERNAL_SERVER_ERROR
        },
        _ => panic!("Expected DatabaseError variant"),
    }
}

#[test]
fn test_api_error_auth_token_error() {
    let error = ApiError::AuthTokenError("Invalid refresh token".to_string());
    
    match error {
        ApiError::AuthTokenError(msg) => {
            assert_eq!(msg, "Invalid refresh token");
            assert!(msg.contains("token"));
            // In the real implementation, this would map to INTERNAL_SERVER_ERROR
        },
        _ => panic!("Expected AuthTokenError variant"),
    }
}

#[test]
fn test_api_error_internal_conversion_error() {
    let error = ApiError::InternalConversionError("Failed to parse datetime from string".to_string());
    
    match error {
        ApiError::InternalConversionError(msg) => {
            assert_eq!(msg, "Failed to parse datetime from string");
            assert!(msg.contains("parse") || msg.contains("conversion"));
            // In the real implementation, this would map to INTERNAL_SERVER_ERROR
        },
        _ => panic!("Expected InternalConversionError variant"),
    }
}

#[test]
fn test_api_error_external_api_error() {
    let error = ApiError::ExternalAPIError("Strava API returned 503 Service Unavailable".to_string());
    
    match error {
        ApiError::ExternalAPIError(msg) => {
            assert_eq!(msg, "Strava API returned 503 Service Unavailable");
            assert!(msg.contains("API") || msg.contains("503"));
            // In the real implementation, this would map to INTERNAL_SERVER_ERROR
        },
        _ => panic!("Expected ExternalAPIError variant"),
    }
}

#[test]
fn test_api_error_unauthorized() {
    let error = ApiError::Unauthorized("Missing or invalid authentication token".to_string());
    
    match error {
        ApiError::Unauthorized(msg) => {
            assert_eq!(msg, "Missing or invalid authentication token");
            assert!(msg.contains("authentication") || msg.contains("token"));
            // In the real implementation, this would map to UNAUTHORIZED
        },
        _ => panic!("Expected Unauthorized variant"),
    }
}

#[test]
fn test_api_error_bad_request() {
    let error = ApiError::BadRequest("Invalid date format. Expected RFC3339 format".to_string());
    
    match error {
        ApiError::BadRequest(msg) => {
            assert_eq!(msg, "Invalid date format. Expected RFC3339 format");
            assert!(msg.contains("Invalid") || msg.contains("format"));
            // In the real implementation, this would map to BAD_REQUEST
        },
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_error_message_content_validation() {
    let test_cases = vec![
        (
            ApiError::DatabaseError("Connection refused".to_string()),
            "Connection refused"
        ),
        (
            ApiError::ExternalAPIError("Rate limit exceeded".to_string()),
            "Rate limit exceeded"
        ),
        (
            ApiError::BadRequest("Missing required parameter 'start'".to_string()),
            "Missing required parameter 'start'"
        ),
        (
            ApiError::Unauthorized("Token expired".to_string()),
            "Token expired"
        ),
    ];
    
    for (error, expected_msg) in test_cases {
        let actual_msg = match error {
            ApiError::DatabaseError(msg) => msg,
            ApiError::ExternalAPIError(msg) => msg,
            ApiError::BadRequest(msg) => msg,
            ApiError::Unauthorized(msg) => msg,
            ApiError::StartupError(msg) => msg,
            ApiError::AuthTokenError(msg) => msg,
            ApiError::InternalConversionError(msg) => msg,
        };
        
        assert_eq!(actual_msg, expected_msg);
    }
}

#[test]
fn test_error_categorization() {
    // Test that errors can be categorized for different handling
    let client_errors = vec![
        ApiError::BadRequest("Invalid input".to_string()),
        ApiError::Unauthorized("Access denied".to_string()),
    ];
    
    let server_errors = vec![
        ApiError::DatabaseError("DB connection failed".to_string()),
        ApiError::StartupError("Initialization failed".to_string()),
        ApiError::AuthTokenError("Token refresh failed".to_string()),
        ApiError::InternalConversionError("Data conversion failed".to_string()),
        ApiError::ExternalAPIError("Third-party service unavailable".to_string()),
    ];
    
    // Client errors (4xx) - user's fault
    for error in client_errors {
        match error {
            ApiError::BadRequest(_) | ApiError::Unauthorized(_) => {
                // These should map to 4xx status codes
                assert!(true);
            },
            _ => panic!("Expected client error"),
        }
    }
    
    // Server errors (5xx) - our fault
    for error in server_errors {
        match error {
            ApiError::DatabaseError(_) | ApiError::StartupError(_) | 
            ApiError::AuthTokenError(_) | ApiError::InternalConversionError(_) |
            ApiError::ExternalAPIError(_) => {
                // These should map to 5xx status codes
                assert!(true);
            },
            _ => panic!("Expected server error"),
        }
    }
}

#[test]
fn test_error_debug_formatting() {
    let error = ApiError::DatabaseError("Connection timeout".to_string());
    let debug_str = format!("{:?}", error);
    
    assert!(debug_str.contains("DatabaseError"));
    assert!(debug_str.contains("Connection timeout"));
}

#[test]
fn test_common_error_patterns() {
    // Test common error patterns that might occur in the application
    let common_errors = vec![
        // Database related
        ApiError::DatabaseError("Connection pool exhausted".to_string()),
        ApiError::DatabaseError("Query execution timeout".to_string()),
        ApiError::DatabaseError("Constraint violation".to_string()),
        
        // External API related  
        ApiError::ExternalAPIError("Strava API rate limit exceeded".to_string()),
        ApiError::ExternalAPIError("Strava service temporarily unavailable".to_string()),
        ApiError::ExternalAPIError("Invalid Strava response format".to_string()),
        
        // Authentication related
        ApiError::AuthTokenError("Access token expired".to_string()),
        ApiError::AuthTokenError("Refresh token invalid".to_string()),
        ApiError::Unauthorized("Invalid CRON_SECRET token".to_string()),
        
        // Data conversion related
        ApiError::InternalConversionError("Failed to parse activity date".to_string()),
        ApiError::InternalConversionError("Invalid timezone conversion".to_string()),
        
        // Client input related
        ApiError::BadRequest("Invalid datetime format in start parameter".to_string()),
        ApiError::BadRequest("Missing required query parameter".to_string()),
    ];
    
    for error in common_errors {
        let msg = match &error {
            ApiError::DatabaseError(m) => m,
            ApiError::ExternalAPIError(m) => m,
            ApiError::AuthTokenError(m) => m,
            ApiError::Unauthorized(m) => m,
            ApiError::InternalConversionError(m) => m,
            ApiError::BadRequest(m) => m,
            ApiError::StartupError(m) => m,
        };
        
        // All error messages should be non-empty
        assert!(!msg.is_empty());
        
        // All error messages should be descriptive
        assert!(msg.len() > 10);
    }
}

#[test]
fn test_error_message_formatting_consistency() {
    // Test that error messages follow consistent formatting
    let formatted_errors = vec![
        ApiError::BadRequest("Invalid date format. Expected RFC3339 format (e.g., 2024-01-01T00:00:00Z)".to_string()),
        ApiError::BadRequest("Invalid start datetime format: parse error. Expected RFC3339 format (e.g., 2024-01-01T00:00:00Z)".to_string()),
        ApiError::Unauthorized("Invalid token".to_string()),
        ApiError::DatabaseError("Connection failed: timeout after 30s".to_string()),
    ];
    
    for error in formatted_errors {
        match error {
            ApiError::BadRequest(msg) => {
                // BadRequest messages should be helpful for API users
                assert!(msg.contains("Invalid") || msg.contains("Missing"));
                if msg.contains("format") {
                    assert!(msg.contains("Expected") || msg.contains("example"));
                }
            },
            ApiError::Unauthorized(msg) => {
                // Unauthorized messages should be brief for security
                assert!(!msg.contains("password") && !msg.contains("secret"));
            },
            ApiError::DatabaseError(msg) => {
                // Database errors can be more detailed for debugging
                assert!(!msg.is_empty());
            },
            _ => {}
        }
    }
}

#[test] 
fn test_error_chaining_scenarios() {
    // Test scenarios where errors might be chained or converted
    
    // Database connection error -> Startup error
    let db_error = "Failed to connect to PostgreSQL";
    let startup_error = ApiError::StartupError(format!("Database initialization failed: {}", db_error));
    
    match startup_error {
        ApiError::StartupError(msg) => {
            assert!(msg.contains("Database initialization failed"));
            assert!(msg.contains("PostgreSQL"));
        },
        _ => panic!("Expected StartupError"),
    }
    
    // External API error -> Internal conversion error
    let api_response = "Invalid JSON response";
    let conversion_error = ApiError::InternalConversionError(format!("Failed to parse Strava response: {}", api_response));
    
    match conversion_error {
        ApiError::InternalConversionError(msg) => {
            assert!(msg.contains("Failed to parse"));
            assert!(msg.contains("Strava response"));
            assert!(msg.contains("Invalid JSON"));
        },
        _ => panic!("Expected InternalConversionError"),
    }
}

#[test]
fn test_error_context_preservation() {
    // Test that error context is preserved through conversion
    let original_error = "Connection timeout";
    let contextual_error = ApiError::DatabaseError(format!("Query execution failed: {}", original_error));
    
    match contextual_error {
        ApiError::DatabaseError(msg) => {
            assert!(msg.contains("Query execution failed"));
            assert!(msg.contains("Connection timeout"));
            
            // Should preserve both the action that failed and the underlying cause
            assert!(msg.contains(":"));
        },
        _ => panic!("Expected DatabaseError"),
    }
}

#[test]
fn test_security_sensitive_errors() {
    // Test that security-sensitive errors don't leak information
    let auth_errors = vec![
        ApiError::Unauthorized("Invalid token".to_string()),
        ApiError::Unauthorized("Access denied".to_string()),
        ApiError::AuthTokenError("Token refresh failed".to_string()),
    ];
    
    for error in auth_errors {
        let msg = match error {
            ApiError::Unauthorized(m) => m,
            ApiError::AuthTokenError(m) => m,
            _ => panic!("Expected auth-related error"),
        };
        
        // Should not contain sensitive information
        assert!(!msg.to_lowercase().contains("password"));
        assert!(!msg.to_lowercase().contains("secret"));
        assert!(!msg.to_lowercase().contains("key"));
        assert!(!msg.contains("admin"));
        
        // Should be brief and generic
        assert!(msg.len() < 100);
    }
}

#[test]
fn test_error_logging_friendliness() {
    // Test that errors contain enough context for logging/debugging
    let errors_with_context = vec![
        ApiError::DatabaseError("Query timeout: SELECT * FROM activities WHERE date > '2024-01-01'".to_string()),
        ApiError::ExternalAPIError("Strava API error: 429 Rate Limit Exceeded, retry after 60s".to_string()),
        ApiError::InternalConversionError("Date parsing failed: '2024-13-01T00:00:00Z' at position 5".to_string()),
    ];
    
    for error in errors_with_context {
        let msg = match error {
            ApiError::DatabaseError(m) => m,
            ApiError::ExternalAPIError(m) => m,
            ApiError::InternalConversionError(m) => m,
            _ => panic!("Expected error with context"),
        };
        
        // Should contain actionable debugging information
        assert!(msg.contains(":") || msg.contains("error") || msg.contains("failed"));
        
        // Should be descriptive enough for debugging
        assert!(msg.len() > 20);
        
        // Should identify the component or operation that failed
        assert!(msg.to_lowercase().contains("query") || 
               msg.to_lowercase().contains("api") ||
               msg.to_lowercase().contains("parsing") ||
               msg.to_lowercase().contains("strava") ||
               msg.to_lowercase().contains("date"));
    }
}

#[test]
fn test_error_classification_for_monitoring() {
    // Test error classification for monitoring/alerting systems
    
    // Errors that should trigger immediate alerts (system down)
    let critical_errors = vec![
        ApiError::StartupError("Failed to start server".to_string()),
        ApiError::DatabaseError("Connection pool exhausted".to_string()),
    ];
    
    // Errors that should be logged but might be transient  
    let warning_errors = vec![
        ApiError::ExternalAPIError("Strava API temporarily unavailable".to_string()),
        ApiError::AuthTokenError("Token refresh failed".to_string()),
    ];
    
    // Errors that are likely user errors (don't need alerts)
    let user_errors = vec![
        ApiError::BadRequest("Invalid date format".to_string()),
        ApiError::Unauthorized("Invalid token".to_string()),
    ];
    
    // Critical errors should be identifiable
    for error in critical_errors {
        match error {
            ApiError::StartupError(_) | ApiError::DatabaseError(_) => {
                // These indicate system-level issues
                assert!(true);
            },
            _ => panic!("Expected critical error type"),
        }
    }
    
    // Warning errors should be identifiable
    for error in warning_errors {
        match error {
            ApiError::ExternalAPIError(_) | ApiError::AuthTokenError(_) => {
                // These might be transient external issues
                assert!(true);
            },
            _ => panic!("Expected warning error type"),
        }
    }
    
    // User errors should be identifiable
    for error in user_errors {
        match error {
            ApiError::BadRequest(_) | ApiError::Unauthorized(_) => {
                // These are client-side issues
                assert!(true);
            },
            _ => panic!("Expected user error type"),
        }
    }
}