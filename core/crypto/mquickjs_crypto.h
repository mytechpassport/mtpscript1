/*
 * MTPScript Crypto Module - ECDSA-P256 Signature Verification
 */

#ifndef MQUICKJS_CRYPTO_H
#define MQUICKJS_CRYPTO_H

#include "mquickjs.h"

typedef struct {
    uint8_t x[32];  /* X coordinate */
    uint8_t y[32];  /* Y coordinate */
} ECDSAPublicKey;

/* Embedded public key for snapshot verification */
extern const ECDSAPublicKey mtpscript_public_key;

/* Verify ECDSA-P256 signature */
JS_BOOL JS_VerifySnapshotSignature(const uint8_t *data, size_t data_len,
                                   const uint8_t *signature, size_t sig_len,
                                   const ECDSAPublicKey *pubkey);

/* Load and verify snapshot with signature */
JSValue JS_LoadSnapshot(JSContext *ctx, const uint8_t *snapshot_data, size_t snapshot_len,
                        const uint8_t *signature_data, size_t sig_len);

#endif /* MQUICKJS_CRYPTO_H */
