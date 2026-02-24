/*
 * MTPScript Typed Error Implementation
 */

#include <stdio.h>
#include "mquickjs_errors.h"
#include "mquickjs.h"

/* Error code to string mapping */
static const char *error_code_names[] = {
    [MTP_ERROR_NONE] = "None",
    [MTP_ERROR_GAS_EXHAUSTED] = "GasExhausted",
    [MTP_ERROR_MEMORY_LIMIT] = "MemoryLimitExceeded",
    [MTP_ERROR_INVALID_DECIMAL] = "InvalidDecimal",
    [MTP_ERROR_OVERFLOW] = "IntegerOverflow",
    [MTP_ERROR_INVALID_EFFECT] = "InvalidEffect",
    [MTP_ERROR_SIGNATURE_INVALID] = "InvalidSignature",
    [MTP_ERROR_FORBIDDEN_SYNTAX] = "ForbiddenSyntax",
};

/* Create a typed error with canonical JSON format */
JSValue JS_ThrowTypedError(JSContext *ctx, MTPScriptErrorCode code, const char *message) {
    char json_buf[512];
    const char *error_name = "UnknownError";

    if (code >= 0 && code < sizeof(error_code_names) / sizeof(error_code_names[0])) {
        error_name = error_code_names[code];
    }

    /* Create canonical JSON error format */
    snprintf(json_buf, sizeof(json_buf),
             "{\"error\":\"%s\",\"code\":%d,\"message\":\"%s\"}",
             error_name, (int)code, message ? message : "");

    /* Create and throw the error */
    JSValue error_obj = JS_NewObject(ctx);
    if (!JS_IsException(error_obj)) {
        JSValue code_val = JS_NewInt32(ctx, code);
        JSValue message_val = JS_NewString(ctx, message ? message : "");

        JS_SetPropertyStr(ctx, error_obj, "code", code_val);
        JS_SetPropertyStr(ctx, error_obj, "message", message_val);
        JS_SetPropertyStr(ctx, error_obj, "error", JS_NewString(ctx, error_name));
    }

    return JS_Throw(ctx, error_obj);
}
