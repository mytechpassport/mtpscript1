use mtpscript_core::gas::{
    costs::{gas_cost, Op},
    counter::GasCounter,
};

#[test]
fn test_literal_cost() {
    assert_eq!(gas_cost(Op::Literal), 1);
    assert_eq!(gas_cost(Op::LiteralString), 1);
}

#[test]
fn test_binop_cost() {
    assert_eq!(gas_cost(Op::BinaryOp), 2);
}

#[test]
fn test_comparison_cost() {
    assert_eq!(gas_cost(Op::Comparison), 1);
}

#[test]
fn test_function_call_cost() {
    assert_eq!(gas_cost(Op::FunctionCall), 5);
}

#[test]
fn test_tail_call_free() {
    assert_eq!(gas_cost(Op::TailCall), 0);
}

#[test]
fn test_non_tail_recursion_cost() {
    assert_eq!(gas_cost(Op::NonTailRecursion), 2);
}

#[test]
fn test_object_array_access_cost() {
    assert_eq!(gas_cost(Op::ObjectAccess), 1);
    assert_eq!(gas_cost(Op::ArrayAccess), 1);
}

#[test]
fn test_if_statement_cost() {
    assert_eq!(gas_cost(Op::IfStatement), 1);
}

#[test]
fn test_pattern_match_cost() {
    assert_eq!(gas_cost(Op::PatternMatchCase), 3);
}

#[test]
fn test_json_parse_cost() {
    assert_eq!(gas_cost(Op::JsonParse(0)), 10);
    assert_eq!(gas_cost(Op::JsonParse(10)), 10 + 1); // 10 + 10/10 = 11
    assert_eq!(gas_cost(Op::JsonParse(100)), 10 + 10); // 10 + 100/10 = 20
}

#[test]
fn test_effect_call_cost() {
    assert_eq!(gas_cost(Op::EffectCall), 20);
    // DbRead/DbWrite/HttpOut have standalone costs (not additive with EffectCall)
    assert_eq!(gas_cost(Op::DbRead), 50);
    assert_eq!(gas_cost(Op::DbWrite), 100); // Write is more expensive than read
    assert_eq!(gas_cost(Op::HttpOut), 100);
}

#[test]
fn test_gas_counter_initialization() {
    let counter = GasCounter::new(1000);
    assert_eq!(counter.remaining(), 1000);
    assert_eq!(counter.used(), 0);
    assert!(!counter.is_exhausted());
    assert!(counter.error().is_none());
}

#[test]
fn test_gas_counter_consumption() {
    let mut counter = GasCounter::new(100);

    // Normal consumption
    assert!(counter.consume(50).is_ok());
    assert_eq!(counter.remaining(), 50);
    assert_eq!(counter.used(), 50);

    // More consumption
    assert!(counter.consume(30).is_ok());
    assert_eq!(counter.remaining(), 20);
    assert_eq!(counter.used(), 80);
}

#[test]
fn test_gas_exhaustion() {
    let mut counter = GasCounter::new(100);

    // Consume all gas
    assert!(counter.consume(100).is_ok());
    assert_eq!(counter.remaining(), 0);
    assert_eq!(counter.used(), 100);
    assert!(!counter.is_exhausted());

    // Try to consume more - should fail
    assert!(counter.consume(1).is_err());
    assert!(counter.is_exhausted());
    assert!(counter.error().is_some());
}

#[test]
fn test_gas_counter_from_env() {
    // Test with valid limit
    std::env::set_var("MTP_GAS_LIMIT", "50000");
    let counter = GasCounter::from_env().unwrap();
    assert_eq!(counter.remaining(), 50000);

    // Test with invalid limit (too high)
    std::env::set_var("MTP_GAS_LIMIT", "3000000000");
    assert!(GasCounter::from_env().is_err());

    // Test with invalid limit (zero)
    std::env::set_var("MTP_GAS_LIMIT", "0");
    assert!(GasCounter::from_env().is_err());

    // Test with invalid format
    std::env::set_var("MTP_GAS_LIMIT", "not-a-number");
    assert!(GasCounter::from_env().is_err());

    // Clean up
    std::env::remove_var("MTP_GAS_LIMIT");
}
