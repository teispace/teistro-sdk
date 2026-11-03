package com.teispace.teistro;

/**
 * The ingress a lunation precedes.
 *
 * @param kind which ingress
 * @param instant when, a Julian day (UTC)
 */
public record MundaneIngress(MundaneKind kind, double instant) {
}
