package com.teispace.teistro;

import com.teispace.teistro.request.NightRule;

/**
 * Why an ashauch's first day is the day it is ({@code 03-design/ashauch.md}).
 *
 * @param by {@code DAYLIGHT}, the instant fell in the day's daylight, or {@code NIGHT}, it fell in
 *     the night and {@code rule} gave the day
 * @param rule the night rule that gave the day; null by daylight
 */
public record AshauchDayOneBy(String by, NightRule rule) {}
