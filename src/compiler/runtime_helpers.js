// Runtime helpers for MTPScript

// ADT constructors
function Some(value) {
    return { type: 'Some', value };
}

function None() {
    return { type: 'None' };
}

function Ok(value) {
    return { type: 'Ok', value };
}

function Err(error) {
    return { type: 'Err', error };
}

// Pattern matching helper
function match(value, cases) {
    for (let i = 0; i < cases.length; i += 2) {
        const pattern = cases[i];
        const handler = cases[i + 1];
        const result = matchPattern(value, pattern);
        if (result.matched) {
            // Apply bindings to handler
            return handler(result.bindings);
        }
    }
    throw new Error('No pattern matched');
}

function matchPattern(value, pattern) {
    if (pattern.type === 'Variant') {
        if (value && value.type === pattern.name) {
            const bindings = {};
            // For now, assume single payload
            if (pattern.subPatterns && pattern.subPatterns.length > 0) {
                const subResult = matchPattern(value.value, pattern.subPatterns[0]);
                if (subResult.matched) {
                    Object.assign(bindings, subResult.bindings);
                    return { matched: true, bindings };
                }
            } else {
                return { matched: true, bindings: {} };
            }
        }
    } else if (pattern.type === 'Wildcard') {
        return { matched: true, bindings: {} };
    } else if (pattern.type === 'Ident') {
        return { matched: true, bindings: { [pattern.name]: value } };
    } else if (pattern.type === 'Lit') {
        // Simple literal matching
        if (value === pattern.value) {
            return { matched: true, bindings: {} };
        }
    }
    return { matched: false, bindings: {} };
}

// Array bounds checking
function array_get(arr, index) {
    if (index < 0 || index >= arr.length) {
        throw new Error(`Array index ${index} out of bounds for array of length ${arr.length}`);
    }
    return arr[index];
}
    if (index < 0 || index >= arr.length) {
        throw new Error(`Array index ${index} out of bounds for array of length ${arr.length}`);
    }
    return arr[index];
}

// JSON operations with duplicate key detection
const Json = {
    parse(str) {
        try {
            // Custom JSON parser that detects duplicate keys
            const parsed = this.parseWithDuplicateCheck(str);
            return Ok(parsed);
        } catch (e) {
            return Err(e.message);
        }
    },

    parseWithDuplicateCheck(str) {
        // Simple duplicate key detection
        const seen = new Set();
        let inString = false;
        let keyStart = -1;
        let braceCount = 0;

        for (let i = 0; i < str.length; i++) {
            const char = str[i];
            if (char === '"' && (i === 0 || str[i-1] !== '\\')) {
                inString = !inString;
                if (inString && braceCount === 1) {
                    keyStart = i + 1;
                } else if (!inString && keyStart !== -1) {
                    const key = str.substring(keyStart, i);
                    if (seen.has(key)) {
                        throw new Error(`Duplicate key: ${key}`);
                    }
                    seen.add(key);
                    keyStart = -1;
                }
            } else if (!inString) {
                if (char === '{') braceCount++;
                else if (char === '}') braceCount--;
            }
        }

        return JSON.parse(str);
    },

    stringify(obj) {
        try {
            return Ok(JSON.stringify(obj));
        } catch (e) {
            return Err(e.message);
        }
    }
};

// Lambda support
function createLambda(params, body) {
    return function(...args) {
        // Simplified - assume body is executable JS
        if (typeof body === 'function') {
            return body.apply(null, args);
        }
        return body;
    };
}

// Async/await support
function async_func(body) {
    // Simplified async support
    return body();
}