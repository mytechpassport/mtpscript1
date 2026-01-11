use crate::errors::compile::CompileError;
use std::fmt;

/// Decimal type for precise fixed-point arithmetic per §4-a of the spec.
/// - value: canonical integer significand (1-34 digits, no leading zeros)
/// - scale: 0 ≤ scale ≤ 28 (IEEE-754-2008 decimal128)
/// - Rounding: round-half-even (banker's rounding) per IEEE-754-2008 clause 4.3.2
#[derive(Debug, Clone, Eq, serde::Serialize, serde::Deserialize)]
pub struct Decimal {
    /// The significand as a string of digits (no leading zeros except for "0")
    significand: String,
    /// Whether the number is negative
    negative: bool,
    /// Scale (number of decimal places), 0 ≤ scale ≤ 28
    scale: u8,
}

/// Maximum digits allowed (IEEE-754 decimal128 has 34 significant digits)
const MAX_DIGITS: usize = 34;
/// Maximum scale
const MAX_SCALE: u8 = 28;

impl Decimal {
    /// Create a new Decimal from significand, sign, and scale
    fn new(significand: String, negative: bool, scale: u8) -> Result<Self, CompileError> {
        if significand.is_empty() || significand.len() > MAX_DIGITS {
            return Err(CompileError::TypeError(
                "Overflow: significand out of range".to_string(),
            ));
        }
        if scale > MAX_SCALE {
            return Err(CompileError::TypeError(
                "Scale too large (max 28)".to_string(),
            ));
        }
        // Normalize: remove leading zeros (except keep single "0")
        let normalized = significand.trim_start_matches('0');
        let normalized = if normalized.is_empty() {
            "0"
        } else {
            normalized
        };

        Ok(Decimal {
            significand: normalized.to_string(),
            negative: negative && normalized != "0", // -0 becomes 0
            scale,
        })
    }

    /// Create a zero decimal
    pub fn zero() -> Self {
        Decimal {
            significand: "0".to_string(),
            negative: false,
            scale: 0,
        }
    }

    /// Parse a decimal from string representation
    pub fn from_str(s: &str) -> Result<Self, CompileError> {
        if s.is_empty() {
            return Err(CompileError::TypeError("Empty decimal string".to_string()));
        }

        let (negative, s) = if s.starts_with('-') {
            (true, &s[1..])
        } else if s.starts_with('+') {
            (false, &s[1..])
        } else {
            (false, s)
        };

        if s.is_empty() {
            return Err(CompileError::TypeError(
                "Invalid decimal format".to_string(),
            ));
        }

        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() > 2 {
            return Err(CompileError::TypeError(
                "Invalid decimal format: multiple decimal points".to_string(),
            ));
        }

        let integer_part = parts[0];
        let fractional_part = if parts.len() == 2 { parts[1] } else { "" };

        // Validate parts contain only digits
        if !integer_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(CompileError::TypeError(
                "Invalid characters in integer part".to_string(),
            ));
        }
        if !fractional_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(CompileError::TypeError(
                "Invalid characters in fractional part".to_string(),
            ));
        }

        // Check for leading zeros in integer part (except "0" or "0.xxx")
        if integer_part.len() > 1 && integer_part.starts_with('0') {
            return Err(CompileError::TypeError(
                "Leading zeros not allowed".to_string(),
            ));
        }

        let scale = fractional_part.len() as u8;
        if scale > MAX_SCALE {
            return Err(CompileError::TypeError(
                "Scale too large (max 28)".to_string(),
            ));
        }

        // Combine into significand
        let significand = format!("{}{}", integer_part, fractional_part);

        Decimal::new(significand, negative, scale)
    }

    /// Convert to canonical string representation (shortest form, no trailing zeros)
    pub fn to_decimal_string(&self) -> String {
        self.to_string_with_scale(None)
    }

    /// Convert to string with optional fixed scale
    fn to_string_with_scale(&self, fixed_scale: Option<u8>) -> String {
        let scale = fixed_scale.unwrap_or(self.scale);
        let sign = if self.negative { "-" } else { "" };

        if scale == 0 {
            return format!("{}{}", sign, self.significand);
        }

        let sig_len = self.significand.len();
        if sig_len <= scale as usize {
            // Need leading zeros after decimal point
            let zeros = scale as usize - sig_len;
            format!("{}0.{}{}", sign, "0".repeat(zeros), self.significand)
        } else {
            let int_part = &self.significand[..sig_len - scale as usize];
            let frac_part = &self.significand[sig_len - scale as usize..];
            // Remove trailing zeros from fractional part for canonical form
            let frac_part = if fixed_scale.is_none() {
                frac_part.trim_end_matches('0')
            } else {
                frac_part
            };
            if frac_part.is_empty() {
                format!("{}{}", sign, int_part)
            } else {
                format!("{}{}.{}", sign, int_part, frac_part)
            }
        }
    }

    /// Getter for value (significand as string for compatibility)
    pub fn value(&self) -> &str {
        &self.significand
    }

    /// Getter for scale
    pub fn scale(&self) -> u8 {
        self.scale
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.significand == "0"
    }

    /// Check if negative
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    /// Normalize two decimals to the same scale (the larger of the two)
    fn normalize_scales(a: &Decimal, b: &Decimal) -> (String, String, u8) {
        let target_scale = a.scale.max(b.scale);

        let a_sig = Self::adjust_scale_significand(&a.significand, a.scale, target_scale);
        let b_sig = Self::adjust_scale_significand(&b.significand, b.scale, target_scale);

        (a_sig, b_sig, target_scale)
    }

    /// Adjust significand to new scale by adding trailing zeros
    fn adjust_scale_significand(sig: &str, from_scale: u8, to_scale: u8) -> String {
        if from_scale >= to_scale {
            sig.to_string()
        } else {
            let zeros_to_add = (to_scale - from_scale) as usize;
            format!("{}{}", sig, "0".repeat(zeros_to_add))
        }
    }

    /// Add two positive significands as strings
    fn add_significands(a: &str, b: &str) -> String {
        let mut result = Vec::new();
        let mut carry = 0u8;

        let a_bytes: Vec<u8> = a.bytes().rev().collect();
        let b_bytes: Vec<u8> = b.bytes().rev().collect();
        let max_len = a_bytes.len().max(b_bytes.len());

        for i in 0..max_len {
            let a_digit = if i < a_bytes.len() {
                a_bytes[i] - b'0'
            } else {
                0
            };
            let b_digit = if i < b_bytes.len() {
                b_bytes[i] - b'0'
            } else {
                0
            };

            let sum = a_digit + b_digit + carry;
            result.push((sum % 10) + b'0');
            carry = sum / 10;
        }

        if carry > 0 {
            result.push(carry + b'0');
        }

        result.reverse();
        String::from_utf8(result).unwrap()
    }

    /// Subtract two positive significands (a - b), assuming a >= b
    fn subtract_significands(a: &str, b: &str) -> String {
        let mut result = Vec::new();
        let mut borrow = 0i8;

        let a_bytes: Vec<u8> = a.bytes().rev().collect();
        let b_bytes: Vec<u8> = b.bytes().rev().collect();

        for i in 0..a_bytes.len() {
            let a_digit = (a_bytes[i] - b'0') as i8;
            let b_digit = if i < b_bytes.len() {
                (b_bytes[i] - b'0') as i8
            } else {
                0
            };

            let mut diff = a_digit - b_digit - borrow;
            if diff < 0 {
                diff += 10;
                borrow = 1;
            } else {
                borrow = 0;
            }
            result.push((diff as u8) + b'0');
        }

        result.reverse();
        // Remove leading zeros
        let s = String::from_utf8(result).unwrap();
        let trimmed = s.trim_start_matches('0');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Compare two significands: -1 if a < b, 0 if equal, 1 if a > b
    fn compare_significands(a: &str, b: &str) -> i8 {
        if a.len() != b.len() {
            return if a.len() > b.len() { 1 } else { -1 };
        }
        for (ca, cb) in a.chars().zip(b.chars()) {
            if ca != cb {
                return if ca > cb { 1 } else { -1 };
            }
        }
        0
    }

    /// Add two decimals
    pub fn add(&self, other: &Decimal) -> Result<Decimal, CompileError> {
        let (a_sig, b_sig, scale) = Self::normalize_scales(self, other);

        let (result_sig, result_neg) = match (self.negative, other.negative) {
            (false, false) => {
                // Both positive: add
                (Self::add_significands(&a_sig, &b_sig), false)
            }
            (true, true) => {
                // Both negative: add and negate
                (Self::add_significands(&a_sig, &b_sig), true)
            }
            (false, true) => {
                // a - |b|
                let cmp = Self::compare_significands(&a_sig, &b_sig);
                if cmp >= 0 {
                    (Self::subtract_significands(&a_sig, &b_sig), false)
                } else {
                    (Self::subtract_significands(&b_sig, &a_sig), true)
                }
            }
            (true, false) => {
                // |b| - |a|
                let cmp = Self::compare_significands(&a_sig, &b_sig);
                if cmp <= 0 {
                    (Self::subtract_significands(&b_sig, &a_sig), false)
                } else {
                    (Self::subtract_significands(&a_sig, &b_sig), true)
                }
            }
        };

        if result_sig.len() > MAX_DIGITS {
            return Err(CompileError::TypeError(
                "Overflow: result too large".to_string(),
            ));
        }

        Decimal::new(result_sig, result_neg, scale)
    }

    /// Subtract: self - other
    pub fn sub(&self, other: &Decimal) -> Result<Decimal, CompileError> {
        let negated = Decimal {
            significand: other.significand.clone(),
            negative: !other.negative,
            scale: other.scale,
        };
        self.add(&negated)
    }

    /// Multiply two decimals
    pub fn mul(&self, other: &Decimal) -> Result<Decimal, CompileError> {
        // Multiply significands
        let result_sig = Self::multiply_significands(&self.significand, &other.significand);

        // Result scale is sum of scales
        let result_scale = self.scale as u16 + other.scale as u16;
        if result_scale > MAX_SCALE as u16 {
            // Need to round to fit scale
            let excess = result_scale - MAX_SCALE as u16;
            let rounded = Self::round_significand(&result_sig, excess as usize)?;
            Decimal::new(rounded, self.negative != other.negative, MAX_SCALE)
        } else {
            if result_sig.len() > MAX_DIGITS {
                return Err(CompileError::TypeError(
                    "Overflow: result too large".to_string(),
                ));
            }
            Decimal::new(
                result_sig,
                self.negative != other.negative,
                result_scale as u8,
            )
        }
    }

    /// Multiply two significands
    fn multiply_significands(a: &str, b: &str) -> String {
        let a_digits: Vec<u8> = a.bytes().rev().map(|c| c - b'0').collect();
        let b_digits: Vec<u8> = b.bytes().rev().map(|c| c - b'0').collect();

        let mut result = vec![0u8; a_digits.len() + b_digits.len()];

        for (i, &ad) in a_digits.iter().enumerate() {
            for (j, &bd) in b_digits.iter().enumerate() {
                let prod = ad as u16 * bd as u16;
                let pos = i + j;
                result[pos] += (prod % 10) as u8;
                result[pos + 1] += (prod / 10) as u8;

                // Handle carries
                let mut k = pos;
                while result[k] >= 10 {
                    result[k + 1] += result[k] / 10;
                    result[k] %= 10;
                    k += 1;
                }
            }
        }

        // Convert to string
        while result.len() > 1 && result.last() == Some(&0) {
            result.pop();
        }
        result.reverse();
        result.iter().map(|&d| (d + b'0') as char).collect()
    }

    /// Divide two decimals with specified scale for result
    pub fn div(&self, other: &Decimal, result_scale: u8) -> Result<Decimal, CompileError> {
        if other.is_zero() {
            return Err(CompileError::TypeError("Division by zero".to_string()));
        }

        // Adjust dividend to have enough precision for the result
        let extra_digits = result_scale as usize + other.scale as usize + 1;
        let adjusted_dividend = format!("{}{}", self.significand, "0".repeat(extra_digits));

        let (quotient, _remainder) =
            Self::divide_significands(&adjusted_dividend, &other.significand);

        // Calculate actual scale of quotient
        let actual_scale = (self.scale as i16 - other.scale as i16 + extra_digits as i16) as u8;

        // Round to result_scale
        if actual_scale > result_scale {
            let digits_to_remove = (actual_scale - result_scale) as usize;
            let rounded = Self::round_significand(&quotient, digits_to_remove)?;
            Decimal::new(rounded, self.negative != other.negative, result_scale)
        } else {
            Decimal::new(quotient, self.negative != other.negative, actual_scale)
        }
    }

    /// Divide significands, returning (quotient, remainder)
    fn divide_significands(dividend: &str, divisor: &str) -> (String, String) {
        // Simple long division
        let divisor_val: u128 = divisor.parse().unwrap_or(0);
        if divisor_val == 0 {
            return ("0".to_string(), dividend.to_string());
        }

        let mut quotient = String::new();
        let mut current: u128 = 0;

        for digit in dividend.chars() {
            current = current * 10 + (digit as u128 - '0' as u128);
            let q = current / divisor_val;
            quotient.push((b'0' + q as u8) as char);
            current %= divisor_val;
        }

        let quotient = quotient.trim_start_matches('0');
        let quotient = if quotient.is_empty() { "0" } else { quotient };

        (quotient.to_string(), current.to_string())
    }

    /// Round a significand by removing `digits_to_remove` digits from the right,
    /// using round-half-even (banker's rounding)
    fn round_significand(sig: &str, digits_to_remove: usize) -> Result<String, CompileError> {
        if digits_to_remove >= sig.len() {
            return Ok("0".to_string());
        }
        if digits_to_remove == 0 {
            return Ok(sig.to_string());
        }

        let keep_len = sig.len() - digits_to_remove;
        let kept = &sig[..keep_len];
        let removed = &sig[keep_len..];

        // Determine if we need to round up
        let first_removed = removed.chars().next().unwrap();
        let round_up = if first_removed > '5' {
            true
        } else if first_removed < '5' {
            false
        } else {
            // Exactly half: check remaining digits
            let rest_all_zeros = removed[1..].chars().all(|c| c == '0');
            if rest_all_zeros {
                // Round to even: round up if last kept digit is odd
                let last_kept = kept.chars().last().unwrap();
                (last_kept as u8 - b'0') % 2 == 1
            } else {
                // More than half: round up
                true
            }
        };

        if round_up {
            Self::add_one_to_significand(kept)
        } else {
            let trimmed = kept.trim_start_matches('0');
            Ok(if trimmed.is_empty() {
                "0".to_string()
            } else {
                trimmed.to_string()
            })
        }
    }

    /// Add 1 to a significand string
    fn add_one_to_significand(sig: &str) -> Result<String, CompileError> {
        let mut bytes: Vec<u8> = sig.bytes().collect();
        let mut carry = 1u8;

        for i in (0..bytes.len()).rev() {
            if carry == 0 {
                break;
            }
            let digit = bytes[i] - b'0' + carry;
            if digit >= 10 {
                bytes[i] = b'0';
                carry = 1;
            } else {
                bytes[i] = digit + b'0';
                carry = 0;
            }
        }

        let mut result = String::from_utf8(bytes).unwrap();
        if carry > 0 {
            result = format!("1{}", result);
        }

        if result.len() > MAX_DIGITS {
            return Err(CompileError::TypeError(
                "Overflow: result too large".to_string(),
            ));
        }

        let trimmed = result.trim_start_matches('0');
        Ok(if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        })
    }

    /// Round to specified number of decimal places using round-half-even
    pub fn round(&self, decimal_places: u8) -> Result<Decimal, CompileError> {
        if decimal_places >= self.scale {
            // No rounding needed, but may need to add zeros
            return Ok(self.clone());
        }

        let digits_to_remove = (self.scale - decimal_places) as usize;
        let rounded_sig = Self::round_significand(&self.significand, digits_to_remove)?;

        Decimal::new(rounded_sig, self.negative, decimal_places)
    }

    /// Constant-time comparison (for security-sensitive comparisons)
    /// Returns true if equal, comparing all bytes regardless of early mismatch
    pub fn constant_time_eq(&self, other: &Decimal) -> bool {
        // Normalize to same scale first
        let (a_sig, b_sig, _) = Self::normalize_scales(self, other);

        // Constant-time comparison
        let mut result = 0u8;
        result |= (self.negative != other.negative) as u8;
        result |= (a_sig.len() != b_sig.len()) as u8;

        let max_len = a_sig.len().max(b_sig.len());
        let a_padded = format!("{:0>width$}", a_sig, width = max_len);
        let b_padded = format!("{:0>width$}", b_sig, width = max_len);

        for (a, b) in a_padded.bytes().zip(b_padded.bytes()) {
            result |= a ^ b;
        }

        result == 0
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut significand = self.significand.clone();
        let scale = self.scale as usize;

        // Insert decimal point
        if scale > 0 {
            if scale >= significand.len() {
                // Add leading zeros
                let zeros = "0".repeat(scale - significand.len() + 1);
                significand = format!("{}{}", zeros, significand);
            }
            let pos = significand.len() - scale;
            significand.insert(pos, '.');
            // Remove trailing zeros after decimal
            while significand.ends_with('0')
                && significand.chars().nth(significand.len() - 2) != Some('.')
            {
                significand.pop();
            }
            // Remove decimal point if no digits after
            if significand.ends_with('.') {
                significand.pop();
            }
        }

        if self.negative {
            write!(f, "-{}", significand)
        } else {
            write!(f, "{}", significand)
        }
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.constant_time_eq(other)
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Handle signs
        match (self.negative, other.negative) {
            (true, false) => {
                if self.is_zero() && other.is_zero() {
                    std::cmp::Ordering::Equal
                } else {
                    std::cmp::Ordering::Less
                }
            }
            (false, true) => {
                if self.is_zero() && other.is_zero() {
                    std::cmp::Ordering::Equal
                } else {
                    std::cmp::Ordering::Greater
                }
            }
            (neg, _) => {
                let (a_sig, b_sig, _) = Self::normalize_scales(self, other);
                let cmp = Self::compare_significands(&a_sig, &b_sig);
                let result = match cmp {
                    -1 => std::cmp::Ordering::Less,
                    0 => std::cmp::Ordering::Equal,
                    1 => std::cmp::Ordering::Greater,
                    _ => unreachable!(),
                };
                // Reverse if both negative
                if neg {
                    result.reverse()
                } else {
                    result
                }
            }
        }
    }
}

impl std::hash::Hash for Decimal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the canonical string representation for determinism
        self.to_string().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_from_str() {
        let d = Decimal::from_str("123.45").unwrap();
        assert_eq!(d.significand, "12345");
        assert_eq!(d.scale, 2);

        let d2 = Decimal::from_str("100").unwrap();
        assert_eq!(d2.significand, "100");
        assert_eq!(d2.scale, 0);

        // Edge cases
        let d3 = Decimal::from_str("0.5").unwrap();
        assert_eq!(d3.significand, "5");
        assert_eq!(d3.scale, 1);

        let d4 = Decimal::from_str("-42.5").unwrap();
        assert!(d4.negative);
        assert_eq!(d4.significand, "425");

        assert!(Decimal::from_str("00.1").is_err()); // leading zero
        assert!(Decimal::from_str("").is_err()); // empty
        assert!(Decimal::from_str("1.2.3").is_err()); // multiple dots
    }

    #[test]
    fn test_decimal_to_string() {
        let d = Decimal::from_str("123.45").unwrap();
        assert_eq!(d.to_string(), "123.45");

        let d2 = Decimal::from_str("100").unwrap();
        assert_eq!(d2.to_string(), "100");

        let d3 = Decimal::from_str("100.00").unwrap();
        assert_eq!(d3.to_string(), "100"); // Trailing zeros removed

        let d4 = Decimal::from_str("-42.5").unwrap();
        assert_eq!(d4.to_string(), "-42.5");

        let d5 = Decimal::from_str("0.005").unwrap();
        assert_eq!(d5.to_string(), "0.005");
    }

    #[test]
    fn test_decimal_add() {
        let d1 = Decimal::from_str("123.45").unwrap();
        let d2 = Decimal::from_str("100.00").unwrap();
        let sum = d1.add(&d2).unwrap();
        assert_eq!(sum.to_string(), "223.45");

        // Different scales
        let d3 = Decimal::from_str("1.5").unwrap();
        let d4 = Decimal::from_str("2.25").unwrap();
        let sum2 = d3.add(&d4).unwrap();
        assert_eq!(sum2.to_string(), "3.75");

        // Negative numbers
        let d5 = Decimal::from_str("-10").unwrap();
        let d6 = Decimal::from_str("15").unwrap();
        let sum3 = d5.add(&d6).unwrap();
        assert_eq!(sum3.to_string(), "5");

        let sum4 = d6.add(&d5).unwrap();
        assert_eq!(sum4.to_string(), "5");
    }

    #[test]
    fn test_decimal_sub() {
        let d1 = Decimal::from_str("100").unwrap();
        let d2 = Decimal::from_str("30.5").unwrap();
        let diff = d1.sub(&d2).unwrap();
        assert_eq!(diff.to_string(), "69.5");
    }

    #[test]
    fn test_decimal_mul() {
        let d1 = Decimal::from_str("12.5").unwrap();
        let d2 = Decimal::from_str("4").unwrap();
        let prod = d1.mul(&d2).unwrap();
        assert_eq!(prod.to_string(), "50");

        let d3 = Decimal::from_str("0.1").unwrap();
        let d4 = Decimal::from_str("0.2").unwrap();
        let prod2 = d3.mul(&d4).unwrap();
        assert_eq!(prod2.to_string(), "0.02");
    }

    #[test]
    fn test_decimal_div() {
        let d1 = Decimal::from_str("100").unwrap();
        let d2 = Decimal::from_str("4").unwrap();
        let quot = d1.div(&d2, 2).unwrap();
        assert_eq!(quot.to_string(), "25");

        let d3 = Decimal::from_str("10").unwrap();
        let d4 = Decimal::from_str("3").unwrap();
        let quot2 = d3.div(&d4, 4).unwrap();
        assert_eq!(quot2.to_string(), "3.3333");
    }

    #[test]
    fn test_decimal_round_half_even() {
        // Round-half-even (banker's rounding): ties go to nearest even

        // 2.5 rounds to 2 (ties to even)
        let d1 = Decimal::from_str("2.5").unwrap();
        let r1 = d1.round(0).unwrap();
        assert_eq!(r1.to_string(), "2");

        // 3.5 rounds to 4 (ties to even)
        let d2 = Decimal::from_str("3.5").unwrap();
        let r2 = d2.round(0).unwrap();
        assert_eq!(r2.to_string(), "4");

        // 2.4 rounds to 2 (less than half)
        let d3 = Decimal::from_str("2.4").unwrap();
        let r3 = d3.round(0).unwrap();
        assert_eq!(r3.to_string(), "2");

        // 2.6 rounds to 3 (more than half)
        let d4 = Decimal::from_str("2.6").unwrap();
        let r4 = d4.round(0).unwrap();
        assert_eq!(r4.to_string(), "3");

        // 2.55 rounds to 2.6 (more than half due to trailing 5)
        let d5 = Decimal::from_str("2.55").unwrap();
        let r5 = d5.round(1).unwrap();
        assert_eq!(r5.to_string(), "2.6");

        // 2.45 rounds to 2.4 (ties to even)
        let d6 = Decimal::from_str("2.45").unwrap();
        let r6 = d6.round(1).unwrap();
        assert_eq!(r6.to_string(), "2.4");

        // 2.450 rounds to 2.4 (exactly half, ties to even)
        let d7 = Decimal::from_str("2.450").unwrap();
        let r7 = d7.round(1).unwrap();
        assert_eq!(r7.to_string(), "2.4");

        // 2.451 rounds to 2.5 (more than half)
        let d8 = Decimal::from_str("2.451").unwrap();
        let r8 = d8.round(1).unwrap();
        assert_eq!(r8.to_string(), "2.5");
    }

    #[test]
    fn test_decimal_comparison() {
        let d1 = Decimal::from_str("10.5").unwrap();
        let d2 = Decimal::from_str("10.50").unwrap();
        let d3 = Decimal::from_str("10.6").unwrap();

        assert_eq!(d1, d2);
        assert!(d1 < d3);
        assert!(d3 > d1);

        let neg1 = Decimal::from_str("-5").unwrap();
        let neg2 = Decimal::from_str("-10").unwrap();
        assert!(neg1 > neg2);
    }

    #[test]
    fn test_constant_time_eq() {
        let d1 = Decimal::from_str("123.45").unwrap();
        let d2 = Decimal::from_str("123.450").unwrap();
        let d3 = Decimal::from_str("123.46").unwrap();

        assert!(d1.constant_time_eq(&d2));
        assert!(!d1.constant_time_eq(&d3));
    }

    #[test]
    fn test_acceptance_criteria_from_spec() {
        // From TASK.md acceptance criteria:

        // Parse and check internal representation
        let d1 = Decimal::from_str("123.45").unwrap();
        assert_eq!(d1.significand, "12345");
        assert_eq!(d1.scale, 2);

        // Addition
        let d2 = Decimal::from_str("100.00").unwrap();
        let sum = d1.add(&d2).unwrap();
        assert_eq!(sum.to_string(), "223.45");

        // Round-half-even test
        let r1 = Decimal::from_str("2.5").unwrap().round(0).unwrap();
        assert_eq!(r1.to_string(), "2"); // ties to even

        let r2 = Decimal::from_str("3.5").unwrap().round(0).unwrap();
        assert_eq!(r2.to_string(), "4"); // ties to even
    }
}
