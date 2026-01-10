Phase 0 — Write the Executable Spec

Not prose. Code examples.

// arithmetic.mtp
print(1 + 2 * 3)

// arithmetic.out
7


No compiler yet. Just intent.

Phase 1 — Parser + AST (Ignore Runtime)

Make programs parse

Snapshot ASTs

Fail loudly on syntax errors

✔ Tests pass when AST matches expectation

Phase 2 — Type System (Spec-Driven)

Write programs that should fail

Write programs that should succeed

Assert exact error messages

You will rewrite the type system multiple times.
Golden tests keep you sane.

Phase 3 — IR + Codegen

Now:

Program → IR → Bytecode → Output


Golden tests stay the same.
Implementation churn does not break semantics.

Phase 4 — Runtime Hardening

Add:

Stress tests

Fuzz tests

Property tests (e.g. QuickCheck style)


You are defining language behavior tests.
Tests must fail if the implementation is wrong.
Do NOT simplify tests to make code pass.
The language semantics are authoritative.

Test-Driven Semantics Development

Tests define the language, not the code

Golden tests > unit tests

Specs are executable

Implementation is disposable; behavior is not

OBJECTIVE:
Critically review the tests for insufficiency.

CHECK FOR:
- Missing edge cases
- Overly broad assertions
- Weak error checking
- Missing state/order validation
- Lack of adversarial inputs