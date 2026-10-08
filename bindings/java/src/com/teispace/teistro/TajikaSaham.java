package com.teispace.teistro;

import java.util.List;

/**
 * Where a saham fell in a year's chart, and what it fell in.
 *
 * @param saham Which of the forty-one.
 * @param longitudeDeg Where it fell, sidereal degrees in [0, 360).
 * @param sign The sign it fell in.
 * @param lord That sign's lord: the saham's lord, by whose strength the source judges it.
 * @param house The house it fell in, 1 to 12, by whole signs from the annual lagna.
 * @param addedSign Whether it was carried a sign further because c did not fall between b and a.
 * @param strong The clauses of the source's strong list that hold. Reported and never weighed: the
 *     source gives no score, and three sahams in five meet clauses on both lists.
 * @param weak The clauses of the source's weak list that hold.
 * @param lordVishwa The saham lord's Panchavargiya Vishwa bala.
 * @param lordHarsha The saham lord's Harsha bala grade.
 * @param inNodeAxis Whether it stands in the Rahu-Ketu axis; null when the chart placed no nodes.
 *     May be null.
 * @param seven How each of the seven stands to it, in the catalogue's order.
 */
public record TajikaSaham(
        Saham saham,
        double longitudeDeg,
        Rashi sign,
        Graha lord,
        int house,
        boolean addedSign,
        List<SahamStrong> strong,
        List<SahamWeak> weak,
        Bala lordVishwa,
        HarshaGrade lordHarsha,
        Boolean inNodeAxis,
        List<SahamSeven> seven) {
    /** The value, its lists copied and unmodifiable. */
    public TajikaSaham {
        strong = List.copyOf(strong);
        weak = List.copyOf(weak);
        seven = List.copyOf(seven);
    }

    /**
     * In the 6th, 8th or 12th, where the source calls a saham handicapped.
     *
     * @return true when handicapped
     */
    public boolean handicapped() {
        return house == 6 || house == 8 || house == 12;
    }
}
