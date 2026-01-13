/**
 * DateTime npm bridge adapter
 *
 * Provides deterministic datetime operations based on seed.
 * Per TECHSPECV5.md §21, this adapter:
 * - Lives in host/unsafe/
 * - Is a pure function of arguments + deterministic seed
 * - Has enforced type signature
 * - No shared state, no exceptions escaping
 *
 * Real wall-clock time is NOT used - all times are derived from seed.
 */

/**
 * Get deterministic "current" time from seed
 * This returns a deterministic timestamp derived from the seed,
 * NOT the actual wall-clock time (which would break determinism).
 *
 * @param {Uint8Array} seed - 32-byte deterministic seed from §0-b
 * @returns {number} - Unix timestamp in milliseconds (deterministic)
 */
function now(seed) {
  if (!(seed instanceof Uint8Array) || seed.length !== 32) {
    throw new Error('now adapter requires 32-byte Uint8Array seed');
  }

  // Derive a deterministic timestamp from seed
  // Use first 8 bytes as a 64-bit value, then scale to reasonable timestamp range
  let value = BigInt(0);
  for (let i = 0; i < 8; i++) {
    value = (value << BigInt(8)) | BigInt(seed[i]);
  }

  // Base timestamp: 2024-01-01T00:00:00Z = 1704067200000ms
  // Add up to ~1 year of offset based on seed
  const baseTimestamp = 1704067200000n;
  const maxOffset = 31536000000n; // ~1 year in ms
  const offset = value % maxOffset;

  return Number(baseTimestamp + offset);
}

/**
 * Format a timestamp as ISO 8601 string
 *
 * @param {Uint8Array} seed - 32-byte deterministic seed (unused but required for signature)
 * @param {number} timestamp - Unix timestamp in milliseconds
 * @returns {string} - ISO 8601 formatted string
 */
function toIsoString(seed, timestamp) {
  if (!(seed instanceof Uint8Array) || seed.length !== 32) {
    throw new Error('toIsoString adapter requires 32-byte Uint8Array seed');
  }

  if (typeof timestamp !== 'number' || !Number.isFinite(timestamp)) {
    throw new Error('toIsoString timestamp must be a finite number');
  }

  return new Date(timestamp).toISOString();
}

/**
 * Parse ISO 8601 string to timestamp
 *
 * @param {Uint8Array} seed - 32-byte deterministic seed (unused but required for signature)
 * @param {string} isoString - ISO 8601 formatted string
 * @returns {number} - Unix timestamp in milliseconds
 */
function parseIso(seed, isoString) {
  if (!(seed instanceof Uint8Array) || seed.length !== 32) {
    throw new Error('parseIso adapter requires 32-byte Uint8Array seed');
  }

  if (typeof isoString !== 'string') {
    throw new Error('parseIso isoString must be a string');
  }

  const timestamp = Date.parse(isoString);
  if (Number.isNaN(timestamp)) {
    throw new Error('parseIso: invalid ISO 8601 string');
  }

  return timestamp;
}

/**
 * Add duration to timestamp
 *
 * @param {Uint8Array} seed - 32-byte deterministic seed (unused but required for signature)
 * @param {number} timestamp - Unix timestamp in milliseconds
 * @param {number} durationMs - Duration to add in milliseconds
 * @returns {number} - New timestamp
 */
function addMs(seed, timestamp, durationMs) {
  if (!(seed instanceof Uint8Array) || seed.length !== 32) {
    throw new Error('addMs adapter requires 32-byte Uint8Array seed');
  }

  if (typeof timestamp !== 'number' || !Number.isFinite(timestamp)) {
    throw new Error('addMs timestamp must be a finite number');
  }

  if (typeof durationMs !== 'number' || !Number.isFinite(durationMs)) {
    throw new Error('addMs durationMs must be a finite number');
  }

  return timestamp + durationMs;
}

module.exports = { now, toIsoString, parseIso, addMs };
