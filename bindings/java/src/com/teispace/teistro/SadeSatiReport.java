package com.teispace.teistro;

import java.util.List;

/**
 * A chart's Sade Satis and smaller spells, each period <b>whole</b> however far its bounds fall
 * outside the window asked about.
 *
 * @param reference What the houses were counted from, and that point's sign.
 * @param reckoning Whole signs, or 30-degree houses centred on the point's degree.
 * @param sadeSati Every Sade Sati reaching into the window, in time order.
 * @param spells The smaller spells asked for, in time order.
 */
public record SadeSatiReport(
        GocharReference reference,
        Reckoning reckoning,
        List<SadeSati> sadeSati,
        List<SadeSatiSpell> spells) {
    /** The value, its lists copied and unmodifiable. */
    public SadeSatiReport {
        sadeSati = List.copyOf(sadeSati);
        spells = List.copyOf(spells);
    }
}
