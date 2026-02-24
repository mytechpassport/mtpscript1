/*
 * MTPScript Typed Error Codes
 */

#ifndef MQUICKJS_ERRORS_H
#define MQUICKJS_ERRORS_H

#include "mquickjs.h"

/* Create a typed error with canonical JSON format */
JSValue JS_ThrowTypedError(JSContext *ctx, MTPScriptErrorCode code, const char *message);

#endif /* MQUICKJS_ERRORS_H */
