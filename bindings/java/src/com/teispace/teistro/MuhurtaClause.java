package com.teispace.teistro;

/**
 * A clause and the interval it held over.
 *
 * @param kind the clause
 * @param at when it held
 */
public record MuhurtaClause(MuhurtaClauseKind kind, Interval at) {}
