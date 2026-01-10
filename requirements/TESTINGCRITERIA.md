TESTING RULES FOR AI

1. NEVER MODIFY EXISTING TESTS
   - Tests are immutable contracts
   - If tests are wrong, humans must revise them
   - AI can only create new tests or fix production code

2. GENERATE TESTS FROM SPECS ONLY
   - Read task.md and NEWARCHITECTURE.md to understand requirements
   - Do NOT look at implementation code when writing tests
   - Tests must include negative, boundary, and adversarial cases
   - Assume implementation is incorrect until proven otherwise

3. INCLUDE THESE TEST TYPES FOR EVERY FUNCTION
   - 3+ invalid input tests (corrupted data, wrong types, empty values)
   - 2+ state transition tests (order of operations, repeated calls)
   - 1+ adversarial test (injection, privilege escalation, boundary exhaustion)

4. WHEN TESTS FAIL
   - STOP and analyze each failure
   - Explain: which spec clause is violated, which function is responsible, root cause hypothesis
   - Do NOT modify tests or auto-fix code
   - Only then fix production code to satisfy tests

5. DETECT TEST REGRESSIONS
   - If any change reduces assertions, removes tests, or weakens expectations
   - STOP immediately and report: "Test regression detected. Human review required."

6. WORKFLOW: spec → tests → code
   - Never code → tests (creates shallow tests)
   - Tests expose real defects, not mirror implementation