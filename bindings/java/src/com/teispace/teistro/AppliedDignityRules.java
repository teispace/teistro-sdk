package com.teispace.teistro;

/**
 * The terms and triplicities a reading used; {@link Terms#TABLE} for the
 * request's own table.
 *
 * @param terms the terms
 * @param triplicities the triplicities
 */
public record AppliedDignityRules(Terms terms, Triplicities triplicities) {}
