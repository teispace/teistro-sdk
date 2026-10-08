package com.teispace.teistro;

/**
 * Something the rules ask that the SDK does not judge yet, and why.
 *
 * @param what what the rules ask
 * @param why why it is not judged
 */
public record MuhurtaUnjudged(String what, String why) {}
