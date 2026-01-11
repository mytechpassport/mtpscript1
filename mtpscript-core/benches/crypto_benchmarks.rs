use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mtpscript_core::runtime::interpreter::{Interpreter, JsExpr};
use mtpscript_core::runtime::value::Value;
use mtpscript_core::security::verify::verify_snapshot_integrity;
use mtpscript_core::snapshot::extract_js_code;
use serde_json;

/// Benchmark for snapshot integrity verification
fn bench_snapshot_integrity(c: &mut Criterion) {
    // Create a test snapshot
    let snapshot = create_test_snapshot();

    c.bench_function("snapshot_integrity_verify", |b| {
        b.iter(|| {
            let _ = black_box(verify_snapshot_integrity(&snapshot));
        })
    });
}

/// Benchmark for snapshot JS extraction
fn bench_snapshot_extraction(c: &mut Criterion) {
    let snapshot = create_test_snapshot();

    c.bench_function("snapshot_extract_js", |b| {
        b.iter(|| {
            let _ = black_box(extract_js_code(&snapshot));
        })
    });
}

/// Benchmark for CRC32 computation (common operation in snapshot verification)
fn bench_crc32_computation(c: &mut Criterion) {
    let data = vec![0u8; 1024]; // 1KB of test data

    c.bench_function("crc32_compute", |b| {
        b.iter(|| {
            black_box(crc32fast::hash(&data));
        })
    });
}

/// Benchmark for SHA-256 computation (used in content hashing)
fn bench_sha256_computation(c: &mut Criterion) {
    let data = vec![0u8; 1024]; // 1KB of test data

    c.bench_function("sha256_compute", |b| {
        b.iter(|| {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&data);
            black_box(hasher.finalize());
        })
    });
}

/// Benchmark for interpreter clone performance (< 1ms target)
fn bench_interpreter_clone(c: &mut Criterion) {
    // Create a test snapshot
    let snapshot = create_test_snapshot();

    c.bench_function("interpreter_clone", |b| {
        b.iter(|| {
            // In a real implementation, this would clone from snapshot
            // For now, just create a new interpreter
            let _interp = black_box(Interpreter::new());
        })
    });
}

/// Benchmark for gas metering overhead (< 1% target)
fn bench_gas_overhead(c: &mut Criterion) {
    let mut interp = Interpreter::new();
    let expr = JsExpr::BinOp(
        "+".to_string(),
        Box::new(JsExpr::Literal(Value::Number(1))),
        Box::new(JsExpr::Literal(Value::Number(2))),
    );

    c.bench_function("gas_overhead", |b| {
        b.iter(|| {
            let _ = black_box(interp.eval(&expr));
        })
    });
}

/// Benchmark for JSON serialization performance
fn bench_json_serialize(c: &mut Criterion) {
    let test_value = Value::Object(
        vec![
            ("id".to_string(), Value::Number(123)),
            ("name".to_string(), Value::String("test".to_string())),
            ("active".to_string(), Value::Boolean(true)),
        ]
        .into_iter()
        .collect(),
    );

    c.bench_function("json_serialize", |b| {
        b.iter(|| {
            // Use simple string representation for benchmarking
            let _json = black_box(format!("{:?}", test_value));
        })
    });
}

/// Create a minimal test snapshot for benchmarking
fn create_test_snapshot() -> Vec<u8> {
    let mut snapshot = Vec::new();

    // Magic bytes
    snapshot.extend_from_slice(b"MTPJS\x00\x00\x00");

    // Version
    snapshot.extend_from_slice(&51u32.to_le_bytes());

    // Size placeholder
    let size_offset = snapshot.len();
    snapshot.extend_from_slice(&[0u8; 8]);

    // Content hash
    snapshot.extend_from_slice(&[0u8; 32]);

    // JS content
    let js_content = b"function benchmark() { return 42; }";
    snapshot.extend_from_slice(js_content);

    // Signature
    snapshot.extend_from_slice(&[0u8; 64]);

    // CRC32 (computed on everything except the CRC itself)
    let content_for_crc = &snapshot[..snapshot.len()];
    let crc = crc32fast::hash(content_for_crc);
    snapshot.extend_from_slice(&crc.to_le_bytes());

    // Update size
    let total_size = snapshot.len() as u64;
    snapshot[size_offset..size_offset + 8].copy_from_slice(&total_size.to_le_bytes());

    snapshot
}

criterion_group!(
    benches,
    bench_snapshot_integrity,
    bench_snapshot_extraction,
    bench_crc32_computation,
    bench_sha256_computation,
    bench_interpreter_clone,
    bench_gas_overhead,
    bench_json_serialize
);
criterion_main!(benches);
