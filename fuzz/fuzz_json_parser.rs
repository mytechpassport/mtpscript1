use mtpscript_core::json::Json;

#[cfg(fuzzing)]
fuzz_target!(|data: &[u8]| {
    // Try to convert to UTF-8 string
    if let Ok(input) = std::str::from_utf8(data) {
        // Attempt to parse as JSON - should not crash
        let _ = Json::parse(input);
    }
});
