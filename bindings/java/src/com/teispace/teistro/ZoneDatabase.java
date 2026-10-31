package com.teispace.teistro;

import java.util.Map;

/**
 * The zone database a context reads ({@code 03-design/zone-packs.md} §4).
 *
 * @param source where it comes from: {@code EMBEDDED} or {@code PACK}
 * @param version its release ({@code 2026f}), as every answer's provenance names it
 * @param zones how many names it holds, links included
 * @param bundled the release compiled into this build: the fallback
 */
public record ZoneDatabase(String source, String version, int zones, String bundled) {
    static ZoneDatabase of(String json) {
        Map<?, ?> raw = Reads.object(Json.read(json));
        return new ZoneDatabase(Reads.string(raw, "source"), Reads.string(raw, "version"),
                Reads.integer(raw, "zones"), Reads.string(raw, "bundled"));
    }
}
