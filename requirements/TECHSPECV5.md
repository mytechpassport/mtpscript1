# **MTPScript Language Specification**
**Version 5.1**

---

## 0. Executive Definition (One-Page)

MTPScript is a **serverless-first, deterministic API language** for regulated environments.
It compiles to a **constrained JavaScript subset**, executed by a **sealed MTPJS runtime** with:

* Zero ambient authority
* Zero hidden I/O
* Zero cross-request state
* Explicit capability declaration
* Per-request sandbox isolation ➜

JavaScript is **not** the language—it is an **execution encoding**.

---

## 0-a. Execution-Isolation Model ➜

MTPScript guarantees **per-request sandbox isolation** with **sub-millisecond reuse overhead**:

1. Build-time snapshot (`app.msqs`) – immutable, 150–400 kB
2. Runtime `clone_vm()` – copy-on-write, ≤ 60 µs **best-case**; **≤ 1 ms worst-case under EFS/FUSE cold page fault**
3. VM discarded after every request – no `fork()`, **secure wipe performed on pages that touched PCI-classified data**
4. Host effects re-injected per VM – static initialisers run **once per VM**, **after** effect seed injection, never reused

This removes cold-start latency while satisfying **“No cold state reuse”**.

---

## 0-b. Deterministic Seed Algorithm ➜

The **deterministic seed** injected into every VM is the **SHA-256 hash** of the **canonical concatenation**:

```
seed = SHA-256(
  AWS_Request_Id       ||
  AWS_Account_Id       ||
  Function_Version     ||
  "mtpscript-v5.1"     ||   // literal constant
  Snapshot_Content_Hash // SHA-256 of app.msqs
)
```

All conforming runtimes **must** produce the **same 32-byte seed** for the **same input byte sequence**; this seed is **never reused** across requests.

---

0-c  Runtime Gas Limit Injection
The gas budget is no longer hard-coded. Instead the **host** supplies an unsigned 64-bit value `gasLimit` (β-reduction units) when it clones the VM snapshot. That value is bound into the deterministic seed (see updated §0-b below) and is **immutable for the lifetime of the VM**. The guest program cannot read or write `gasLimit`; it is visible only to the host adapter and to audit logs.

1.  Deterministic Seed Algorithm (replacement text for §0-b)

```
seed = SHA-256(
  AWS_Request_Id       ||
  AWS_Account_Id       ||
  Function_Version     ||
  "mtpscript-v5.1"     ||
  Snapshot_Content_Hash||
  Gas_Limit_ASCII      ||   // ASCII decimal, no leading zeros, min="1", max="2000000000"
)
```

2.  Host Adapter Contract (adds to §13 Runtime Model)

Immediately after `clone_vm()` and before any static initialiser runs the adapter **must**:

a. Read the environment variable `MTP_GAS_LIMIT`.
   – If unset, default = 10 000 000.
   – If set but outside the range 1–2 000 000 000 (inclusive) the adapter **must abort the request** with `MTPError: GasLimitOutOfRange`.
b. Write the chosen value into the VM's internal `gasLimit` word (64-bit unsigned).
c. Append the field `gasLimit=<value>` to the request audit log.
d. Continue with effect injection and execution.

3.  Gas Exhaustion Semantics (adds to §6)

When the cumulative gas consumed ≥ `gasLimit` the VM **must** terminate with the deterministic error value:

```
{ "error": "GasExhausted",
  "gasLimit": <uint64>,
  "gasUsed": <uint64> }
```

No stack trace is emitted in production. The response body is still canonical JSON per §23 and is hashed into the response SHA-256; therefore the failure is deterministic across all conforming runtimes that receive the same `gasLimit`.

4.  Annex A – Gas Cost Table (adds preamble)

All opcode and built-in costs are fixed and expressed in β-reduction units. Changing `gasLimit` does **not** change the **unit cost** of any operation; it only changes the **budget** against which those costs are charged.

5.  OpenAPI / Audit Schema (adds one field)

Every request log entry published to the audit stream **must** include:

```
"gasLimit": { "type": "integer", "minimum": 1, "maximum": 2000000000 }
```

6.  Formal Determinism Claim (adds to §26)

The SHA-256 of the response body is identical across all conforming runtimes **if and only if** the injected `gasLimit` is identical. Thus the new claim becomes:

"For every MTPScript program P, compiler version C, input byte sequence I, and operator-supplied `gasLimit` L, the SHA-256 of the canonical JSON response is identical across all conforming runtimes."

7.  Security Considerations

- Guest code cannot query `gasLimit`; therefore knowledge of the budget cannot alter control flow.
- The value is bound into the seed, so replaying the same request with a different budget yields a different seed and therefore a different (yet still deterministic) execution trace.
- The upper bound 2 000 000 000 ≈ 2× the default prevents accidental DoS while staying within a 64-bit unsigned integer.
- Operators must re-sign the snapshot if they want to hard-code a non-default limit into the build; otherwise the limit is chosen at runtime and logged.

---

## 1. Design Goals (Hard Constraints)

### 1.1 Primary Goals
* Deterministic execution semantics (bit-exact SHA-256 response) ➜
* Explicit authority via effects
* Strong auditability
* Serverless suitability (AWS Lambda custom runtime)
* Mechanical migration from TypeScript APIs

### 1.2 Explicit Non-Goals
* Classes & inheritance
* Reflection / introspection
* Metaprogramming / macros
* Dynamic code loading
* Shared mutable state
* Threads or concurrency primitives
* Implicit coercions or I/O
* Floating-point math

---

## 2. Determinism Model (Auditor-Safe) ➜

| Guarantee | Status | Normative Requirement |
|-----------|--------|------------------------|
| Deterministic execution | ✅ | Same input bytes → same output bytes (SHA-256) |
| Deterministic hashing | ✅ | FNV-1a 64-bit + **deterministic CBOR (RFC 7049 §3.9)** |
| Deterministic equality | ✅ | Structural, total |
| Deterministic serialization | ✅ | Canonical JSON (RFC 8785) **+ duplicate-key rejection** |
| Deterministic API behaviour | ✅ | Host effects replay-identical **using seed per §0-b** |
| Bit-identical JS | ❌ Not claimed | – |
| Bit-identical VM bytecode | ❌ Not claimed | – |

---

## 3. Syntax & Grammar (Locked)

### 3.1 Full EBNF Grammar

The MTPScript grammar is defined by the following EBNF. Pipeline operator is left-associative. Await is only permitted inside functions that use the `Async` effect.

```
program ::= module_decl*

module_decl ::= 'import' string_literal 'as' identifier
               | type_decl
               | func_decl
               | api_decl

type_decl ::= 'type' identifier '{' field_decl* '}'
            | 'type' identifier '=' variant_decl ('|' variant_decl)*

field_decl ::= identifier ':' type_expr

variant_decl ::= identifier '(' type_expr* ')'
               | identifier

type_expr ::= identifier
            | 'List<' type_expr '>'
            | 'Map<' type_expr ',' type_expr '>'
            | 'Option<' type_expr '>'
            | 'Result<' type_expr ',' type_expr '>'
            | 'Decimal'
            | 'boolean'
            | 'number'
            | 'string'
            | 'Json'

func_decl ::= 'function' identifier '(' param_list? ')' ('uses' '{' effect_list '}')? '{' expr '}'

param_list ::= identifier ':' type_expr (',' identifier ':' type_expr)*

effect_list ::= identifier (',' identifier)*

api_decl ::= 'api' http_method string_literal
             ('uses' '{' effect_list '}')? '{'
             expr
             '}'

http_method ::= 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'

expr ::= literal
       | identifier
       | expr '.' identifier
       | expr '[' expr ']'
       | expr '(' expr_list? ')'
       | '!' expr
       | '-' expr
       | expr binop expr
       | expr '|' '>' expr
       | 'if' '(' expr ')' '{' expr '}' 'else' '{' expr '}'
       | 'match' expr '{' case+ '}'
       | 'const' identifier '=' expr ';' expr
       | 'function' '(' param_list? ')' '{' expr '}'
       | 'await' expr   // only inside `uses { Async }`
       | 'respond' 'json' '(' expr ')'
       | '(' expr ')'

expr_list ::= expr (',' expr)*

case ::= pattern '=>' expr

pattern ::= '_' | identifier | literal | identifier '(' pattern* ')' | identifier '{' field_pattern* '}'

field_pattern ::= identifier ':' pattern

binop ::= '+' | '-' | '*' | '/' | '==' | '!=' | '<' | '>' | '<=' | '>=' | '&&' | '||'

literal ::= number_literal | string_literal | boolean_literal | array_literal | object_literal

array_literal ::= '[' expr_list? ']'

object_literal ::= '{' (string_literal ':' expr)* '}'

number_literal ::= digit+ ('.' digit+)?
string_literal ::= '"' [^"]* '"'
boolean_literal ::= 'true' | 'false'
identifier ::= [a-zA-Z_][a-zA-Z0-9_]*

```

### 3.2 Pipeline Associativity

Left-associative: `a |> b |> c ≡ (a |> b) |> c`

**Addition:**
```
expr ::= ...
       | await expr   // only inside `uses { Async }`
```

---

## 4. Type System

### 4.1 Primitive Types
| Type | Notes |
|------|-------|
| `number` | Signed 64-bit, checked overflow |
| `boolean` | `true` / `false` only |
| `string` | Immutable UTF-8 |
| `Decimal` | Deterministic fixed-point |

### 4.2 Composite Types
Records and algebraic data types.

**Records:**
```mtp
type User {
  id: number
  name: string
  email: string
}
```

**Algebraic Data Types (ADTs):**
```mtp
type Result<T, E> = Ok(T) | Err(E)
type Option<T> = Some(T) | None
```

Pattern matching on ADTs:
```mtp
match result {
  Ok(value) => value
  Err(error) => handleError(error)
}
```

### 4.3 No `null`, No `undefined`
Use `Option<T>` and `Result<T, E>`.

---

## 4-a. Decimal / Money ➜

```mtp
type Decimal {
  value: string   // canonical integer significand, **1–34 digits**, no leading zeros
  scale: number      // 0 ≤ scale ≤ 28 (IEEE-754-2008 decimal128)
}
```

* Rounding: **round-half-even**; **ties to even** as required by IEEE-754-2008 **clause 4.3.2**
* Overflow: `Result<Decimal, Overflow>`
* Comparison: **constant-time** algorithm; normalise to **larger scale**, then compare significands
* Serialization: shortest canonical string (no trailing zeros)

---

## 5. Equality, Ordering & Hashing ➜

* Equality: structural, total, no reference identity
* Ordering: only `number` and `string`
* Hash: FNV-1a 64-bit of **deterministic CBOR (RFC 7049 §3.9)**
* Map key order:
  1. Type tag
  2. Hash
  3. CBOR byte-wise tie-break

Functions **excluded** from map keys.
**Closure environments are included in structural equality.**

---

## 6. Control Flow & Execution

* All values immutable
* `if` must have `else`, both branches same type
* Pattern matches exhaustive, compiler-checked
* Recursion bounded by **gas** (10 M β-reductions) ➜
  **Gas cost table appended in Annex A – every IR opcode and built-in carries a fixed cost; tail calls cost 0.**

## 6-a. Functions and Closures ➜

* Functions are declared with `function identifier(params) uses { effects } { body }`
* Lambdas are anonymous functions: `function(params) { body }`
* Lambdas are pure: they cannot use effects and must be total (no divergence)
* Named functions may use effects declared in their `uses` clause
* Closures capture their environment immutably; no mutable closures
* Tail recursion is optimized and costs 0 gas

---

## 7. Effect System (Authority Model)

Effects represent **capabilities**.
Lambdas are **pure**; only named functions may use effects.
Host effects **must** be deterministic functions of their arguments + **request seed per §0-b** ➜

**Built-in effects:**

| Effect | Capability |
|---|---|
| `DbRead`, `DbWrite` | Sqlite execution |
| `HttpOut` | Outbound HTTP |
| `Log` | Structured logging |
| `Async` | ➜ **Deterministic async I/O** (see §7-a) |

---

## 7-a. Async Effect (Deterministic Await) ➜

```mtp
effect Async {
  await<T>(promiseHash: string, contId: number, effectArgs: Json): Result<T, Err>
}
```

**Surface syntax:**

```mtp
api POST /invoice
uses { Async, DbWrite } {
  const rate = await httpGet("https://fx.example.com/usd-eur")   // desugars to Async.await
  const total = amount * rate
  DbWrite.insert("invoice", total)
  respond json({ total })
}
```

**Compile-time desugaring:**

```
let x = await e
≡
let contId   = freshInt()
let ph       = sha256(cbor(e))
let x        = Async.await(ph, contId, e)
```

**Host adapter contract:**

1. Block-synchronously execute the I/O.
2. Cache response bytes keyed by `(seed, contId)`.
3. Return **identical** bytes on every replay.
4. **No JavaScript event loop visible inside VM.**

## 7-b. Effect Invocation in Surface Syntax ➜

Effects are accessed as global identifiers injected by the runtime. They can be called directly as functions or may provide objects with methods.

For example:
- `DbRead(sql: string, params: Json): Json` – executes SQLITE read
- `DbWrite(sql: string, params: Json): Json` – executes SQLITE write
- `HttpOut(method: string, url: string, body?: Json): Json` – makes HTTP request
- `Log(level: string, message: string, data?: Json): void` – logs message

Higher-level APIs may be provided as library functions that desugar to these effects. For instance, `db.insert(table, data)` may be a library function that calls `DbWrite` with appropriate SQLITE.

In API declarations, the `uses { ... }` block declares which effects are permitted in that scope. Attempting to use an undeclared effect results in a compile-time error.

## 7-c. Built-in Functions ➜

In addition to effects, the runtime provides built-in pure functions:

* `Json.parse(s: string): Result<Json, string>` – Parses JSON string to Json type
* `Json.stringify(j: Json): string` – Serializes Json to canonical JSON string
* `Decimal.fromString(s: string): Result<Decimal, string>` – Parses decimal from string
* `Decimal.toString(d: Decimal): string` – Formats decimal to shortest string
* Hash functions: `fnv1a32(data: string): number`, `fnv1a64(data: string): number`
* CBOR encoding: `cborEncode(j: Json): string` – Deterministic CBOR bytes as hex string

These functions are available globally and have fixed gas costs as per Annex A.

---

## 8. API System (First-Class) ➜

```mtp
api POST /users
uses { db, log } {
  const user = db.insert(...)
  log.info("created user", user)
  respond json(user)
}
```

* Compile-time OpenAPI generation **with deterministic field ordering and $ref folding rules (Annex B)**
* No hidden middleware

---

## 9. JSON Model ➜

```mtp
type Json {
  | JsonNull           // **inhabited only through parsing; no MTPScript literal produces JsonNull**
  | JsonBool(boolean)
  | JsonInt(number)
  | JsonDecimal(Decimal)
  | JsonString(string)
  | JsonArray(List<Json>)
  | JsonObject(Map<string, Json>)  // **duplicate keys rejected at parse time**
}
```

* Parsing returns `Result`
* Output **canonical JSON** (RFC 8785 + Decimal form)

---

## 10. Module System

* Static imports only
* Git-hash pinned, **signed tag required**, vendored at build
* Order-independent compilation

---

## 11. Package Manager (v1)

* Git-hash based, **git-tag signature required**, no runtime network
* npm bridge via **explicit unsafe adapters**
* Produces audit manifest:

```json
{ "unsafeDeps": ["uuid@9.0.1"] }
```

---

## 12. Compilation Pipeline

```
MTPScript
  → AST
  → Typed IR
  → Effect-checked IR
  → Deterministic JS Subset (.js text file)
  → VM Snapshot (.msqs) ➜ **ECDSA-P256 signature appended**
```

Forbidden JS: `eval`, `class`, `this`, `try/catch`, loops, global mutation.
The JS subset is minimal: functions, objects, arrays, primitives, no prototypes, no closures beyond what's needed.

---

## 13. Runtime Model ➜

* One **fresh interpreter instance** per request (snapshot clone)
* Fixed memory budget (no shared heap)
* Interpreter discarded after response; **secure wipe executed on sensitive data**
* Host effects injected **per instance**, **after** static init, **deterministic seed per §0-b**

---

## 14. Serverless Deployment (AWS Lambda)

* Custom runtime ships **Rust binary** + **app.msqs** + **signature certificate**
* Cold-start ≤ 1 ms **best-case**; **≤ 2 ms worst-case** under EFS page fault
* No Node.js, no state reuse
**Runtime verifies ECDSA signature of app.msqs before loading; abort on mismatch.**

---

## 15. Local Web Server (Reference)

```mtp
serve { port: 8080, routes: [...] }
```

Identical semantics to Lambda; uses same snapshot clone path.

---

## 16. Error System

* Typed error codes
* No stack traces in prod
* Deterministic error shapes (canonical JSON)

---

## 17. TypeScript → MTPScript Migration

Mechanical transforms (unchanged).

---

## 18. Security & Audit Posture

Supports SOC 2, SOX, ISO 27001, PCI-DSS.
Authority explicit, behaviour deterministic, runtime sealed, surface minimal.
**Reproducible builds enforced by containerised build image pinned by SHA-256 and signed build-info.json.**

---

## 19. Final Positioning Statement

> MTPScript is a serverless-first, deterministic API language that uses JavaScript as a constrained execution format under a sealed, per-request sandbox runtime, designed for regulated environments where auditability and explicit authority matter more than dynamism.

---

## 20. HTTP Server Support

Same snapshot-clone isolation; not user-programmable.

---

## 21. npm Bridging (Unsafe Boundary) ➜

* Adapters live **outside** MTPScript in `host/unsafe/*.js`
* Adapters **must** be **pure functions** of arguments + **deterministic seed (§0-b)**
* **Type signature** enforced:
```js
function adapterName(seed: Uint8Array, ...args: JsonValue[]): JsonValue
```
* No `require()` inside MTPScript, no shared state, no exceptions escaping
* Audit manifest lists every unsafe dependency **and its content-hash**

---

## 22. Interpreter Snapshot Lifecycle ➜

```
Build
  └── mtp compile app.mtp → app.js
  └── mtp snapshot app.js → app.msqs
  └── sign app.msqs with ECDSA-P256 → app.msqs.sig

Runtime (per request)
  ├── verify app.msqs.sig against embedded certificate
  ├── load app.msqs
  ├── clone_interpreter()       // <1 ms
  ├── inject deterministic effects **after** static init
  ├── interpret JS
  └── drop_interpreter() **+ secure wipe on sensitive data**
```

No memory-wipe for non-sensitive data; **zero cross-request leakage**.

---

## 23. Canonical JSON Output ➜

* Object keys ordered by §5 rules
* Decimal shortest form, no `-0`, no `NaN`, no `Infinity`
* Array order preserved from source literal (left-associative)
**Output byte sequence is hashed with SHA-256 to produce the deterministic claim in §26.**

---

## 24. Union Exhaustiveness (Link-Time) ➜

* Union carries content-hash of variant list
* Link step fails if any unit sees different variant set
* Guarantees exhaustive matches without runtime checks

---

## 25. Pipeline Operator Associativity ➜

Left-associative:
`a |> b |> c ≡ (a |> b) |> c`
Generated JS **α-equivalent** across all compilers.

---

## 26. Formal Determinism Claim ➜

> For every MTPScript program P, compiler version C, and input byte sequence I, the SHA-256 of the HTTP response body is identical across all conforming runtimes **after canonical JSON encoding per §23**, **using the deterministic seed algorithm per §0-b**, **and assuming deterministic CBOR per §2**.

---

## 27. Runtime Implementation Guide

This section provides exhaustive, normative detail on implementing the MTPJS runtime as a Rust-based interpreter for the constrained JS subset. No stubs, mocks, or assumptions; all details are prescriptive. The runtime is a Rust binary that parses and interprets .js files.

### 27.1 Architecture Overview

The runtime is a Rust binary `mtpjs-runtime` with:
- **HTTP Handler**: Tokio for HTTP.
- **Interpreter Manager**: Struct `InterpreterManager` for clone, execute, wipe.
- **JS Interpreter**: Custom tree-walking interpreter.
- **Effect Registry**: HashMap<String, Box<dyn Fn(Vec<Value>) -> Value>>.
- **Gas Meter**: u64 per interpreter.
- **Audit Logger**: JSON to stdout.

#### Host Process Flow
1. Listen on port 8080.
2. Parse request.
3. Extract metadata.
4. Compute seed.
5. Clone interpreter.
6. Inject seed as global.
7. Inject effects.
8. Interpret JS with gas.
9. Serialize output to JSON.
10. Hash response.
11. Log audit.
12. Return HTTP response.

### 27.2 Interpreter Snapshot Format

`.msqs` binary:
- 0-7: "MTPJS\x00\x00\x00"
- 8-11: u32 51
- 12-19: u64 size
- 20-51: SHA256 of JS
- 52..size-132: UTF-8 JS text
- size-132..size-4: ECDSA sig
- size-4..size: CRC32

Creation: compile to .js, embed in format.

### 27.3 Interpreter Cloning and Isolation

#### Clone Algorithm
```rust
fn clone_interpreter(snapshot: &[u8]) -> Result<Interpreter, Error> {
    verify_sig(snapshot)?;
    let js = std::str::from_utf8(&snapshot[52..snapshot.len()-132])?;
    let ast = parse_js(js)?;
    let interp = Interpreter::new(ast, 512 * 1024 * 1024);
    Ok(interp)
}
```

#### Isolation
- Interpreter only accesses injected globals.
- No syscalls except via effects.
- Memory: bump allocator, wiped on drop.

#### Secure Wipe
```rust
fn wipe_interpreter(interp: Interpreter, pci: bool) {
    if pci { interp.zero_sensitive(); }
    drop(interp);
}
```

### 27.4 Effect Injection and Implementation

Effects injected post-clone:
```rust
fn inject_effects(isolate: &mut Isolate, seed: &[u8;32]) {
    let context = isolate.get_current_context();
    let global = context.global();

    // DbRead
    let db_read = v8::Function::new(context, |args| {
        let sql = args[0].to_string();
        let params = args[1].to_json();
        let key = sha256(concat(seed, sql.as_bytes(), params));
        // Deterministic SQLite query: use seeded RNG for any non-deterministic parts
        let result = sqlite_execute(sql, params, seed);
        v8::json::parse(context, &result)
    });
    global.set(context, "DbRead", db_read);

    // Similarly for DbWrite, HttpOut, Log, Async
    // Async.await: cache by (seed, cont_id), execute HTTP synchronously
}
```

#### Async Implementation
- Cache: HashMap<(seed, cont_id), response>.
- On await(ph, cont_id, args): if cached, return; else execute HTTP, cache, return.
- HTTP: use reqwest with seeded client, deterministic timeouts.

#### Deterministic Behavior
All effects: hash(seed || args) to seed any randomness (e.g., UUIDs, timestamps).

### 27.5 Gas Metering

Gas: AtomicU64 initialized to gas_limit.
Costs from Annex A CSV, loaded at startup.
Hook: V8 bytecode callback decrements gas.
On underflow: isolate.terminate_execution(); throw GasExhausted error.

### 27.6 Canonical JSON

Serialization:
- Use custom serializer: sort keys by type_tag + fnv_hash + cbor_bytes.
- Decimals: format as per §4-a.
- Output: UTF-8 bytes, SHA256 hash.

### 27.7 Error Handling

All errors: structured JSON, no panics.
Audit: JSON lines to stderr, forwarded to CloudWatch.

### 27.8 Security

- Code review mandatory.
- Fuzzing with AFL on input parsing.
- No network except HttpOut.
- Sandbox: seccomp-bpf to restrict syscalls.

### 27.9 Performance

- Clone: <1ms via V8 snapshot restore.
- Gas: <1% overhead.
- Async cache: LRU with 10k entries.

### 27.10 Testing

- Unit tests for each component.
- Integration: determinism fuzzing.
- Benchmarks: hyperfine on cold starts.

---

**Annex A – Gas Cost Table (Normative)**

All costs in β-reduction units.

- Literal number: 1
- Literal string: 1
- Binary op (+,-,*,/): 2
- Comparison (==, <, etc.): 1
- Function call: 5
- Recursion (tail): 0, non-tail: 2
- Object access: 1
- Array access: 1
- If statement: 1
- Pattern match: 3 per case
- Json.parse: 10 + length/10
- Json.stringify: 10 + length/10
- Effect call: 20 + effect-specific (DbRead: 50, HttpOut: 100, etc.)

Full table in `/gas-v5.1.csv`, but these are examples.

**Annex B – Deterministic OpenAPI Generation Rules**

- Paths ordered alphabetically.
- Schemas: records as objects, ADTs as oneOf.
- Refs: use SHA256 of schema as $ref.
- Field order: sorted by name.

Full in `/openapi-rules-v5.1.json`.

--------------------------------------------------------
End of Specification 5.1