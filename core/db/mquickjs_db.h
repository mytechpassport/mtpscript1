/*
 * MTPScript Database Effects Implementation
 * Specification §7 - DbRead, DbWrite Effects
 */

#ifndef MQUICKJS_DB_H
#define MQUICKJS_DB_H

#include "mquickjs.h"
#include <mysql/mysql.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifndef size_t
#define size_t unsigned long
#endif

// Database connection pool
typedef struct {
    MYSQL *connections[16];  // Max 16 connections per request
    int count;
    int max_connections;
} MTPScriptDBPool;

// Database query parameters
typedef struct {
    char *name;
    char *value;
} MTPScriptDBParam;

// Database effect cache entry
typedef struct {
    uint8_t cache_key[32];   // SHA-256 of (seed, query, params)
    JSValue result;          // Cached JS result value
    bool has_result;
} MTPScriptDBCacheEntry;

// Database effect cache
typedef struct {
    MTPScriptDBCacheEntry entries[1024];  // Simple array cache
    int count;
    uint8_t execution_seed[32];
    bool has_seed;
} MTPScriptDBCache;

// Initialize database connection pool
MTPScriptDBPool *mtpscript_db_pool_new(void);
void mtpscript_db_pool_free(MTPScriptDBPool *pool);

// Get database connection from pool
MYSQL *mtpscript_db_get_connection(MTPScriptDBPool *pool);

// Execute query with caching (DbRead)
JSValue mtpscript_db_read(JSContext *ctx, const uint8_t *seed, size_t seed_len, JSValue args);

// Execute write operation (DbWrite)
JSValue mtpscript_db_write(JSContext *ctx, const uint8_t *seed, size_t seed_len, JSValue args);

// Database cache management
MTPScriptDBCache *mtpscript_db_cache_new(void);
void mtpscript_db_cache_free(MTPScriptDBCache *cache);
JSValue mtpscript_db_cache_get(MTPScriptDBCache *cache, const uint8_t *cache_key);
void mtpscript_db_cache_put(MTPScriptDBCache *cache, const uint8_t *cache_key, JSValue result);

// Register database effects
void mtpscript_db_register_effects(JSContext *ctx);

#endif /* MQUICKJS_DB_H */
