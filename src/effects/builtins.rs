use crate::errors::MtpError;
use crate::json::Json;

/// Built-in pure functions with input validation
pub fn call_builtin_function(name: &str, args: &[String]) -> Result<String, MtpError> {
    match name {
        "Json.parse" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "Json.parse requires exactly 1 argument".into(),
                ));
            }
            let input = &args[0];

            // Validate input size (prevent DoS)
            if input.len() > 10 * 1024 * 1024 {
                // 10MB limit
                return Err(MtpError::RuntimeError("Json.parse input too large".into()));
            }

            // Basic validation - check for obviously malicious content
            if input.contains("<script") || input.contains("javascript:") {
                return Err(MtpError::RuntimeError(
                    "Json.parse rejected potentially malicious input".into(),
                ));
            }

            match Json::parse(input) {
                Ok(json) => Ok(serde_json::to_string(&json).unwrap()), // Placeholder serialization
                Err(_) => Err(MtpError::RuntimeError("Invalid JSON".into())),
            }
        }
        "Json.stringify" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "Json.stringify requires exactly 1 argument".into(),
                ));
            }
            // For now, just return the input - proper implementation needed
            Ok(args[0].clone())
        }
        "Decimal.fromString" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "Decimal.fromString requires exactly 1 argument".into(),
                ));
            }
            let input = &args[0];

            // Validate decimal format
            if !is_valid_decimal_string(input) {
                return Err(MtpError::RuntimeError("invalid decimal string".into()));
            }

            // Parse and normalize the decimal
            Ok(normalize_decimal_string(input))
        }
        "Decimal.toString" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "Decimal.toString requires exactly 1 argument".into(),
                ));
            }
            // Already normalized from fromString, just return it
            Ok(args[0].clone())
        }
        "fnv1a32" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "fnv1a32 requires exactly 1 argument".into(),
                ));
            }
            let input = &args[0];

            // Validate input size
            if input.len() > 1024 * 1024 {
                // 1MB limit
                return Err(MtpError::RuntimeError("fnv1a32 input too large".into()));
            }

            let hash = fnv1a_32(input.as_bytes());
            Ok(hash.to_string())
        }
        "fnv1a64" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "fnv1a64 requires exactly 1 argument".into(),
                ));
            }
            let input = &args[0];

            // Validate input size
            if input.len() > 1024 * 1024 {
                // 1MB limit
                return Err(MtpError::RuntimeError("fnv1a64 input too large".into()));
            }

            let hash = fnv1a_64(input.as_bytes());
            Ok(hash.to_string())
        }
        "cborEncode" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "cborEncode requires exactly 1 argument".into(),
                ));
            }
            let input = &args[0];

            // Validate input size
            if input.len() > 10 * 1024 * 1024 {
                // 10MB limit
                return Err(MtpError::RuntimeError("cborEncode input too large".into()));
            }

            match Json::parse(input) {
                Ok(json) => {
                    let cbor = crate::json::encode_cbor(&json);
                    Ok(hex::encode(cbor))
                }
                Err(_) => Err(MtpError::RuntimeError(
                    "Invalid JSON for CBOR encoding".into(),
                )),
            }
        }
            let input = &args[0];

            // Validate input size
            if input.len() > 10 * 1024 * 1024 {
                // 10MB limit
                return Err(MtpError::RuntimeError("cborEncode input too large".into()));
            }

            match Json::parse(input) {
                Ok(json) => {
                    let cbor = crate::json::encode_cbor(&json);
                    Ok(hex::encode(cbor))
                }
                Err(_) => Err(MtpError::RuntimeError(
                    "Invalid JSON for CBOR encoding".into(),
                )),
            }
        }
        // ADT Constructors
        "Some" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "Some requires exactly 1 argument".into(),
                ));
            }
            // Return ADT representation - for now as JSON-like string
            Ok(format!("{{\"Some\":{}}}", args[0]))
        }
        "None" => {
            if args.len() != 0 {
                return Err(MtpError::RuntimeError("None requires no arguments".into()));
            }
            Ok("{\"None\":{}}".to_string())
        }
        "Ok" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "Ok requires exactly 1 argument".into(),
                ));
            }
            Ok(format!("{{\"Ok\":{}}}", args[0]))
        }
        "Err" => {
            if args.len() != 1 {
                return Err(MtpError::RuntimeError(
                    "Err requires exactly 1 argument".into(),
                ));
            }
            Ok(format!("{{\"Err\":{}}}", args[0]))
        }
        _ => Err(MtpError::RuntimeError(format!(
            "Unknown built-in function: {}",
            name
        ))),
    }
}

/// Validate decimal string format
fn is_valid_decimal_string(s: &str) -> bool {
    // Basic validation: optional sign, digits, optional decimal point with digits
    let mut chars = s.chars();
    let mut has_digits = false;
    let mut has_decimal = false;

    // Optional sign
    if let Some(c) = chars.next() {
        if c != '-' && c != '+' && !c.is_ascii_digit() {
            return false;
        }
        if c.is_ascii_digit() {
            has_digits = true;
        }
    }

    for c in chars {
        if c.is_ascii_digit() {
            has_digits = true;
        } else if c == '.' {
            if has_decimal {
                return false; // Multiple decimal points
            }
            has_decimal = true;
        } else {
            return false; // Invalid character
        }
    }

    has_digits && s.len() <= 34 // Max 34 characters as per spec
}

/// Normalize a decimal string by removing trailing zeros after decimal point
fn normalize_decimal_string(s: &str) -> String {
    // Handle sign
    let (sign, rest) = if s.starts_with('-') {
        ("-", &s[1..])
    } else if s.starts_with('+') {
        ("", &s[1..])
    } else {
        ("", s)
    };

    // Split on decimal point
    if let Some(dot_pos) = rest.find('.') {
        let integer_part = &rest[..dot_pos];
        let fractional_part = &rest[dot_pos + 1..];

        // Remove trailing zeros from fractional part
        let trimmed_frac = fractional_part.trim_end_matches('0');

        // Remove leading zeros from integer part (but keep at least one digit)
        let trimmed_int = integer_part.trim_start_matches('0');
        let int_part = if trimmed_int.is_empty() { "0" } else { trimmed_int };

        if trimmed_frac.is_empty() {
            // No fractional part after trimming
            format!("{}{}", sign, int_part)
        } else {
            format!("{}{}.{}", sign, int_part, trimmed_frac)
        }
    } else {
        // No decimal point - remove leading zeros
        let trimmed = rest.trim_start_matches('0');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            format!("{}{}", sign, trimmed)
        }
    }
}

/// FNV-1a 32-bit hash
fn fnv1a_32(data: &[u8]) -> u32 {
    const FNV_PRIME: u32 = 0x01000193;
    const FNV_OFFSET: u32 = 0x811c9dc5;

    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// FNV-1a 64-bit hash
fn fnv1a_64(data: &[u8]) -> u64 {
    const FNV_PRIME: u64 = 0x00000100000001B3;
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;

    let mut hash = FNV_OFFSET;
    for &byte in data {
        hash ^= data as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
