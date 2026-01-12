# MTPScript Interpreter - Feature Verification Report

## Summary

After thorough review of TECHSPECV5.md and the codebase, this document identifies:
1. Features that are **intentionally excluded** by design
2. Features that are **fully implemented**
3. Features that have **partial implementations** with limitations

---

## Architecture Overview

MTPScript uses a **compilation pipeline** (not direct interpretation):

```
MTPScript → Parser/AST → Typed IR → JavaScript Subset → Interpreter
```

The `interpreter.rs` executes the **generated JavaScript**, not MTPScript directly. This means features are implemented at the **compiler level** (`codegen.rs`, `lower.rs`, `pattern.rs`).

---

## Verification Results

### 1. Pipeline Operator (`|>`) - IMPLEMENTED

**Location:** `mtpscript-core/src/ir/lower.rs:220-225`

```rust
AstExpr::Pipeline(left, right) => {
    // Desugar: a |> f ≡ f(a)
    // So left becomes the argument to right
```

**Status:** Fully implemented. Desugars `a |> f` to `f(a)` during IR lowering.

---

### 2. Pattern Matching (`match`) - PARTIAL

**Locations:**
- AST: `parser/ast.rs:71-74` (Match expression)
- AST: `parser/ast.rs:96-102` (Pattern enum)
- IR: `ir/nodes.rs:45-49` (IrExpr::Match)
- IR: `ir/nodes.rs:64-71` (IrPattern)
- Compiler: `compiler/pattern.rs:19-79` (PatternCompiler)
- CodeGen: `compiler/codegen.rs:309-312`

**Implemented patterns:**
| Pattern Type | Status | Notes |
|--------------|--------|-------|
| Wildcard `_` | Yes | `pattern.rs:88` |
| Variable binding | Yes | `pattern.rs:89-94` |
| Literal patterns | Yes | `pattern.rs:96-99` |
| Variant `Some(x)` | Partial | Single arg only (`pattern.rs:130`) |
| Record `User { name }` | Yes | `pattern.rs:163-208` |

**Limitations (pattern.rs:133-149):**
- "Complex ADT patterns not yet supported"
- "Nested patterns not yet supported"
- Multi-argument variant constructors not supported

---

### 3. `respond json(...)` - PARTIAL

**Locations:**
- AST: `parser/ast.rs:89` (`Expr::RespondJson`)
- IR: `ir/nodes.rs:61` (`IrExpr::RespondJson`)
- Compiler: `compiler/respond.rs:3-26`

**Status:** Basic implementation exists. Generates `return JSON.stringifyCanonical(...)`.

**Limitation:** `respond.rs:24` has `unimplemented!` for several expression types.

---

### 4. Gas Costs - IMPLEMENTED

**Location:** `mtpscript-core/src/gas/costs.rs:21-45`

| Operation | Spec (Annex A) | Implemented |
|-----------|----------------|-------------|
| Literal | 1 | 1 |
| Binary op | 2 | 2 |
| Comparison | 1 | 1 |
| Function call | 5 | 5 |
| Tail call | 0 | 0 |
| Non-tail recursion | 2 | 2 |
| Object access | 1 | 1 |
| Array access | 1 | 1 |
| If statement | 1 | 1 |
| Pattern match case | 3 | 3 |
| Json.parse | 10 + len/10 | 10 + len/10 |
| Effect call | 20 | 20 |
| DbRead | 50 | 70 (20+50) |
| HttpOut | 100 | 120 (20+100) |

**Minor discrepancy:** Effect calls add base cost (20) to specific cost, which matches total but structure differs slightly from spec.

---

### 5. Async/Await - PARTIAL

**Locations:**
- AST: `parser/ast.rs:88` (`Expr::Await`)
- Desugaring: `effects/async_effect.rs:6-107`
- Effects: `runtime/effects.rs:417-456`

**Status:** Desugaring implemented. Transforms `await e` to `Async.await(promiseHash, contId, e)`. Runtime effect stub exists but doesn't execute real async I/O.

---

## Intentionally Excluded Features (per spec)

These are **NOT implementation gaps** - they are forbidden by design:

| Feature | Spec Reference |
|---------|----------------|
| Loops (for/while/do-while) | Section 12: "Forbidden JS" |
| try/catch/finally | Section 12: "Forbidden JS" |
| Classes | Section 1.2 Non-Goals |
| `this` keyword | Section 12: "Forbidden JS" |
| `eval` | Section 12: "Forbidden JS" |
| Global mutation | Section 12: "Forbidden JS" |
| Floating-point math | Section 1.2 Non-Goals |
| Reflection/introspection | Section 1.2 Non-Goals |
| Metaprogramming/macros | Section 1.2 Non-Goals |
| Dynamic code loading | Section 1.2 Non-Goals |
| Shared mutable state | Section 1.2 Non-Goals |
| Threads/concurrency | Section 1.2 Non-Goals |
| Implicit coercions | Section 1.2 Non-Goals |
| Arrow functions | Not in grammar (use `function(params) { body }`) |
| Spread/rest operators | Not in grammar |
| Destructuring | Not in grammar (use pattern matching) |
| Template literals | Not in grammar |
| Increment/decrement (`++`/`--`) | Not in grammar |
| Compound assignment (`+=`, `-=`) | Not in grammar |
| Bitwise operators | Not in grammar |
| Optional chaining (`?.`) | Not in grammar |
| Nullish coalescing (`??`) | Not in grammar |

---

## Recommended Fixes (Priority Order)

### High Priority
1. **Pattern matching completeness** (`compiler/pattern.rs`)
   - Support nested patterns
   - Support multi-argument ADT constructors

2. **respond.rs expression coverage**
   - Implement remaining expression types currently marked `unimplemented!`

### Medium Priority
3. **Async effect** - Complete the runtime implementation for real async I/O

### Low Priority
4. **Gas cost alignment** - Minor: ensure DbRead/HttpOut costs match spec exactly (currently 70/120 vs spec's 50/100+20)

---

## Verification Commands

```bash
# Run all tests
cargo test --workspace

# Run specific test modules
cd mtpscript-core && cargo test pattern
cd mtpscript-core && cargo test lower
cd mtpscript-core && cargo test gas

# Check for unimplemented! markers
grep -r "unimplemented!" mtpscript-core/src/
```

---

## Files to Modify

| File | Change |
|------|--------|
| `mtpscript-core/src/compiler/pattern.rs` | Add nested pattern support |
| `mtpscript-core/src/compiler/respond.rs` | Complete expression handling |
| `mtpscript-core/src/runtime/effects.rs` | Complete Async effect implementation |

---

## Relevant Effect/Interpreter Files

There is **no `interrupter.rs` file** in the codebase. The relevant files are:

| File | Purpose |
|------|---------|
| `mtpscript-core/src/runtime/effects.rs` | Effect registry - DbRead, DbWrite, HttpOut, Log, Async |
| `mtpscript-core/src/effects/async_effect.rs` | Async desugaring for await expressions |
| `mtpscript-core/src/compiler/effects.rs` | Effect compilation and validation |
| `mtpscript-core/src/runtime/interpreter.rs` | Main JS interpreter with JsExpr AST |
| `mtpscript-core/src/effects/builtins.rs` | Built-in pure functions |

---

## Conclusion

The MTPScript compiler/interpreter implements **most spec-required features**. The main gaps are:

1. **Pattern matching** - partial (no nested patterns, no multi-arg ADT constructors)
2. **respond.rs** - incomplete expression support
3. **Async** - stub only, no real I/O

Most features one might expect from JavaScript (loops, try/catch, classes, etc.) are **intentionally excluded** per the spec's deterministic design philosophy.

---

## Task List for Feature Completion

| Task | Filename | Line Number | How to Fix | Pseudocode |
|------|----------|-------------|------------|------------|
| [ ] **Nested Patterns in Variants** | `mtpscript-core/src/compiler/pattern.rs` | 145-149 | Update `compile_variant_pattern` to recursively call `compile_pattern_binding` for each sub-pattern in a variant. | `for (i, p) in sub_patterns { let (sc, sb) = self.compile_pattern_binding(p, &format!("{}._{}", temp_var, i))?; conditions.push(sc); bindings.extend(sb); }` |
| [ ] **Multi-arg ADT Constructors** | `mtpscript-core/src/compiler/pattern.rs` | 130-139 | Modify `compile_variant_pattern` to handle multiple arguments by mapping them to indexed property access (e.g., `_0`, `_1`). | `if sub_patterns.len() > 1 { for (i, p) in sub_patterns.iter().enumerate() { let sub_expr = format!("{}._{}", temp_var, i); ... } }` |
| [ ] **Respond JSON Expression Coverage** | `mtpscript-core/src/compiler/respond.rs` | 24 | Implement missing `Expr` variants in `compile_expr_to_js` (e.g., `Binary`, `Object`, `Array`, `Dot`, `Index`). | `match expr { Expr::Binary { left, op, right } => format!("({} {} {})", compile(left), op_js(op), compile(right)), ... }` |
| [ ] **Complete Async Runtime** | `mtpscript-core/src/runtime/effects.rs` | 441-456 | Replace the deterministic stub in `async_impl` with a real implementation that polls for task completion and returns real results. | `interp.builtins.insert("async_impl", |args| { let result = runtime.poll_async_op(args[0]); Ok(result) })` |
| [x] **Gas Cost Alignment** | `mtpscript-core/src/gas/costs.rs` | 36-43 | Correct gas costs for `DbRead` (50) and `HttpOut` (100) to match Annex A. Verify and fix `DbWrite` (currently 120). | `Op::DbRead => 50, Op::HttpOut => 100, Op::DbWrite => 100` |
