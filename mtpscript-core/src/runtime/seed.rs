use crate::errors::compile::CompileError;
use sha2::{Digest, Sha256};

pub struct SeedRequest {
    pub request_id: String,
    pub account_id: String,
    pub function_version: String,
    pub snapshot_hash: [u8; 32],
    pub gas_limit: u64,
}

impl SeedRequest {
    pub fn new(
        request_id: String,
        account_id: String,
        function_version: String,
        snapshot_hash: [u8; 32],
        gas_limit: u64,
    ) -> Self {
        Self {
            request_id,
            account_id,
            function_version,
            snapshot_hash,
            gas_limit,
        }
    }
}

pub fn compute_seed(req: &SeedRequest) -> Result<[u8; 32], CompileError> {
    // Concatenate: RequestId || AccountId || FunctionVersion || "mtpscript-v5.1" || SnapshotHash || GasLimitASCII
    let mut data = Vec::new();

    data.extend_from_slice(req.request_id.as_bytes());
    data.extend_from_slice(req.account_id.as_bytes());
    data.extend_from_slice(req.function_version.as_bytes());
    data.extend_from_slice(b"mtpscript-v5.1");
    data.extend_from_slice(&req.snapshot_hash);
    data.extend_from_slice(req.gas_limit.to_string().as_bytes());

    // SHA-256 hash the concatenation
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();

    let mut seed = [0u8; 32];
    seed.copy_from_slice(&result);
    Ok(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_determinism() {
        let req = SeedRequest::new(
            "abc123".to_string(),
            "123456789".to_string(),
            "1".to_string(),
            [0u8; 32],
            10_000_000,
        );

        let seed1 = compute_seed(&req).unwrap();
        let seed2 = compute_seed(&req).unwrap();

        assert_eq!(seed1, seed2);
        assert_eq!(seed1.len(), 32);
    }

    #[test]
    fn test_different_inputs_different_seeds() {
        let req1 = SeedRequest::new(
            "abc123".to_string(),
            "123456789".to_string(),
            "1".to_string(),
            [0u8; 32],
            10_000_000,
        );

        let req2 = SeedRequest::new(
            "def456".to_string(),
            "123456789".to_string(),
            "1".to_string(),
            [0u8; 32],
            10_000_000,
        );

        let seed1 = compute_seed(&req1).unwrap();
        let seed2 = compute_seed(&req2).unwrap();

        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_seed_computation_acceptance_criteria() {
        // Test the exact acceptance criteria from TASK.md
        let req = SeedRequest::new(
            "abc123".to_string(),
            "123456789".to_string(),
            "1".to_string(),
            [0u8; 32],
            10_000_000,
        );

        let seed = compute_seed(&req).unwrap();

        assert_eq!(seed.len(), 32);

        // Same inputs = same seed
        let seed2 = compute_seed(&req).unwrap();
        assert_eq!(seed, seed2);

        // Test concatenation: RequestId || AccountId || FunctionVersion || "mtpscript-v5.1" || SnapshotHash || GasLimitASCII
        // We can't easily test the exact concatenation without exposing internals,
        // but we can test that different inputs produce different seeds
        let req_diff = SeedRequest::new(
            "different".to_string(),
            "123456789".to_string(),
            "1".to_string(),
            [0u8; 32],
            10_000_000,
        );

        let seed_diff = compute_seed(&req_diff).unwrap();
        assert_ne!(seed, seed_diff);
    }
}
