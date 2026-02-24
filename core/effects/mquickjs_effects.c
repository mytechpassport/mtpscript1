/*
 * MTPScript Effect System Implementation
 */

#include <string.h>
#include <stdlib.h>
#include <stdbool.h>
#include "mquickjs.h"
#include "mquickjs_effects.h"

#define MAX_EFFECTS 64

typedef struct {
    char *name;
    JSEffectHandler handler;
} JSEffectEntry;

typedef struct {
    char *promise_hash;
    int cont_id;
    uint8_t seed[32];
    JSValue result;
    bool has_result;
} JSIoCacheEntry;

typedef struct {
    JSEffectEntry effects[MAX_EFFECTS];
    int count;
    JSDeclaredEffects declared_effects;
    uint8_t execution_seed[32];
    bool has_seed;
    JSIoCacheEntry io_cache[1024];
    int cache_count;
} JSEffectRegistry;

/* Get effect registry from context (stored in opaque) */
static JSEffectRegistry *get_effect_registry(JSContext *ctx) {
    JSEffectRegistry *registry = JS_GetContextOpaque(ctx);
    if (!registry) {
        registry = calloc(1, sizeof(JSEffectRegistry));
        if (!registry) return NULL;
        JS_SetContextOpaque(ctx, registry);
    }
    return registry;
}

/* Register an effect handler */
JS_BOOL JS_RegisterEffect(JSContext *ctx, const char *name, JSEffectHandler handler) {
    JSEffectRegistry *registry = get_effect_registry(ctx);
    if (!registry || registry->count >= MAX_EFFECTS) {
        return 0;
    }

    /* Check if effect already exists */
    for (int i = 0; i < registry->count; i++) {
        if (strcmp(registry->effects[i].name, name) == 0) {
            return 0; /* Already registered */
        }
    }

    registry->effects[registry->count].name = strdup(name);
    registry->effects[registry->count].handler = handler;
    registry->count++;

    return 1;
}

/* Call an effect */
JSValue JS_CallEffect(JSContext *ctx, const char *name, const uint8_t *seed, size_t seed_len,
                      JSValue args) {
    JSEffectRegistry *registry = get_effect_registry(ctx);
    if (!registry) {
        return JS_ThrowError(ctx, JS_CLASS_INTERNAL_ERROR, "Effect system not initialized");
    }

    /* Runtime enforcement: check if effect is declared */
    if (!JS_IsEffectDeclared(ctx, name)) {
        return JS_ThrowError(ctx, JS_CLASS_TYPE_ERROR,
                           "Undeclared effect usage blocked by runtime enforcement: %s", name);
    }

    for (int i = 0; i < registry->count; i++) {
        if (strcmp(registry->effects[i].name, name) == 0) {
            return registry->effects[i].handler(ctx, seed, seed_len, args);
        }
    }

    return JS_ThrowError(ctx, JS_CLASS_TYPE_ERROR, "Unknown effect: %s", name);
}

/* Runtime enforcement: set declared effects for this context */
JS_BOOL JS_SetDeclaredEffects(JSContext *ctx, const char **effects, int count) {
    JSEffectRegistry *registry = get_effect_registry(ctx);
    if (!registry || count > MAX_DECLARED_EFFECTS) {
        return 0;
    }

    /* Clear existing declared effects */
    for (int i = 0; i < registry->declared_effects.count; i++) {
        free(registry->declared_effects.effects[i]);
    }
    registry->declared_effects.count = 0;

    /* Set new declared effects */
    for (int i = 0; i < count; i++) {
        registry->declared_effects.effects[i] = strdup(effects[i]);
        if (!registry->declared_effects.effects[i]) {
            /* Cleanup on failure */
            for (int j = 0; j < i; j++) {
                free(registry->declared_effects.effects[j]);
            }
            registry->declared_effects.count = 0;
            return 0;
        }
    }
    registry->declared_effects.count = count;

    return 1;
}

/* Runtime enforcement: check if effect is declared */
JS_BOOL JS_IsEffectDeclared(JSContext *ctx, const char *effect_name) {
    JSEffectRegistry *registry = JS_GetContextOpaque(ctx);
    if (!registry) {
        return 0; /* No registry means no effects declared */
    }

    for (int i = 0; i < registry->declared_effects.count; i++) {
        if (strcmp(registry->declared_effects.effects[i], effect_name) == 0) {
            return 1;
        }
    }

    return 0;
}

/* Deterministic I/O caching: set execution seed */
JS_BOOL JS_SetExecutionSeed(JSContext *ctx, const uint8_t *seed, size_t seed_len) {
    if (seed_len != 32) return 0;

    JSEffectRegistry *registry = get_effect_registry(ctx);
    if (!registry) return 0;

    memcpy(registry->execution_seed, seed, 32);
    registry->has_seed = 1;
    return 1;
}

/* Helper: generate cache key from (seed, cont_id) */
static void generate_cache_key(const uint8_t *seed, int cont_id, uint8_t key[32]) {
    /* Simple key generation: hash(seed + cont_id) */
    uint8_t data[32 + sizeof(int)];
    memcpy(data, seed, 32);
    memcpy(data + 32, &cont_id, sizeof(int));

    /* Use simple XOR-based hash for determinism (in real impl, use SHA-256) */
    memset(key, 0, 32);
    for (int i = 0; i < sizeof(data); i++) {
        key[i % 32] ^= data[i];
    }
}

/* Check I/O cache for deterministic replay */
static JSValue check_io_cache(JSEffectRegistry *registry, const char *promise_hash, int cont_id) {
    if (!registry->has_seed) return JS_UNDEFINED;

    uint8_t cache_key[32];
    generate_cache_key(registry->execution_seed, cont_id, cache_key);

    /* Simple linear search (in real impl, use hash table) */
    for (int i = 0; i < registry->cache_count; i++) {
        JSIoCacheEntry *entry = &registry->io_cache[i];
        if (entry->cont_id == cont_id &&
            memcmp(entry->seed, registry->execution_seed, 32) == 0 &&
            strcmp(entry->promise_hash, promise_hash) == 0) {
            return entry->result; /* Return cached result */
        }
    }

    return JS_UNDEFINED;
}

/* Store result in I/O cache */
static void store_io_cache(JSEffectRegistry *registry, const char *promise_hash,
                          int cont_id, JSValue result) {
    if (!registry->has_seed || registry->cache_count >= 1024) return;

    JSIoCacheEntry *entry = &registry->io_cache[registry->cache_count++];
    entry->promise_hash = strdup(promise_hash);
    entry->cont_id = cont_id;
    memcpy(entry->seed, registry->execution_seed, 32);
    entry->result = result;
    entry->has_result = 1;
}

/* Async await implementation with runtime enforcement and I/O caching */
JSValue JS_AsyncAwait(JSContext *ctx, const char *promise_hash, int cont_id, JSValue effect_args) {
    JSEffectRegistry *registry = JS_GetContextOpaque(ctx);

    /* Runtime enforcement: check if Async effect is declared */
    if (!JS_IsEffectDeclared(ctx, "Async")) {
        return JS_ThrowError(ctx, JS_CLASS_TYPE_ERROR,
                           "Undeclared Async effect usage blocked by runtime enforcement");
    }

    /* Check I/O cache for deterministic replay */
    if (registry) {
        JSValue cached = check_io_cache(registry, promise_hash, cont_id);
        if (!JS_IsUndefined(cached)) {
            return cached; /* Return cached result for replay determinism */
        }
    }

    /* Execute I/O synchronously (block until completion) */
    JSValue result;

    /* Check if this is a known promise hash */
    if (strcmp(promise_hash, "mock_http_get") == 0) {
        /* Mock HTTP GET result - in real impl, this would make actual HTTP call */
        result = JS_NewString(ctx, "{\"status\": 200, \"body\": \"Hello World\"}");
    } else if (strcmp(promise_hash, "mock_db_query") == 0) {
        /* Mock database query result - in real impl, this would make actual DB call */
        result = JS_NewString(ctx, "[{\"id\": 1, \"name\": \"test\"}]");
    } else {
        /* Unknown promise hash */
        return JS_ThrowError(ctx, JS_CLASS_TYPE_ERROR, "Unknown async effect: %s", promise_hash);
    }

    /* Cache result for deterministic replay */
    if (registry) {
        store_io_cache(registry, promise_hash, cont_id, result);
    }

    return result;
}

/* Clean up effect registry */
void cleanup_effects(JSContext *ctx) {
    JSEffectRegistry *registry = JS_GetContextOpaque(ctx);
    if (registry) {
        /* Free registered effects */
        for (int i = 0; i < registry->count; i++) {
            free(registry->effects[i].name);
        }
        /* Free declared effects */
        for (int i = 0; i < registry->declared_effects.count; i++) {
            free(registry->declared_effects.effects[i]);
        }
        /* Free I/O cache */
        for (int i = 0; i < registry->cache_count; i++) {
            free(registry->io_cache[i].promise_hash);
            // JS_FreeValue(ctx, registry->io_cache[i].result); // No-op for now
        }
        free(registry);
        JS_SetContextOpaque(ctx, NULL);
    }
}
