package com.teispace.teistro;

/**
 * What a crash reporter hears of an {@code INTERNAL} refusal, through
 * {@link Teistro#onInternalError}. These parts are all it hears: there is no
 * message, on purpose, since one may quote what the call was given.
 *
 * @param status the status, as its key: {@code INTERNAL}
 * @param code {@link Teistro#INTERNAL_CODE}
 * @param kind {@code boundary}: a panic the C ABI caught, the only kind
 *     outside a wasm module
 * @param sdkVersion the SDK that failed
 */
public record InternalReport(String status, int code, String kind, String sdkVersion) {}
