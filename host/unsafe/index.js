/**
 * MTPScript npm bridge adapters
 *
 * Per TECHSPECV5.md §21:
 * - Adapters live **outside** MTPScript in `host/unsafe/*.js`
 * - Adapters **must** be **pure functions** of arguments + **deterministic seed (§0-b)**
 * - **Type signature** enforced: function adapterName(seed: Uint8Array, ...args: JsonValue[]): JsonValue
 * - No `require()` inside MTPScript, no shared state, no exceptions escaping
 * - Audit manifest lists every unsafe dependency **and its content-hash**
 *
 * Usage from MTPScript runtime:
 *   const adapters = require('./host/unsafe');
 *   const result = adapters.uuid.uuid(seed, 0);
 */

const uuid = require('./uuid');
const crypto = require('./crypto');
const datetime = require('./datetime');

/**
 * Load and validate manifest
 */
const manifest = require('./manifest.json');

/**
 * Wrap adapter function to catch exceptions and convert to errors
 * Per §21, exceptions must not escape
 *
 * @param {Function} fn - Adapter function
 * @param {string} name - Function name for error reporting
 * @returns {Function} - Wrapped function that returns { ok: value } or { error: message }
 */
function wrapAdapter(fn, name) {
  return function wrappedAdapter(seed, ...args) {
    try {
      const result = fn(seed, ...args);
      return { ok: result };
    } catch (error) {
      return { error: `${name}: ${error.message}` };
    }
  };
}

/**
 * Create wrapped adapters that catch all exceptions
 */
const safeAdapters = {
  uuid: {
    uuid: wrapAdapter(uuid.uuid, 'uuid')
  },
  crypto: {
    randomBytes: wrapAdapter(crypto.randomBytes, 'randomBytes'),
    sha256: wrapAdapter(crypto.sha256, 'sha256'),
    hmacSha256: wrapAdapter(crypto.hmacSha256, 'hmacSha256')
  },
  datetime: {
    now: wrapAdapter(datetime.now, 'now'),
    toIsoString: wrapAdapter(datetime.toIsoString, 'toIsoString'),
    parseIso: wrapAdapter(datetime.parseIso, 'parseIso'),
    addMs: wrapAdapter(datetime.addMs, 'addMs')
  }
};

/**
 * Verify adapter signature at runtime
 * All adapters must accept seed as first argument
 *
 * @param {Uint8Array} seed - 32-byte seed
 * @returns {boolean} - true if seed is valid
 */
function verifySeed(seed) {
  return seed instanceof Uint8Array && seed.length === 32;
}

module.exports = {
  ...safeAdapters,
  manifest,
  verifySeed
};
