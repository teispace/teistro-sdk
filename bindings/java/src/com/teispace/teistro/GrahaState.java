package com.teispace.teistro;

/**
 * What one graha <b>is</b>, as opposed to where it is.
 *
 * @param graha Which graha.
 * @param sign The sign it stands in.
 * @param house The bhava it falls in, under the chart's placement system.
 * @param dignity Its dignity.
 * @param friendship How it stands to its dispositor.
 * @param combustion What the Sun does to it.
 * @param age Which fifth of its sign it stands in.
 * @param wakefulness Awake, dreaming or asleep.
 * @param deeptadi The bright state, where the SDK can decide one. May be null.
 * @param lajjitadi The lajjitadi that hold, and the ones nothing decides.
 * @param war The war it is in, if it is in one. May be null.
 * @param sayanadi The Sayanadi state and its sub-states, or null for a body the verses give no
 *     number. May be null.
 * @param pushkaraNavamsha Whether it stands in one of its sign's two pushkara navamshas
 *     (Vidyamadhaviyam ch. 1 v. 18), or null for an outer planet. May be null.
 * @param pushkaraBhaga Whether it stands on its sign's pushkara bhaga in the table
 *     {@code state.pushkara_bhaga} names, or null for a body that table gives no row: Jataka
 *     Parijata's gives the Moon's alone. May be null.
 * @param maranaKarakaSthana Whether it stands in its marana karaka sthana (Jataka Parijata ch. 17
 *     vv. 34 and 35), or null for Ketu and the outer planets, which the verses do not name. May be
 *     null.
 * @param boundaries How near it stands to a classification boundary.
 */
public record GrahaState(
        Graha graha,
        Rashi sign,
        int house,
        Dignity dignity,
        Friendship friendship,
        Combustion combustion,
        AvasthaBaladi age,
        AvasthaJagradadi wakefulness,
        AvasthaDeeptadi deeptadi,
        Lajjitadi lajjitadi,
        War war,
        Sayanadi sayanadi,
        Boolean pushkaraNavamsha,
        Boolean pushkaraBhaga,
        Boolean maranaKarakaSthana,
        EdgeDistance boundaries) {
}
