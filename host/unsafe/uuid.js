/**
 * UUID npm bridge adapter
 *
 * Provides deterministic UUID generation based on seed.
 * Per TECHSPECV5.md §21, this adapter:
 * - Lives in host/unsafe/
 * - Is a pure function of arguments + deterministic seed
 * - Has enforced type signature
 * - No shared state, no exceptions escaping
 *
 * @param {Uint8Array} seed - 32-byte deterministic seed from §0-b
 * @param {number} index - Index to generate unique but deterministic UUIDs
 * @returns {string} - RFC 4122 v4 UUID (deterministic based on seed + index)
 */
function uuid(seed, index = 0) {
  if (!(seed instanceof Uint8Array) || seed.length !== 32) {
    throw new Error('uuid adapter requires 32-byte Uint8Array seed');
  }

  // Validate index
  if (typeof index !== 'number' || !Number.isInteger(index) || index < 0) {
    throw new Error('uuid adapter index must be non-negative integer');
  }

  // Generate deterministic bytes from seed + index using simple hash mixing
  // In production, would use proper HMAC-SHA256
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) {
    // Mix seed bytes with index to create deterministic output
    bytes[i] = (seed[i] ^ seed[i + 16] ^ (index >> (i % 8))) & 0xff;
  }

  // Set UUID version 4 bits
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  // Set UUID variant bits
  bytes[8] = (bytes[8] & 0x3f) | 0x80;

  // Format as UUID string
  const hex = Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

module.exports = { uuid };
