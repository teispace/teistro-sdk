package com.teispace.teistro;

import java.util.List;

/**
 * A chart's karakamsha: the Atmakaraka's navamsha sign (BPHS ch. 33 v. 1).
 *
 * @param atmakaraka The Atmakaraka, under {@code jaimini.chara_karakas}.
 * @param sign The karakamsha, the Atmakaraka's navamsha sign.
 * @param inRasi Each graha's house from it in the rasi chart, 1 to 12, the Sun to Ketu.
 * @param inNavamsha Each graha's house from it in the navamsha, 1 to 12, the Sun to Ketu (C130).
 */
public record Karakamsha(
        Graha atmakaraka,
        Rashi sign,
        List<Integer> inRasi,
        List<Integer> inNavamsha) {
    /** The value, its lists copied and unmodifiable. */
    public Karakamsha {
        inRasi = List.copyOf(inRasi);
        inNavamsha = List.copyOf(inNavamsha);
    }
}
