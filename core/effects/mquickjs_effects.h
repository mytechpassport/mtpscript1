/*
 * MTPScript Effect System
 */

#ifndef MQUICKJS_EFFECTS_H
#define MQUICKJS_EFFECTS_H

#include "mquickjs.h"

typedef JSValue (*JSEffectHandler)(JSContext *ctx, const uint8_t *seed, size_t seed_len,
                                   JSValue args);

typedef enum {
    EFFECT_ASYNC_AWAIT,
    EFFECT_CUSTOM,
} JSEffectType;

/* Runtime effect enforcement */
#define MAX_DECLARED_EFFECTS 64

typedef struct {
    char *effects[MAX_DECLARED_EFFECTS];
    int count;
} JSDeclaredEffects;

/* Register an effect handler */
JS_BOOL JS_RegisterEffect(JSContext *ctx, const char *name, JSEffectHandler handler);

/* Call an effect (internal use) */
JSValue JS_CallEffect(JSContext *ctx, const char *name, const uint8_t *seed, size_t seed_len,
                      JSValue args);

/* Runtime enforcement: set declared effects for this context */
JS_BOOL JS_SetDeclaredEffects(JSContext *ctx, const char **effects, int count);

/* Runtime enforcement: check if effect is declared */
JS_BOOL JS_IsEffectDeclared(JSContext *ctx, const char *effect_name);

/* Async effect support */
JSValue JS_AsyncAwait(JSContext *ctx, const char *promise_hash, int cont_id, JSValue effect_args);

/* Deterministic I/O caching */
JS_BOOL JS_SetExecutionSeed(JSContext *ctx, const uint8_t *seed, size_t seed_len);

/* Cleanup effects (internal) */
void cleanup_effects(JSContext *ctx);

#endif /* MQUICKJS_EFFECTS_H */
