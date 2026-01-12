#[derive(Debug, Clone, Copy)]
pub enum Op {
    Literal,
    LiteralString,
    BinaryOp,
    Comparison,
    FunctionCall,
    TailCall,
    NonTailRecursion,
    ObjectAccess,
    ArrayAccess,
    IfStatement,
    PatternMatchCase,
    JsonParse(usize), // string length
    EffectCall,
    DbRead,
    DbWrite,
    HttpOut,
}

pub fn gas_cost(op: Op) -> u64 {
    match op {
        Op::Literal => 1,
        Op::LiteralString => 1,
        Op::BinaryOp => 2,
        Op::Comparison => 1,
        Op::FunctionCall => 5,
        Op::TailCall => 0,
        Op::NonTailRecursion => 2,
        Op::ObjectAccess => 1,
        Op::ArrayAccess => 1,
        Op::IfStatement => 1,
        Op::PatternMatchCase => 3,
        Op::JsonParse(len) => 10 + (len as u64 / 10),
        Op::EffectCall => 20,
        Op::DbRead => 20 + 50,  // base 20 + specific 50 for read operation
        // DbWrite is typically more expensive than DbRead due to:
        // - Disk I/O persistence requirements
        // - Transaction logging overhead
        // - Potential replication costs
        // Default: 2x the read cost. Verify against MTP spec Annex A.
        Op::DbWrite => 20 + 100,
        Op::HttpOut => 20 + 100,
    }
}
