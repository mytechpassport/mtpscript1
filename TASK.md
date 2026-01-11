# MTPScript Implementation Task List

**Based on:** TECHSPECV5.md (Version 5.1)
**Generated:** 2026-01-10
**Total Tasks:** 72

---

## Effort Legend

| Size | Effort | Description |
|------|--------|-------------|
| **S** | 1-2 hours | Simple, well-defined, minimal dependencies |
| **M** | 2-8 hours | Moderate complexity, some design decisions |
| **L** | 1-3 days | Significant work, multiple components |
| **XL** | 3+ days | Complex, core system, high integration |

---

## Section 1: Project Scaffold & Structure

### - [ ] MTP-001: Initialize Rust Project Structure
**Effort:** S | **Files:** `Cargo.toml`, `src/lib.rs`, `src/main.rs`
**Spec Lines:** 625-636 (§27.1 Architecture Overview)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a new project directory
WHEN cargo build is executed
THEN it should compile successfully with the following crate structure:
  - mtpscript (binary) - CLI tool
  - mtpscript-core (library) - shared components

Pseudo-test:
  assert(run("cargo build").exit_code == 0)
  assert(exists("target/debug/mtpscript"))
```

**Directory Structure:**
```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── lexer/               # Tokenization
├── parser/              # AST generation
├── types/               # Type system
├── ir/                  # Intermediate representation
├── effects/             # Effect system
├── compiler/            # JS codegen
├── runtime/             # MTPJS interpreter
├── snapshot/            # .msqs format
├── gas/                 # Gas metering
├── json/                # Canonical JSON
├── api/                 # HTTP/OpenAPI
├── security/            # Signatures, wipe
└── errors/              # Error types
```

---

### - [ ] MTP-002: Define Core Error Types
**Effort:** S | **Files:** `src/errors/mod.rs`, `src/errors/compile.rs`, `src/errors/runtime.rs`
**Spec Lines:** 525-529 (§16 Error System)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN the error module
WHEN any component encounters an error
THEN it should return a typed error with:
  - Deterministic error code
  - No stack traces in production mode
  - Canonical JSON serialization

Pseudo-test:
  let err = MtpError::GasExhausted { gas_limit: 1000, gas_used: 1001 };
  assert(err.to_json() == r#"{"error":"GasExhausted","gasLimit":1000,"gasUsed":1001}"#)
  assert(err.stack_trace().is_none()) // in prod mode
```

---

### - [ ] MTP-003: Create CLI Interface
**Effort:** M | **Files:** `src/main.rs`, `src/cli/mod.rs`, `src/cli/commands.rs`
**Spec Lines:** 481-493 (§12 Compilation Pipeline)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN the mtpscript CLI
WHEN invoked with subcommands
THEN it should support:
  - mtp compile <input.mtp> -o <output.js>
  - mtp snapshot <input.js> -o <output.msqs>
  - mtp serve --port 8080
  - mtp run <input.msqs>

Pseudo-test:
  assert(run("mtp --help").stdout.contains("compile"))
  assert(run("mtp compile test.mtp -o test.js").exit_code == 0)
```

---

## Section 2: Lexer

### - [ ] MTP-010: Implement Lexer/Tokenizer
**Effort:** M | **Files:** `src/lexer/mod.rs`, `src/lexer/token.rs`, `src/lexer/scanner.rs`
**Spec Lines:** 153-237 (§3 Syntax & Grammar)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN MTPScript source code
WHEN the lexer processes it
THEN it should produce a stream of tokens:
  - Keywords: function, type, api, const, if, else, match, await, uses, import, respond
  - Operators: |>, +, -, *, /, ==, !=, <, >, <=, >=, &&, ||, !, .
  - Delimiters: (, ), {, }, [, ], ,, :, ;, =>
  - Literals: numbers, strings, booleans
  - Identifiers: [a-zA-Z_][a-zA-Z0-9_]*
  - HTTP methods: GET, POST, PUT, DELETE, PATCH

Pseudo-test:
  let tokens = lex("function foo(x: number) { x + 1 }");
  assert(tokens == [
    Token::Function, Token::Ident("foo"), Token::LParen,
    Token::Ident("x"), Token::Colon, Token::Ident("number"),
    Token::RParen, Token::LBrace, Token::Ident("x"),
    Token::Plus, Token::Number(1), Token::RBrace
  ])
```

---

### - [ ] MTP-011: Implement String Literal Parsing
**Effort:** S | **Files:** `src/lexer/scanner.rs`
**Spec Lines:** 233 (string_literal definition)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a string literal in source
WHEN the lexer processes it
THEN it should:
  - Handle escape sequences (\n, \t, \\, \")
  - Preserve UTF-8 content
  - Reject unterminated strings

Pseudo-test:
  assert(lex(r#""hello\nworld""#) == [Token::String("hello\nworld")])
  assert(lex(r#""unterminated"#).is_err())
```

---

### - [ ] MTP-012: Implement Number Literal Parsing
**Effort:** S | **Files:** `src/lexer/scanner.rs`
**Spec Lines:** 232 (number_literal definition)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a number literal in source
WHEN the lexer processes it
THEN it should:
  - Parse integers: 42, 0, -17
  - Parse decimals: 3.14, 0.5
  - Reject invalid formats: .5, 1., 1.2.3

Pseudo-test:
  assert(lex("42") == [Token::Number(42)])
  assert(lex("3.14") == [Token::Decimal("3.14")])
  assert(lex(".5").is_err())
```

---

## Section 3: Parser

### - [ ] MTP-020: Implement AST Data Structures
**Effort:** M | **Files:** `src/parser/ast.rs`
**Spec Lines:** 159-236 (Full EBNF Grammar)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN the grammar specification
WHEN defining AST nodes
THEN it should include:
  - Program: Vec<ModuleDecl>
  - ModuleDecl: Import | TypeDecl | FuncDecl | ApiDecl
  - TypeDecl: Record { fields } | Adt { variants }
  - FuncDecl: { name, params, effects, body }
  - ApiDecl: { method, path, effects, body }
  - Expr: all expression variants from grammar
  - Pattern: all pattern variants

Pseudo-test:
  let ast = Program { decls: vec![
    ModuleDecl::Func(FuncDecl {
      name: "add".into(),
      params: vec![("a", Type::Number), ("b", Type::Number)],
      effects: vec![],
      body: Expr::BinOp("+", box Expr::Ident("a"), box Expr::Ident("b"))
    })
  ]};
  assert(ast.validate().is_ok())
```

---

### - [ ] MTP-021: Implement Recursive Descent Parser
**Effort:** L | **Files:** `src/parser/mod.rs`, `src/parser/parser.rs`
**Spec Lines:** 159-236 (Full EBNF Grammar)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a token stream from the lexer
WHEN the parser processes it
THEN it should produce a valid AST or error with location

Pseudo-test:
  let src = r#"
    type User { id: number, name: string }
    function greet(u: User) { "Hello, " + u.name }
  "#;
  let ast = parse(src)?;
  assert(ast.decls.len() == 2)
  assert(ast.decls[0].is_type_decl())
  assert(ast.decls[1].is_func_decl())
```

---

### - [ ] MTP-022: Implement Expression Parser with Precedence
**Effort:** M | **Files:** `src/parser/expr.rs`
**Spec Lines:** 199-227 (expr rules), 239-242 (Pipeline Associativity)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN expressions with operators
WHEN parsed
THEN precedence should be:
  1. () grouping (highest)
  2. . [] member access
  3. ! - unary
  4. * / multiplication
  5. + - addition
  6. < > <= >= == != comparison
  7. && logical and
  8. || logical or
  9. |> pipeline (left-associative, lowest)

Pseudo-test:
  let expr = parse_expr("a + b * c");
  assert(expr == BinOp("+", Ident("a"), BinOp("*", Ident("b"), Ident("c"))))

  let pipe = parse_expr("a |> b |> c");
  assert(pipe == Pipeline(Pipeline(Ident("a"), Ident("b")), Ident("c")))
```

---

### - [ ] MTP-023: Implement Pattern Matching Parser
**Effort:** M | **Files:** `src/parser/pattern.rs`
**Spec Lines:** 218-222 (case, pattern rules)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a match expression
WHEN parsed
THEN it should handle:
  - Wildcard: _
  - Identifier binding: x
  - Literals: 42, "hello", true
  - Variant destructuring: Some(x), Ok(value)
  - Record destructuring: User { name: n, id: i }

Pseudo-test:
  let src = r#"match result { Ok(v) => v, Err(e) => 0 }"#;
  let ast = parse_expr(src)?;
  assert(ast.cases.len() == 2)
  assert(ast.cases[0].pattern == Pattern::Variant("Ok", vec![Pattern::Bind("v")]))
```

---

### - [ ] MTP-024: Implement Type Declaration Parser
**Effort:** M | **Files:** `src/parser/type_decl.rs`
**Spec Lines:** 167-180 (type_decl, field_decl, variant_decl, type_expr)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN type declarations
WHEN parsed
THEN it should handle:
  - Records: type User { id: number, name: string }
  - ADTs: type Option<T> = Some(T) | None
  - Generic types: List<T>, Map<K, V>, Result<T, E>

Pseudo-test:
  let src = "type Result<T, E> = Ok(T) | Err(E)";
  let decl = parse_type_decl(src)?;
  assert(decl.name == "Result")
  assert(decl.type_params == ["T", "E"])
  assert(decl.variants.len() == 2)
```

---

### - [ ] MTP-025: Implement API Declaration Parser
**Effort:** S | **Files:** `src/parser/api_decl.rs`
**Spec Lines:** 192-197 (api_decl, http_method)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN an API declaration
WHEN parsed
THEN it should capture:
  - HTTP method (GET, POST, PUT, DELETE, PATCH)
  - Path string
  - Effects list
  - Body expression

Pseudo-test:
  let src = r#"
    api POST /users
    uses { DbWrite, Log } {
      respond json({ "created": true })
    }
  "#;
  let api = parse_api_decl(src)?;
  assert(api.method == HttpMethod::POST)
  assert(api.path == "/users")
  assert(api.effects == ["DbWrite", "Log"])
```

---

## Section 4: Type System

### - [ ] MTP-030: Implement Primitive Types
**Effort:** S | **Files:** `src/types/primitives.rs`, `src/types/mod.rs`
**Spec Lines:** 253-259 (§4.1 Primitive Types)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN primitive type definitions
WHEN used in the type system
THEN it should support:
  - number: Signed 64-bit with checked overflow
  - boolean: true/false only
  - string: Immutable UTF-8
  - Decimal: Fixed-point per §4-a

Pseudo-test:
  assert(Type::Number.is_primitive())
  assert(Type::Number.size_bits() == 64)
  assert(Type::Boolean.values() == [true, false])
```

---

### - [ ] MTP-031: Implement Decimal Type
**Effort:** L | **Files:** `src/types/decimal.rs`
**Spec Lines:** 292-305 (§4-a Decimal / Money)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN the Decimal type
WHEN performing operations
THEN it should:
  - Store value as string (1-34 digits, no leading zeros)
  - Store scale as number (0 ≤ scale ≤ 28)
  - Use round-half-even (banker's rounding)
  - Return Result<Decimal, Overflow> for operations
  - Compare in constant time
  - Serialize to shortest canonical string

Pseudo-test:
  let d1 = Decimal::from_str("123.45")?;
  assert(d1.value == "12345")
  assert(d1.scale == 2)

  let d2 = Decimal::from_str("100.00")?;
  let sum = d1.add(d2)?;
  assert(sum.to_string() == "223.45")

  // Round-half-even test
  let r1 = Decimal::from_str("2.5")?.round(0)?;
  assert(r1.to_string() == "2") // ties to even
  let r2 = Decimal::from_str("3.5")?.round(0)?;
  assert(r2.to_string() == "4") // ties to even
```

---

### - [ ] MTP-032: Implement Composite Types (Records)
**Effort:** M | **Files:** `src/types/record.rs`
**Spec Lines:** 261-271 (Records in §4.2)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a record type definition
WHEN type-checking
THEN it should:
  - Validate all field types
  - Support field access via dot notation
  - Enforce immutability
  - Generate structural equality

Pseudo-test:
  let user_type = RecordType {
    name: "User",
    fields: [("id", Type::Number), ("name", Type::String)]
  };
  let expr = parse_expr("user.name")?;
  assert(typecheck(expr, &[("user", user_type)])? == Type::String)
```

---

### - [ ] MTP-033: Implement Algebraic Data Types (ADTs)
**Effort:** L | **Files:** `src/types/adt.rs`
**Spec Lines:** 273-285 (ADTs in §4.2), 600-606 (§24 Union Exhaustiveness)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN an ADT definition
WHEN used in the type system
THEN it should:
  - Define variants with optional payloads
  - Support generic type parameters
  - Compute content-hash of variant list
  - Enforce exhaustive pattern matching at link time

Pseudo-test:
  let result_type = AdtType {
    name: "Result",
    params: ["T", "E"],
    variants: [
      Variant { name: "Ok", payload: Some(TypeVar("T")) },
      Variant { name: "Err", payload: Some(TypeVar("E")) },
    ]
  };
  assert(result_type.content_hash() == sha256("Ok(T)|Err(E)"))

  // Exhaustiveness check
  let match_expr = parse_expr("match r { Ok(v) => v }")?;
  assert(check_exhaustive(match_expr, result_type).is_err()) // missing Err case
```

---

### - [ ] MTP-034: Implement Option and Result Built-in Types
**Effort:** S | **Files:** `src/types/builtins.rs`
**Spec Lines:** 275-277, 287-288 (Option/Result, No null/undefined)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN the built-in Option and Result types
WHEN used in programs
THEN they should be:
  - Pre-defined: Option<T> = Some(T) | None
  - Pre-defined: Result<T, E> = Ok(T) | Err(E)
  - Used instead of null/undefined

Pseudo-test:
  let ctx = TypeContext::with_builtins();
  assert(ctx.lookup("Option").is_some())
  assert(ctx.lookup("Result").is_some())

  let src = "const x: Option<number> = Some(42)";
  assert(typecheck(parse(src)?, ctx).is_ok())
```

---

### - [ ] MTP-035: Implement Type Checker
**Effort:** XL | **Files:** `src/types/checker.rs`, `src/types/unify.rs`
**Spec Lines:** 251-291 (§4 Type System)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN an AST
WHEN type-checking
THEN it should:
  - Infer types for all expressions
  - Unify generic type parameters
  - Report type mismatches with locations
  - Validate effect declarations

Pseudo-test:
  let src = r#"
    function add(a: number, b: number): number {
      a + b
    }
    const x: string = add(1, 2)  // type error
  "#;
  let result = typecheck(parse(src)?);
  assert(result.is_err())
  assert(result.err().message.contains("expected string, got number"))
```

---

### - [ ] MTP-036: Implement Json Type
**Effort:** M | **Files:** `src/types/json.rs`
**Spec Lines:** 441-458 (§9 JSON Model)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN the Json ADT
WHEN used in the type system
THEN it should support:
  - JsonNull (only from parsing, no literal)
  - JsonBool(boolean)
  - JsonInt(number)
  - JsonDecimal(Decimal)
  - JsonString(string)
  - JsonArray(List<Json>)
  - JsonObject(Map<string, Json>)

Pseudo-test:
  let json = Json::Object(map![
    "name" => Json::String("Alice"),
    "age" => Json::Int(30)
  ]);
  assert(json.type_of() == Type::Json)
```

---

## Section 5: Effect System

### - [ ] MTP-040: Implement Effect Declarations
**Effort:** M | **Files:** `src/effects/mod.rs`, `src/effects/declaration.rs`
**Spec Lines:** 342-357 (§7 Effect System)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN effect declarations
WHEN parsed and checked
THEN it should:
  - Register built-in effects: DbRead, DbWrite, HttpOut, Log, Async
  - Validate `uses { ... }` clauses
  - Prevent lambdas from using effects
  - Allow only named functions to use effects

Pseudo-test:
  let src = r#"
    function query() uses { DbRead } {
      DbRead("SELECT 1", {})
    }
  "#;
  assert(check_effects(parse(src)?).is_ok())

  let invalid = r#"
    const f = function() { DbRead("SELECT 1", {}) }  // lambda with effect
  "#;
  assert(check_effects(parse(invalid)?).is_err())
```

---

### - [ ] MTP-041: Implement Effect Checker
**Effort:** L | **Files:** `src/effects/checker.rs`
**Spec Lines:** 342-357, 396-409 (§7, §7-b)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a function or API with effects
WHEN compiling
THEN it should:
  - Track effect usage in function bodies
  - Verify all used effects are declared
  - Propagate effect requirements through calls
  - Error on undeclared effect usage

Pseudo-test:
  let src = r#"
    function save(data: Json) uses { DbRead } {  // declares DbRead
      DbWrite("INSERT ...", data)                  // uses DbWrite - ERROR
    }
  "#;
  let result = check_effects(parse(src)?);
  assert(result.is_err())
  assert(result.err().message.contains("DbWrite not declared"))
```

---

### - [ ] MTP-042: Implement Async Effect
**Effort:** L | **Files:** `src/effects/async_effect.rs`
**Spec Lines:** 359-395 (§7-a Async Effect)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN an async effect usage
WHEN compiled
THEN it should:
  - Only allow `await` inside `uses { Async }` functions
  - Desugar to Async.await(promiseHash, contId, effectArgs)
  - Compute promiseHash as SHA-256 of CBOR-encoded expression
  - Generate unique contId per await point

Pseudo-test:
  let src = r#"
    api POST /fx uses { Async } {
      const rate = await httpGet("https://fx.example.com")
      respond json({ rate })
    }
  "#;
  let ir = compile_to_ir(parse(src)?)?;
  assert(ir.contains_call("Async.await"))

  // contId uniqueness
  let cont_ids = ir.extract_cont_ids();
  assert(cont_ids.len() == cont_ids.unique().len())
```

---

### - [ ] MTP-043: Implement Built-in Functions
**Effort:** M | **Files:** `src/effects/builtins.rs`
**Spec Lines:** 410-422 (§7-c Built-in Functions)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN built-in pure functions
WHEN called
THEN they should be available:
  - Json.parse(s: string): Result<Json, string>
  - Json.stringify(j: Json): string
  - Decimal.fromString(s: string): Result<Decimal, string>
  - Decimal.toString(d: Decimal): string
  - fnv1a32(data: string): number
  - fnv1a64(data: string): number
  - cborEncode(j: Json): string

Pseudo-test:
  assert(call("Json.parse", [r#"{"a":1}"#]) == Ok(Json::Object(...)))
  assert(call("fnv1a64", ["hello"]) == 0x779a65e7023cd2e7)
```

---

## Section 6: Intermediate Representation (IR)

### - [ ] MTP-050: Define IR Data Structures
**Effort:** M | **Files:** `src/ir/mod.rs`, `src/ir/nodes.rs`
**Spec Lines:** 481-493 (§12 Compilation Pipeline)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN the compilation pipeline
WHEN transforming AST to IR
THEN IR should include:
  - Typed expressions (each node carries its type)
  - Effect-annotated functions
  - Desugared constructs (pipelines, pattern matches)
  - Tail-call annotations

Pseudo-test:
  let ir = IrProgram {
    functions: vec![
      IrFunction {
        name: "add",
        params: vec![("a", Type::Number), ("b", Type::Number)],
        return_type: Type::Number,
        effects: vec![],
        body: IrExpr::BinOp {
          op: "+",
          left: Box::new(IrExpr::Var("a", Type::Number)),
          right: Box::new(IrExpr::Var("b", Type::Number)),
          result_type: Type::Number,
        },
        is_tail_recursive: false,
      }
    ]
  };
  assert(ir.validate().is_ok())
```

---

### - [ ] MTP-051: Implement AST to IR Lowering
**Effort:** L | **Files:** `src/ir/lower.rs`
**Spec Lines:** 481-493 (§12), 609-614 (§25 Pipeline)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a typed AST
WHEN lowering to IR
THEN it should:
  - Desugar pipeline: a |> f ≡ f(a)
  - Desugar pattern matching to conditionals
  - Annotate tail calls
  - Preserve type information

Pseudo-test:
  let ast = parse("x |> double |> add(1)")?;
  let ir = lower(typecheck(ast)?)?;
  // Should become: add(double(x), 1)
  assert(ir == IrExpr::Call("add", [
    IrExpr::Call("double", [IrExpr::Var("x")]),
    IrExpr::Lit(1)
  ]))
```

---

### - [ ] MTP-052: Implement Tail Call Detection
**Effort:** M | **Files:** `src/ir/tail_call.rs`
**Spec Lines:** 328-329 (tail calls in §6), 770-771 (Annex A gas costs)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN recursive functions
WHEN analyzing IR
THEN it should:
  - Detect tail-recursive calls
  - Mark them for 0-gas optimization
  - Detect non-tail recursion (cost 2 gas)

Pseudo-test:
  let src = r#"
    function factorial(n: number, acc: number): number {
      if (n <= 1) { acc }
      else { factorial(n - 1, n * acc) }  // tail call
    }
  "#;
  let ir = lower(typecheck(parse(src)?)?)?;
  assert(ir.functions[0].body.last_call().is_tail_call == true)
```

---

## Section 7: Compiler (JS Codegen)

### - [ ] MTP-060: Implement JS Code Generator
**Effort:** L | **Files:** `src/compiler/mod.rs`, `src/compiler/codegen.rs`
**Spec Lines:** 481-493 (§12), 491 (Forbidden JS)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN typed IR
WHEN generating JS
THEN it should:
  - Produce valid JS subset
  - Exclude: eval, class, this, try/catch, loops, global mutation
  - Use only: functions, objects, arrays, primitives
  - Generate α-equivalent code across compilers

Pseudo-test:
  let ir = compile_to_ir(parse("function add(a, b) { a + b }")?)?;
  let js = codegen(ir)?;
  assert(js == "function add(a, b) { return a + b; }")

  // Forbidden constructs
  assert(!js.contains("class"))
  assert(!js.contains("this"))
  assert(!js.contains("eval"))
```

---

### - [ ] MTP-061: Implement Pattern Match Compilation
**Effort:** L | **Files:** `src/compiler/pattern.rs`
**Spec Lines:** 207-222 (match expressions)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a pattern match expression
WHEN compiled to JS
THEN it should:
  - Generate if-else chains or switch
  - Handle variant destructuring
  - Bind variables correctly
  - Maintain exhaustiveness (verified at compile time)

Pseudo-test:
  let src = r#"
    match opt {
      Some(x) => x * 2
      None => 0
    }
  "#;
  let js = compile(src)?;
  // Should generate something like:
  // if (opt.tag === "Some") { let x = opt.value; return x * 2; }
  // else { return 0; }
```

---

### - [ ] MTP-062: Implement Effect Call Compilation
**Effort:** M | **Files:** `src/compiler/effects.rs`
**Spec Lines:** 396-409 (§7-b Effect Invocation)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN effect calls in source
WHEN compiled to JS
THEN it should:
  - Generate calls to injected effect globals
  - Pass arguments as specified
  - Handle async effect desugaring

Pseudo-test:
  let src = r#"
    function query() uses { DbRead } {
      DbRead("SELECT * FROM users", {})
    }
  "#;
  let js = compile(src)?;
  assert(js.contains("DbRead(\"SELECT * FROM users\", {})"))
```

---

### - [ ] MTP-063: Implement Deterministic Code Generation
**Effort:** M | **Files:** `src/compiler/deterministic.rs`
**Spec Lines:** 613-614 (α-equivalent), 617-620 (§26 Formal Determinism)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN the same source code
WHEN compiled multiple times
THEN it should:
  - Produce identical JS output (byte-for-byte)
  - Use deterministic variable naming
  - Sort declarations consistently

Pseudo-test:
  let src = "function f() { const a = 1; const b = 2; a + b }";
  let js1 = compile(src)?;
  let js2 = compile(src)?;
  assert(sha256(js1) == sha256(js2))
```

---

## Section 8: Snapshot System

### - [ ] MTP-070: Implement Snapshot File Format
**Effort:** M | **Files:** `src/snapshot/format.rs`, `src/snapshot/mod.rs`
**Spec Lines:** 651-661 (§27.2 Interpreter Snapshot Format)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN the .msqs format specification
WHEN creating/reading snapshots
THEN it should follow:
  - Bytes 0-7: "MTPJS\x00\x00\x00" (magic)
  - Bytes 8-11: u32 version (51 for v5.1)
  - Bytes 12-19: u64 size
  - Bytes 20-51: SHA-256 of JS content
  - Bytes 52..size-132: UTF-8 JS text
  - Bytes size-132..size-4: ECDSA-P256 signature
  - Bytes size-4..size: CRC32

Pseudo-test:
  let js = "function main() { return 42; }";
  let snapshot = create_snapshot(js, &signing_key)?;

  assert(&snapshot[0..8] == b"MTPJS\x00\x00\x00")
  assert(u32::from_le_bytes(snapshot[8..12]) == 51)
  assert(&snapshot[20..52] == sha256(js))
```

---

### - [ ] MTP-071: Implement Snapshot Creation
**Effort:** M | **Files:** `src/snapshot/create.rs`
**Spec Lines:** 571-578 (Build in §22)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN compiled JS code
WHEN creating a snapshot
THEN it should:
  - Compute SHA-256 hash of JS
  - Sign with ECDSA-P256
  - Pack into .msqs format
  - Compute CRC32 checksum

Pseudo-test:
  let js = compile(parse("function main() { 1 + 1 }")?)?;
  let key = load_signing_key("key.pem")?;
  let snapshot = create_snapshot(js, &key)?;

  assert(snapshot.len() > 132) // minimum overhead
  assert(verify_snapshot(&snapshot, &key.public()).is_ok())
```

---

### - [ ] MTP-072: Implement Snapshot Verification
**Effort:** M | **Files:** `src/snapshot/verify.rs`
**Spec Lines:** 509-510 (signature verification), 579-586 (Runtime verify)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a .msqs file
WHEN loading at runtime
THEN it should:
  - Verify magic bytes
  - Verify version compatibility
  - Verify ECDSA-P256 signature against embedded certificate
  - Verify CRC32 checksum
  - Abort on any mismatch

Pseudo-test:
  let snapshot = read_file("app.msqs")?;
  let cert = load_cert("app.cert")?;

  assert(verify_snapshot(&snapshot, &cert).is_ok())

  // Tampered snapshot
  let mut bad = snapshot.clone();
  bad[100] ^= 0xFF;
  assert(verify_snapshot(&bad, &cert).is_err())
```

---

## Section 9: Runtime / Interpreter

### - [ ] MTP-080: Implement Interpreter Core
**Effort:** XL | **Files:** `src/runtime/interpreter.rs`, `src/runtime/mod.rs`
**Spec Lines:** 625-659 (§27.1-27.3)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a JS subset program
WHEN the interpreter runs
THEN it should:
  - Parse JS text to AST
  - Evaluate expressions
  - Support all allowed constructs
  - Reject forbidden constructs

Pseudo-test:
  let js = "function add(a, b) { return a + b; } add(1, 2)";
  let result = interpret(js)?;
  assert(result == Value::Number(3))

  // Reject forbidden
  let bad = "class Foo {}";
  assert(interpret(bad).is_err())
```

---

### - [ ] MTP-081: Implement Value Representation
**Effort:** M | **Files:** `src/runtime/value.rs`
**Spec Lines:** 253-259, 441-452 (Types and JSON)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN runtime values
WHEN processing
THEN they should support:
  - Number: i64 with checked overflow
  - Boolean: true/false
  - String: UTF-8
  - Decimal: fixed-point
  - Array: Vec<Value>
  - Object: Map<String, Value>
  - Function: closure representation

Pseudo-test:
  let num = Value::Number(42);
  assert(num.type_tag() == "number")

  let overflow = Value::Number(i64::MAX).add(Value::Number(1));
  assert(overflow.is_err())
```

---

### - [ ] MTP-082: Implement Interpreter Cloning
**Effort:** L | **Files:** `src/runtime/clone.rs`
**Spec Lines:** 663-688 (§27.3 Interpreter Cloning)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a snapshot
WHEN cloning for a request
THEN it should:
  - Complete in ≤ 1ms (best case)
  - Isolate from other instances
  - Use copy-on-write where possible
  - Initialize fresh heap

Pseudo-test:
  let snapshot = load_snapshot("app.msqs")?;
  let start = Instant::now();
  let interp = clone_interpreter(&snapshot)?;
  let elapsed = start.elapsed();

  assert(elapsed < Duration::from_millis(1))
  assert(interp.heap_size() == 0) // fresh heap
```

---

### - [ ] MTP-083: Implement Effect Injection
**Effort:** L | **Files:** `src/runtime/effects.rs`
**Spec Lines:** 690-718 (§27.4 Effect Injection)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a cloned interpreter
WHEN injecting effects
THEN it should:
  - Inject after clone, before static init
  - Provide DbRead, DbWrite, HttpOut, Log, Async
  - Make effects use deterministic seed
  - Cache Async responses by (seed, contId)

Pseudo-test:
  let mut interp = clone_interpreter(&snapshot)?;
  let seed = compute_seed(&request)?;
  inject_effects(&mut interp, &seed)?;

  // Effects should be available
  let result = interp.call_global("DbRead", &["SELECT 1", "{}"])?;
  assert(result.is_ok())
```

---

### - [ ] MTP-084: Implement Secure Wipe
**Effort:** M | **Files:** `src/runtime/wipe.rs`
**Spec Lines:** 27, 500, 585, 686-688 (secure wipe mentions)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN an interpreter after request completion
WHEN disposing
THEN it should:
  - Zero all memory if PCI data was touched
  - Prevent cross-request leakage
  - Release all resources

Pseudo-test:
  let mut interp = clone_interpreter(&snapshot)?;
  interp.mark_pci_touched();

  let heap_ptr = interp.heap_ptr();
  wipe_interpreter(interp, true)?;

  // Memory should be zeroed (in practice, check via memory dump)
  // No cross-request state should persist
```

---

### - [ ] MTP-085: Implement Deterministic Seed Computation
**Effort:** S | **Files:** `src/runtime/seed.rs`
**Spec Lines:** 34-67 (§0-b, §0-c Deterministic Seed)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN request metadata
WHEN computing the seed
THEN it should:
  - Concatenate: RequestId || AccountId || FunctionVersion || "mtpscript-v5.1" || SnapshotHash || GasLimitASCII
  - SHA-256 hash the concatenation
  - Produce identical 32-byte seed for same inputs

Pseudo-test:
  let seed = compute_seed(
    request_id: "abc123",
    account_id: "123456789",
    function_version: "1",
    snapshot_hash: &[0u8; 32],
    gas_limit: 10_000_000,
  )?;

  assert(seed.len() == 32)

  // Same inputs = same seed
  let seed2 = compute_seed(/* same args */)?;
  assert(seed == seed2)
```

---

## Section 10: Gas Metering

### - [ ] MTP-090: Implement Gas Counter
**Effort:** S | **Files:** `src/gas/counter.rs`, `src/gas/mod.rs`
**Spec Lines:** 52-115 (§0-c Gas Limit), 762-780 (Annex A)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a gas budget
WHEN executing code
THEN it should:
  - Initialize from MTP_GAS_LIMIT env var (default 10M)
  - Validate range: 1 to 2,000,000,000
  - Decrement per operation
  - Terminate with GasExhausted when exceeded

Pseudo-test:
  let mut gas = GasCounter::new(1000);

  gas.consume(100)?;  // OK
  assert(gas.remaining() == 900)

  gas.consume(1000);  // Exceeds
  assert(gas.is_exhausted())
  assert(gas.error() == GasExhausted { limit: 1000, used: 1100 })
```

---

### - [ ] MTP-091: Implement Gas Cost Table
**Effort:** S | **Files:** `src/gas/costs.rs`
**Spec Lines:** 762-780 (Annex A - Gas Cost Table)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN operations
WHEN costed
THEN they should match Annex A:
  - Literal number: 1
  - Literal string: 1
  - Binary op: 2
  - Comparison: 1
  - Function call: 5
  - Tail recursion: 0
  - Non-tail recursion: 2
  - Object/array access: 1
  - If statement: 1
  - Pattern match: 3 per case
  - Json.parse: 10 + len/10
  - Effect call: 20 + specific (DbRead: 50, HttpOut: 100)

Pseudo-test:
  assert(gas_cost(Op::Literal) == 1)
  assert(gas_cost(Op::BinaryOp) == 2)
  assert(gas_cost(Op::FunctionCall) == 5)
  assert(gas_cost(Op::TailCall) == 0)
  assert(gas_cost(Op::JsonParse(100)) == 10 + 10) // 10 + 100/10
```

---

### - [ ] MTP-092: Integrate Gas Metering in Interpreter
**Effort:** M | **Files:** `src/runtime/interpreter.rs` (update)
**Spec Lines:** 723-728 (§27.5 Gas Metering)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN the interpreter
WHEN executing each operation
THEN it should:
  - Decrement gas atomically
  - Check exhaustion after each step
  - Terminate immediately when exhausted
  - Return deterministic error value

Pseudo-test:
  let mut interp = clone_interpreter(&snapshot)?;
  interp.set_gas_limit(100);

  // Run expensive computation
  let result = interp.run("function loop(n) { if (n > 0) { loop(n-1) } } loop(1000)");

  assert(result.is_err())
  assert(result.err() == GasExhausted { limit: 100, used: /* >= 100 */ })
```

---

## Section 11: JSON / Serialization

### - [ ] MTP-100: Implement JSON Parser
**Effort:** M | **Files:** `src/json/parse.rs`, `src/json/mod.rs`
**Spec Lines:** 441-458 (§9 JSON Model)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN JSON input
WHEN parsing
THEN it should:
  - Return Result<Json, ParseError>
  - Support all JSON types
  - Reject duplicate keys
  - Handle JsonNull only from parsing

Pseudo-test:
  let json = Json::parse(r#"{"a": 1, "b": null}"#)?;
  assert(json.get("a") == Some(Json::Int(1)))
  assert(json.get("b") == Some(Json::Null))

  // Duplicate keys rejected
  let bad = Json::parse(r#"{"a": 1, "a": 2}"#);
  assert(bad.is_err())
```

---

### - [ ] MTP-101: Implement Canonical JSON Serializer
**Effort:** M | **Files:** `src/json/serialize.rs`
**Spec Lines:** 591-598 (§23 Canonical JSON), 146 (RFC 8785)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a Json value
WHEN serializing
THEN it should:
  - Follow RFC 8785 (JCS)
  - Order keys by §5 rules (type tag, hash, CBOR tie-break)
  - Use shortest Decimal form
  - Exclude -0, NaN, Infinity
  - Be byte-identical across runs

Pseudo-test:
  let json = Json::Object(map![
    "z" => Json::Int(1),
    "a" => Json::Int(2),
  ]);
  let canonical = json.to_canonical_string();

  // Keys sorted
  assert(canonical == r#"{"a":2,"z":1}"#)

  // SHA-256 deterministic
  assert(sha256(canonical) == /* fixed hash */)
```

---

### - [ ] MTP-102: Implement CBOR Encoder
**Effort:** M | **Files:** `src/json/cbor.rs`
**Spec Lines:** 144, 312 (CBOR mentions), 419 (cborEncode)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN a Json value
WHEN encoding to CBOR
THEN it should:
  - Follow RFC 7049 §3.9 (deterministic)
  - Produce identical bytes for identical values
  - Return hex string from cborEncode()

Pseudo-test:
  let json = Json::Object(map!["a" => Json::Int(1)]);
  let cbor = cbor_encode(&json);

  // Deterministic
  assert(cbor_encode(&json) == cbor_encode(&json))

  // Hex output
  assert(call("cborEncode", [json]) == hex::encode(cbor))
```

---

### - [ ] MTP-103: Implement Equality, Ordering, and Hashing
**Effort:** M | **Files:** `src/json/equality.rs`, `src/json/hash.rs`
**Spec Lines:** 308-320 (§5 Equality, Ordering & Hashing)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN values
WHEN comparing/hashing
THEN it should:
  - Use structural equality (no reference identity)
  - Order only number and string
  - Hash using FNV-1a 64-bit of deterministic CBOR
  - Include closure environments in equality
  - Exclude functions from map keys

Pseudo-test:
  let a = Json::Object(map!["x" => Json::Int(1)]);
  let b = Json::Object(map!["x" => Json::Int(1)]);

  assert(a == b) // structural
  assert(hash(&a) == hash(&b))

  // FNV-1a of CBOR
  assert(hash(&a) == fnv1a64(cbor_encode(&a)))
```

---

## Section 12: API System

### - [ ] MTP-110: Implement HTTP Router
**Effort:** M | **Files:** `src/api/router.rs`, `src/api/mod.rs`
**Spec Lines:** 425-438 (§8 API System), 515-520 (§15 Local Web Server)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN API declarations
WHEN routing requests
THEN it should:
  - Match method and path
  - Extract path parameters
  - Route to correct handler
  - Support GET, POST, PUT, DELETE, PATCH

Pseudo-test:
  let router = Router::from_apis(&[
    ApiDecl { method: POST, path: "/users", ... },
    ApiDecl { method: GET, path: "/users/:id", ... },
  ]);

  assert(router.match("POST", "/users").is_some())
  assert(router.match("GET", "/users/42").params["id"] == "42")
```

---

### - [ ] MTP-111: Implement Request Handler
**Effort:** L | **Files:** `src/api/handler.rs`
**Spec Lines:** 637-649 (§27.1 Host Process Flow)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN an HTTP request
WHEN handling
THEN it should:
  1. Parse request
  2. Extract metadata
  3. Compute seed
  4. Clone interpreter
  5. Inject effects
  6. Execute handler
  7. Serialize response to canonical JSON
  8. Hash response
  9. Log audit
  10. Return HTTP response

Pseudo-test:
  let req = Request::post("/users").body(r#"{"name":"Alice"}"#);
  let resp = handle_request(req)?;

  assert(resp.status == 200)
  assert(is_canonical_json(resp.body))
  assert(audit_log.last().response_hash == sha256(resp.body))
```

---

### - [ ] MTP-112: Implement OpenAPI Generator
**Effort:** L | **Files:** `src/api/openapi.rs`
**Spec Lines:** 436, 782-788 (§8, Annex B)
**Priority:** P2

**Acceptance Criteria:**
```
GIVEN API declarations and types
WHEN generating OpenAPI
THEN it should:
  - Order paths alphabetically
  - Generate schemas for records (as objects) and ADTs (as oneOf)
  - Use SHA-256 for $ref
  - Sort fields by name

Pseudo-test:
  let apis = parse_file("app.mtp")?;
  let openapi = generate_openapi(&apis);

  // Paths sorted
  let paths: Vec<_> = openapi.paths.keys().collect();
  assert(paths == paths.sorted())

  // Deterministic
  assert(generate_openapi(&apis) == generate_openapi(&apis))
```

---

### - [ ] MTP-113: Implement respond json() Expression
**Effort:** S | **Files:** `src/compiler/respond.rs`
**Spec Lines:** 213 (respond expression)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN a respond json(...) expression
WHEN compiled and executed
THEN it should:
  - Serialize expression to canonical JSON
  - Set appropriate Content-Type
  - Return as HTTP response body

Pseudo-test:
  let src = r#"
    api GET /status {
      respond json({ "ok": true })
    }
  "#;
  let resp = handle_request(Request::get("/status"))?;

  assert(resp.headers["Content-Type"] == "application/json")
  assert(resp.body == r#"{"ok":true}"#)
```

---

## Section 13: Security

### - [ ] MTP-120: Implement ECDSA-P256 Signing
**Effort:** M | **Files:** `src/security/sign.rs`, `src/security/mod.rs`
**Spec Lines:** 488, 577, 659 (signature mentions)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN JS code
WHEN signing
THEN it should:
  - Use ECDSA-P256
  - Sign SHA-256 hash of content
  - Append signature to snapshot

Pseudo-test:
  let key = load_private_key("signing.key")?;
  let js = "function main() { 42 }";
  let signature = sign_ecdsa_p256(&sha256(js), &key)?;

  assert(signature.len() == 64) // P-256 signature
  assert(verify(&sha256(js), &signature, &key.public()).is_ok())
```

---

### - [ ] MTP-121: Implement Signature Verification
**Effort:** M | **Files:** `src/security/verify.rs`
**Spec Lines:** 509-510, 579-580 (verification)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN a snapshot with signature
WHEN loading at runtime
THEN it should:
  - Verify ECDSA signature against certificate
  - Abort with clear error on mismatch
  - Complete before any code execution

Pseudo-test:
  let snapshot = load_file("app.msqs")?;
  let cert = load_cert("app.cert")?;

  match verify_snapshot(&snapshot, &cert) {
    Ok(js) => proceed(js),
    Err(e) => abort("Signature verification failed: {}", e)
  }
```

---

### - [ ] MTP-122: Implement Sandboxing
**Effort:** L | **Files:** `src/security/sandbox.rs`
**Spec Lines:** 676-680 (Isolation), 745-746 (§27.8 seccomp)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN the interpreter
WHEN sandboxed
THEN it should:
  - Only access injected globals
  - No direct syscalls except via effects
  - Use seccomp-bpf on Linux
  - Reject network access except HttpOut

Pseudo-test:
  let mut interp = sandboxed_interpreter(&snapshot)?;

  // Should fail: direct network
  let result = interp.eval("fetch('http://evil.com')");
  assert(result.is_err())

  // Should succeed: via effect
  inject_effects(&mut interp, &seed);
  let result = interp.call("HttpOut", ["GET", "http://allowed.com"]);
  assert(result.is_ok())
```

---

### - [ ] MTP-123: Implement Reproducible Builds
**Effort:** L | **Files:** `src/security/reproducible.rs`
**Spec Lines:** 541-542 (§18 reproducible builds)
**Priority:** P2

**Acceptance Criteria:**
```
GIVEN the same source code
WHEN built in reproducible container
THEN it should:
  - Use containerized build image pinned by SHA-256
  - Produce byte-identical snapshots
  - Generate signed build-info.json

Pseudo-test:
  let build1 = docker_build("sha256:abc123", "app.mtp")?;
  let build2 = docker_build("sha256:abc123", "app.mtp")?;

  assert(sha256(build1.snapshot) == sha256(build2.snapshot))
  assert(build1.build_info.container_hash == "sha256:abc123")
```

---

## Section 14: Module System

### - [ ] MTP-130: Implement Static Imports
**Effort:** M | **Files:** `src/modules/import.rs`, `src/modules/mod.rs`
**Spec Lines:** 460-465 (§10 Module System)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN import declarations
WHEN compiling
THEN it should:
  - Only allow static imports
  - Require git-hash pinning
  - Require signed git tags
  - Vendor at build time

Pseudo-test:
  let src = r#"
    import "github.com/example/lib@v1.2.3#abc123" as lib
    lib.helper()
  "#;
  let result = compile(src)?;

  assert(result.vendored.contains("lib@abc123"))
  assert(verify_git_tag("lib", "v1.2.3", "abc123").is_ok())
```

---

### - [ ] MTP-131: Implement Module Resolution
**Effort:** M | **Files:** `src/modules/resolve.rs`
**Spec Lines:** 460-465, 466-477 (§10, §11)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN module imports
WHEN resolving
THEN it should:
  - Support order-independent compilation
  - Detect circular dependencies
  - Produce audit manifest for unsafe deps

Pseudo-test:
  let modules = resolve_modules(&["main.mtp", "util.mtp", "types.mtp"])?;

  // Order independent
  let order1 = compile_modules(&modules)?;
  let order2 = compile_modules(&modules.reverse())?;
  assert(order1 == order2)

  // Circular detection
  let circular = resolve_modules(&["a.mtp imports b", "b.mtp imports a"]);
  assert(circular.is_err())
```

---

### - [ ] MTP-132: Implement npm Bridge (Unsafe)
**Effort:** L | **Files:** `src/modules/npm_bridge.rs`
**Spec Lines:** 558-568 (§21 npm Bridging)
**Priority:** P2

**Acceptance Criteria:**
```
GIVEN npm adapters in host/unsafe/
WHEN bridging
THEN they should:
  - Live outside MTPScript
  - Be pure functions of (seed, ...args)
  - Have no require() inside MTPScript
  - Generate audit manifest with content hashes

Pseudo-test:
  // host/unsafe/uuid.js
  // function uuid(seed, ...args) { return deterministicUUID(seed); }

  let manifest = compile_with_npm_bridge("app.mtp")?;
  assert(manifest.unsafeDeps == [{ name: "uuid", version: "9.0.1", hash: "sha256:..." }])
```

---

## Section 15: Audit & Logging

### - [ ] MTP-140: Implement Audit Logger
**Effort:** M | **Files:** `src/audit/logger.rs`, `src/audit/mod.rs`
**Spec Lines:** 95-101 (audit schema), 635 (Audit Logger), 737-739 (§27.7)
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN request execution
WHEN logging
THEN it should:
  - Output JSON lines to stderr
  - Include gasLimit in every entry
  - Include response SHA-256 hash
  - Forward to CloudWatch (in Lambda)

Pseudo-test:
  let entry = AuditEntry {
    request_id: "abc123",
    gas_limit: 10_000_000,
    gas_used: 5_000,
    response_hash: sha256(response_body),
    timestamp: "2024-01-01T00:00:00Z",
  };

  audit_log(&entry);
  assert(stderr.last_line().contains("gasLimit"))
```

---

### - [ ] MTP-141: Implement Request Tracing
**Effort:** M | **Files:** `src/audit/trace.rs`
**Spec Lines:** 76 (gasLimit in log), 97-101 (audit schema)
**Priority:** P2

**Acceptance Criteria:**
```
GIVEN each request
WHEN processed
THEN audit should record:
  - Request ID
  - Gas limit and usage
  - Effect calls made
  - Response hash
  - Duration

Pseudo-test:
  let trace = execute_with_tracing(&request)?;

  assert(trace.effects == ["DbRead(1)", "DbWrite(1)", "Log(2)"])
  assert(trace.response_hash.len() == 64) // hex SHA-256
```

---

## Section 16: Testing Infrastructure

### - [ ] MTP-150: Implement Test Runner Framework
**Effort:** M | **Files:** `tests/runner.rs`, `tests/mod.rs`
**Spec Lines:** 753-759 (§27.10 Testing)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN test cases
WHEN running tests
THEN it should:
  - Support unit tests for each component
  - Support integration tests
  - Support determinism fuzzing
  - Report coverage

Pseudo-test:
  cargo test --all
  # All tests pass

  cargo test --features fuzz -- determinism
  # Fuzz tests pass
```

---

### - [ ] MTP-151: Implement Determinism Verification Tests
**Effort:** M | **Files:** `tests/determinism.rs`
**Spec Lines:** 617-620 (§26 Formal Determinism Claim)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN the same program, inputs, and gas limit
WHEN executed multiple times
THEN SHA-256 of response should be identical

Pseudo-test:
  for _ in 0..1000 {
    let resp1 = execute("app.msqs", &input, gas_limit)?;
    let resp2 = execute("app.msqs", &input, gas_limit)?;
    assert(sha256(resp1) == sha256(resp2))
  }
```

---

### - [ ] MTP-152: Implement Lexer Tests
**Effort:** S | **Files:** `tests/lexer_tests.rs`
**Spec Lines:** 153-237 (§3 Syntax)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN various source inputs
WHEN lexing
THEN correct tokens should be produced

Pseudo-tests:
  #[test] fn test_keywords() { ... }
  #[test] fn test_operators() { ... }
  #[test] fn test_literals() { ... }
  #[test] fn test_identifiers() { ... }
  #[test] fn test_error_cases() { ... }
```

---

### - [ ] MTP-153: Implement Parser Tests
**Effort:** S | **Files:** `tests/parser_tests.rs`
**Spec Lines:** 153-237 (§3 Syntax)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN various source inputs
WHEN parsing
THEN correct AST should be produced

Pseudo-tests:
  #[test] fn test_type_decl() { ... }
  #[test] fn test_func_decl() { ... }
  #[test] fn test_api_decl() { ... }
  #[test] fn test_expressions() { ... }
  #[test] fn test_patterns() { ... }
```

---

### - [ ] MTP-154: Implement Type Checker Tests
**Effort:** M | **Files:** `tests/typecheck_tests.rs`
**Spec Lines:** 251-320 (§4, §5)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN various programs
WHEN type-checking
THEN correct types or errors should be reported

Pseudo-tests:
  #[test] fn test_primitives() { ... }
  #[test] fn test_records() { ... }
  #[test] fn test_adts() { ... }
  #[test] fn test_generics() { ... }
  #[test] fn test_errors() { ... }
```

---

### - [ ] MTP-155: Implement Effect Checker Tests
**Effort:** S | **Files:** `tests/effect_tests.rs`
**Spec Lines:** 342-422 (§7)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN various effect usages
WHEN checking
THEN correct validation should occur

Pseudo-tests:
  #[test] fn test_declared_effects() { ... }
  #[test] fn test_undeclared_effects() { ... }
  #[test] fn test_lambda_no_effects() { ... }
  #[test] fn test_async_await() { ... }
```

---

### - [ ] MTP-156: Implement Compiler Tests
**Effort:** M | **Files:** `tests/compiler_tests.rs`
**Spec Lines:** 481-493 (§12)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN various programs
WHEN compiling
THEN correct JS should be produced

Pseudo-tests:
  #[test] fn test_simple_function() { ... }
  #[test] fn test_pattern_match() { ... }
  #[test] fn test_effects() { ... }
  #[test] fn test_forbidden_constructs() { ... }
```

---

### - [ ] MTP-157: Implement Runtime Tests
**Effort:** M | **Files:** `tests/runtime_tests.rs`
**Spec Lines:** 625-759 (§27)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN compiled snapshots
WHEN executing
THEN correct results should be produced

Pseudo-tests:
  #[test] fn test_basic_execution() { ... }
  #[test] fn test_effect_injection() { ... }
  #[test] fn test_gas_metering() { ... }
  #[test] fn test_deterministic_output() { ... }
```

---

### - [ ] MTP-158: Implement Gas Metering Tests
**Effort:** S | **Files:** `tests/gas_tests.rs`
**Spec Lines:** 52-115, 762-780 (§0-c, Annex A)
**Priority:** P0 - Blocker

**Acceptance Criteria:**
```
GIVEN various operations
WHEN metering gas
THEN correct costs should be charged

Pseudo-tests:
  #[test] fn test_literal_cost() { ... }
  #[test] fn test_binop_cost() { ... }
  #[test] fn test_function_call_cost() { ... }
  #[test] fn test_tail_call_free() { ... }
  #[test] fn test_exhaustion() { ... }
```

---

### - [ ] MTP-159: Implement End-to-End Tests
**Effort:** L | **Files:** `tests/e2e_tests.rs`
**Spec Lines:** All major sections
**Priority:** P1

**Acceptance Criteria:**
```
GIVEN complete MTPScript programs
WHEN running full pipeline
THEN correct HTTP responses should be produced

Pseudo-tests:
  #[test] fn test_hello_world_api() { ... }
  #[test] fn test_crud_api() { ... }
  #[test] fn test_async_api() { ... }
  #[test] fn test_error_responses() { ... }
```

---

### - [ ] MTP-160: Implement Benchmark Suite
**Effort:** M | **Files:** `benches/benchmarks.rs`
**Spec Lines:** 26, 506-508 (performance requirements)
**Priority:** P2

**Acceptance Criteria:**
```
GIVEN benchmark programs
WHEN measuring performance
THEN it should meet targets:
  - Clone: ≤ 1ms best-case, ≤ 2ms worst-case
  - Gas overhead: < 1%

Pseudo-benchmarks:
  #[bench] fn bench_clone_interpreter() { ... }
  #[bench] fn bench_gas_overhead() { ... }
  #[bench] fn bench_json_serialize() { ... }
```

---

## Section 17: AWS Lambda Integration

### - [ ] MTP-170: Implement Lambda Custom Runtime
**Effort:** L | **Files:** `src/lambda/runtime.rs`, `src/lambda/mod.rs`
**Spec Lines:** 503-511 (§14 Serverless Deployment)
**Priority:** P2

**Acceptance Criteria:**
```
GIVEN the runtime binary
WHEN deployed to Lambda
THEN it should:
  - Ship as Rust binary + app.msqs + certificate
  - Achieve cold-start ≤ 2ms worst-case
  - Use no Node.js
  - Have no state reuse

Pseudo-test:
  let lambda = deploy_lambda("app.msqs")?;
  let cold_start = measure_cold_start(&lambda)?;

  assert(cold_start < Duration::from_millis(2))
```

---

### - [ ] MTP-171: Implement Lambda Request Adapter
**Effort:** M | **Files:** `src/lambda/adapter.rs`
**Spec Lines:** 68-78 (Host Adapter Contract)
**Priority:** P2

**Acceptance Criteria:**
```
GIVEN Lambda invocation
WHEN adapting
THEN it should:
  - Read MTP_GAS_LIMIT env var
  - Validate range (1-2B)
  - Append gasLimit to audit log
  - Extract AWS request metadata for seed

Pseudo-test:
  env::set_var("MTP_GAS_LIMIT", "5000000");
  let adapter = LambdaAdapter::new()?;

  assert(adapter.gas_limit == 5_000_000)

  env::set_var("MTP_GAS_LIMIT", "0");
  assert(LambdaAdapter::new().is_err()) // out of range
```

---

## Summary Statistics

### By Priority

| Priority | Count | Description |
|----------|-------|-------------|
| **P0** | 49 | Blockers - Must complete for MVP |
| **P1** | 16 | Important - Core functionality |
| **P2** | 7 | Nice-to-have - Can defer |

### By Effort

| Effort | Count | Total Hours (Est.) |
|--------|-------|-------------------|
| **S** (1-2h) | 16 | 16-32 hours |
| **M** (2-8h) | 35 | 70-280 hours |
| **L** (1-3d) | 17 | 136-408 hours |
| **XL** (3+d) | 4 | 96+ hours |

### By Section

| Section | Tasks | S | M | L | XL | P0 | P1 | P2 |
|---------|-------|---|---|---|----|----|----|----|
| 1. Scaffold | 3 | 2 | 1 | 0 | 0 | 2 | 1 | 0 |
| 2. Lexer | 3 | 2 | 1 | 0 | 0 | 3 | 0 | 0 |
| 3. Parser | 6 | 1 | 4 | 1 | 0 | 6 | 0 | 0 |
| 4. Type System | 7 | 2 | 3 | 1 | 1 | 7 | 0 | 0 |
| 5. Effect System | 4 | 0 | 2 | 2 | 0 | 2 | 2 | 0 |
| 6. IR | 3 | 0 | 2 | 1 | 0 | 2 | 1 | 0 |
| 7. Compiler | 4 | 0 | 2 | 2 | 0 | 3 | 1 | 0 |
| 8. Snapshot | 3 | 0 | 3 | 0 | 0 | 3 | 0 | 0 |
| 9. Runtime | 6 | 1 | 2 | 2 | 1 | 5 | 1 | 0 |
| 10. Gas | 3 | 2 | 1 | 0 | 0 | 3 | 0 | 0 |
| 11. JSON | 4 | 0 | 4 | 0 | 0 | 3 | 1 | 0 |
| 12. API | 4 | 1 | 1 | 2 | 0 | 0 | 3 | 1 |
| 13. Security | 4 | 0 | 2 | 2 | 0 | 2 | 1 | 1 |
| 14. Modules | 3 | 0 | 2 | 1 | 0 | 0 | 2 | 1 |
| 15. Audit | 2 | 0 | 2 | 0 | 0 | 0 | 1 | 1 |
| 16. Testing | 11 | 5 | 5 | 1 | 0 | 8 | 2 | 1 |
| 17. Lambda | 2 | 0 | 1 | 1 | 0 | 0 | 0 | 2 |
| **Total** | **72** | **16** | **38** | **16** | **2** | **49** | **16** | **7** |

---

## Task Division Recommendations

### Quick Wins (S tasks - Good for onboarding or parallel work)
- MTP-001, MTP-002 (Scaffold)
- MTP-011, MTP-012 (Lexer strings/numbers)
- MTP-025 (API declaration parser)
- MTP-030, MTP-034 (Primitive types, builtins)
- MTP-085 (Deterministic seed)
- MTP-090, MTP-091 (Gas counter, cost table)
- MTP-113 (respond json)
- MTP-152, MTP-153, MTP-155, MTP-158 (Simple tests)

### Core Complex Work (XL/L tasks - Need experienced devs)
- MTP-035 (Type Checker) - **XL** - Critical, complex
- MTP-080 (Interpreter Core) - **XL** - Critical, complex
- MTP-021 (Recursive Descent Parser) - **L**
- MTP-031 (Decimal Type) - **L** - Precision-critical
- MTP-033 (ADTs) - **L** - Type system core
- MTP-041 (Effect Checker) - **L**
- MTP-051 (AST to IR Lowering) - **L**
- MTP-060, MTP-061 (JS Codegen) - **L**
- MTP-082, MTP-083 (Interpreter cloning, effects) - **L**

### Independent Modules (Can be parallelized)
- Lexer (MTP-010-012) - No dependencies
- JSON/CBOR (MTP-100-103) - Mostly standalone
- Gas (MTP-090-092) - Standalone until integration
- Security signing (MTP-120-121) - Standalone
- Audit (MTP-140-141) - Standalone

### Sequential Dependencies
```
Scaffold → Lexer → Parser → Types → Effects → IR → Compiler → Snapshot → Runtime
                                                                    ↓
                                                              Gas Integration
                                                                    ↓
                                                               API System
                                                                    ↓
                                                                 Testing
```

---

## Dependency Graph (Critical Path)

```
MTP-001 (Scaffold)
    ↓
MTP-002 (Errors)
    ↓
┌───────────────┬───────────────┐
MTP-010 (Lexer) │               │
    ↓           │               │
MTP-020-025     │               │
(Parser)        │               │
    ↓           │               │
┌───────────────┤               │
MTP-030-036     MTP-040-043     │
(Types)         (Effects)       │
    ↓               ↓           │
└───────────────────────────────┤
                                ↓
                    MTP-050-052 (IR)
                                ↓
                    MTP-060-063 (Compiler)
                                ↓
                    MTP-070-072 (Snapshot)
                                ↓
                    MTP-080-085 (Runtime)
                                ↓
                    MTP-090-092 (Gas)
                                ↓
                    MTP-100-103 (JSON)
                                ↓
                    MTP-110-113 (API)
                                ↓
                    MTP-120-123 (Security)
                                ↓
                    MTP-150-160 (Tests)
```

---

*Generated from TECHSPECV5.md Version 5.1*
