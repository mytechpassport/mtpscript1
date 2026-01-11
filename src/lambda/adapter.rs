use std::env;

pub fn read_gas_limit() -> Result<u64, String> {
    let gas_limit_str = env::var("MTP_GAS_LIMIT").unwrap_or("10000000".to_string());
    let gas_limit: u64 = gas_limit_str.parse().map_err(|_| "Invalid gas limit")?;
    if gas_limit < 1 || gas_limit > 2_000_000_000 {
        return Err("Gas limit out of range".to_string());
    }
    Ok(gas_limit)
}
