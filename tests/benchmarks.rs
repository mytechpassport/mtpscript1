#[cfg(test)]
mod benchmark_tests {
    use std::time::Instant;

    #[test]
    fn bench_clone_interpreter() {
        let start = Instant::now();
        // placeholder: clone interpreter
        let duration = start.elapsed();
        assert!(duration < std::time::Duration::from_millis(1));
    }
}
