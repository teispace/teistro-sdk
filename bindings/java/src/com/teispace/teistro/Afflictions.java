package com.teispace.teistro;

import java.util.List;

/**
 * The two lords' afflictions, clause by clause: what made a Rudda or a Durapha.
 *
 * @param lagnesha The lagnesha's.
 * @param karyesha The karyesha's.
 */
public record Afflictions(
        List<Affliction> lagnesha,
        List<Affliction> karyesha) {
    /** The value, its lists copied and unmodifiable. */
    public Afflictions {
        lagnesha = List.copyOf(lagnesha);
        karyesha = List.copyOf(karyesha);
    }
}
