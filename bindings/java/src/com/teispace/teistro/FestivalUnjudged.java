package com.teispace.teistro;

/**
 * An occurrence no day could be given to, and why.
 *
 * @param rule the rule's key
 * @param tithi the occurrence
 * @param why why no day was given
 */
public record FestivalUnjudged(String rule, Interval tithi, String why) {}
