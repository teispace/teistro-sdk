package com.teispace.teistro;

/**
 * A count from sunrise in ghatis of sixty palas of sixty vipalas.
 *
 * @param ghati ghatis, 0 to 59 (60 when a civil day outlasts twenty-four hours)
 * @param pala palas, 0 to 59
 * @param vipala vipalas, 0 to 59
 */
public record GhatiPala(int ghati, int pala, int vipala) {}
