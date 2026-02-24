/*
 * MTPScript Crypto Implementation - ECDSA-P256
 */

#include <string.h>
#include <openssl/evp.h>
#include <openssl/ecdsa.h>
#include <openssl/sha.h>
#include <openssl/ec.h>
#include <openssl/bn.h>
#include "mquickjs_crypto.h"

/* Embedded public key - in production this would be compiled in */
const ECDSAPublicKey mtpscript_public_key = {
    /* Placeholder values - replace with actual public key */
    .x = {0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
          0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
          0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
          0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F},
    .y = {0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
          0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
          0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
          0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F}
};

/* Verify ECDSA-P256 signature using OpenSSL */
JS_BOOL JS_VerifySnapshotSignature(const uint8_t *data, size_t data_len,
                                   const uint8_t *signature, size_t sig_len,
                                   const ECDSAPublicKey *pubkey) {
    int ret = 0;
    EC_KEY *ec_key = NULL;
    EC_GROUP *group = NULL;
    BIGNUM *x = NULL;
    BIGNUM *y = NULL;
    unsigned char hash[SHA256_DIGEST_LENGTH];
    ECDSA_SIG *ecdsa_sig = NULL;
    BIGNUM *r = NULL;
    BIGNUM *s = NULL;

    /* Basic validation */
    if (!data || !signature || !pubkey || data_len == 0 || sig_len != 64) {
        return 0;
    }

    /* Create EC key for P-256 curve */
    group = EC_GROUP_new_by_curve_name(NID_X9_62_prime256v1);
    if (!group) goto cleanup;

    ec_key = EC_KEY_new();
    if (!ec_key) goto cleanup;

    if (!EC_KEY_set_group(ec_key, group)) goto cleanup;

    /* Set public key from provided coordinates */
    x = BN_bin2bn(pubkey->x, 32, NULL);
    y = BN_bin2bn(pubkey->y, 32, NULL);
    if (!x || !y) goto cleanup;

    if (!EC_KEY_set_public_key_affine_coordinates(ec_key, x, y)) goto cleanup;

    /* Hash the data with SHA-256 */
    SHA256(data, data_len, hash);

    /* Parse signature - ECDSA-P256 uses DER format, but we expect raw r,s */
    /* For simplicity, assume signature is 64 bytes: r(32) + s(32) */
    r = BN_bin2bn(signature, 32, NULL);
    s = BN_bin2bn(signature + 32, 32, NULL);
    if (!r || !s) goto cleanup;

    ecdsa_sig = ECDSA_SIG_new();
    if (!ecdsa_sig) goto cleanup;

    if (!ECDSA_SIG_set0(ecdsa_sig, r, s)) goto cleanup;
    r = s = NULL; /* Ownership transferred */

    /* Verify the signature */
    ret = ECDSA_do_verify(hash, SHA256_DIGEST_LENGTH, ecdsa_sig, ec_key);

cleanup:
    if (ecdsa_sig) ECDSA_SIG_free(ecdsa_sig);
    if (r) BN_free(r);
    if (s) BN_free(s);
    if (x) BN_free(x);
    if (y) BN_free(y);
    if (ec_key) EC_KEY_free(ec_key);
    if (group) EC_GROUP_free(group);

    return ret == 1 ? 1 : 0;
}

/* Load and verify snapshot with signature */
JSValue JS_LoadSnapshot(JSContext *ctx, const uint8_t *snapshot_data, size_t snapshot_len,
                        const uint8_t *signature_data, size_t sig_len) {
    /* Verify signature first */
    if (!JS_VerifySnapshotSignature(snapshot_data, snapshot_len,
                                    signature_data, sig_len,
                                    &mtpscript_public_key)) {
        return JS_ThrowError(ctx, JS_CLASS_INTERNAL_ERROR,
                           "Snapshot signature verification failed");
    }

    /* Load the snapshot */
    return JS_LoadBytecode(ctx, snapshot_data);
}
