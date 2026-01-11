pub mod clone;
pub mod effects;
pub mod interpreter;
pub mod js_parser;
pub mod seed;
pub mod value;
pub mod wipe;

pub use clone::clone_interpreter;
pub use effects::inject_effects;
pub use interpreter::Interpreter;
pub use js_parser::parse_js_program;
pub use seed::{compute_seed, SeedRequest};
pub use value::Value;
pub use wipe::*;

/// Get gas limit from MTP_GAS_LIMIT environment variable, defaulting to 10M
pub fn get_gas_limit() -> u64 {
    std::env::var("MTP_GAS_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gas_limit_from_env() {
        std::env::set_var("MTP_GAS_LIMIT", "5000000");
        assert_eq!(get_gas_limit(), 5_000_000);
        std::env::remove_var("MTP_GAS_LIMIT");
    }
}
