package com.teispace.teistro;

/**
 * One Jovian year of the Surya Siddhanta's count (I.55).
 *
 * @param member the year's name
 * @param count the signs mean Jupiter had crossed since the Kali age began, from 0
 * @param from when mean Jupiter entered the sign, a UTC Julian day
 * @param to when it entered the next, a UTC Julian day
 */
public record JovianYear(Samvatsara member, int count, double from, double to) {}
