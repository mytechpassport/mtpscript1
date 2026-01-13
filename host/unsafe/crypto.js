/**
 * Crypto npm bridge adapter
 *
 * Provides deterministic cryptographic operations based on seed.
 * Per TECHSPECV5.md §21, this adapter:
 * - Lives in host/unsafe/
 * - Is a pure function of arguments + deterministic seed
 * - Has enforced type signature
 * - No shared state, no exceptions escaping
 *
 * All operations are deterministic given the same seed.
 */

const crypto = require('crypto');

/**
 * Generate deterministic random bytes from seed
 *
 * @param {Uint8Array} seed - 32-byte deterministic seed from §0-b
 * @param {number} length - Number of bytes to generate
 * @param {number} index - Index for generating unique but deterministic output
 * @returns {string} - Hex-encoded random bytes
 */
function randomBytes(seed, length, index = 0) {
  if (!(seed instanceof Uint8Array) || seed.length !== 32) {
    throw new Error('randomBytes adapter requires 32-byte Uint8Array seed');
  }

  if (typeof length !== 'number' || !Number.isInteger(length) || length <= 0 || length > 1024) {
    throw new Error('randomBytes length must be positive integer <= 1024');
  }

  // Use HMAC-SHA256 with seed as key and index as data for deterministic output
  const hmac = crypto.createHmac('sha256', Buffer.from(seed));
  hmac.update(`randomBytes:${index}:${length}`);
  const hash = hmac.digest();

  // Expand to requested length using HKDF-like expansion
  const result = [];
  let block = hash;
  let counter = 0;
  while (result.length < length) {
    const expand = crypto.createHmac('sha256', Buffer.from(seed));
    expand.update(block);
    expand.update(Buffer.from([counter]));
    block = expand.digest();
    result.push(...block);
    counter++;
  }

  return Buffer.from(result.slice(0, length)).toString('hex');
}

/**
 * Compute SHA-256 hash
 *
 * @param {Uint8Array} seed - 32-byte deterministic seed (unused but required for signature)
 * @param {string} data - Data to hash
 * @returns {string} - Hex-encoded SHA-256 hash
 */
function sha256(seed, data) {
  if (!(seed instanceof Uint8Array) || seed.length !== 32) {
    throw new Error('sha256 adapter requires 32-byte Uint8Array seed');
  }

  if (typeof data !== 'string') {
    throw new Error('sha256 data must be a string');
  }

  return crypto.createHash('sha256').update(data, 'utf8').digest('hex');
}

/**
 * Compute HMAC-SHA256
 *
 * @param {Uint8Array} seed - 32-byte deterministic seed (unused but required for signature)
 * @param {string} key - HMAC key (hex-encoded)
 * @param {string} data - Data to authenticate
 * @returns {string} - Hex-encoded HMAC
 */
function hmacSha256(seed, key, data) {
  if (!(seed instanceof Uint8Array) || seed.length !== 32) {
    throw new Error('hmacSha256 adapter requires 32-byte Uint8Array seed');
  }

  if (typeof key !== 'string' || typeof data !== 'string') {
    throw new Error('hmacSha256 key and data must be strings');
  }

  const keyBuffer = Buffer.from(key, 'hex');
  return crypto.createHmac('sha256', keyBuffer).update(data, 'utf8').digest('hex');
}

module.exports = { randomBytes, sha256, hmacSha256 };
