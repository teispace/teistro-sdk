package com.teispace.teistro;

/**
 * What a period already running did to an ashauch ({@code 03-design/ashauch.md}).
 *
 * @param overlap the tag: {@code ENDS_WITH_EARLIER} (it ends with the earlier period, which is no
 *     shorter), {@code LAST_NIGHT} (known in the earlier equal period's last night: its end and
 *     {@code days} more) or {@code MOTHER_AFTER_FATHER} (the father's end and a pakshini)
 * @param earlier the earlier answer's key
 * @param days the days added, for {@code LAST_NIGHT}; null otherwise
 */
public record AshauchOverlapped(String overlap, String earlier, Integer days) {}
