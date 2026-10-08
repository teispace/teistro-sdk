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
        EdgeDistance boundaries) {
}
