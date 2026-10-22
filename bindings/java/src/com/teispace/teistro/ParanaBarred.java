package com.teispace.teistro;

/**
 * A rule's nakshatra bar met on the parana day (C700).
 *
 * @param bar the rule's bar
 * @param span the Moon's stay in the barred share
 * @param setAside whether the bar was set aside, as the share outlasts the
 *     tithi the meal is kept in
 */
public record ParanaBarred(NakshatraBar bar, Interval span, boolean setAside) {}
