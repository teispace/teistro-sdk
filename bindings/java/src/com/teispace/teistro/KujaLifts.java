package com.teispace.teistro;

import java.util.List;
import java.util.Optional;

/**
 * The lifts of the Kuja dosha that hold on one side, each a clause (C288, C755): BPHS ch. 80
 * v. 47's benefic aspect or conjunction and vv. 48–49's dosha in both charts (rank 1), then the
 * five the almanacs printed in Nepal give, from the lagna (rank 4).
 *
 * @param benefic A benefic aspects Mars or stands with it.
 * @param mutual The other chart carries the dosha too.
 * @param saturn Saturn in the 1st, 4th, 7th, 9th or 12th.
 * @param moonVenusSecond The Moon and Venus both in the 2nd.
 * @param jupiterAspect Jupiter aspects Mars.
 * @param rahuKendra Rahu in a kendra.
 * @param withRahu Mars with Rahu.
 */
public record KujaLifts(
        boolean benefic,
        boolean mutual,
        boolean saturn,
        boolean moonVenusSecond,
        boolean jupiterAspect,
        boolean rahuKendra,
        boolean withRahu) {
    /**
     * A side's lifts from the boundary's bit set.
     *
     * @param bits the byte, bit 0 {@code benefic} to bit 6 {@code withRahu}
     * @return the lifts, or empty where bit 7 says none were read
     */
    static Optional<KujaLifts> ofBits(int bits) {
        if ((bits & 0x80) == 0) {
            return Optional.empty();
        }
        return Optional.of(new KujaLifts(
                (bits & 1) != 0,
                (bits & 1 << 1) != 0,
                (bits & 1 << 2) != 0,
                (bits & 1 << 3) != 0,
                (bits & 1 << 4) != 0,
                (bits & 1 << 5) != 0,
                (bits & 1 << 6) != 0));
    }

    /**
     * The clauses in their order.
     *
     * @return each clause, {@code benefic} first
     */
    public List<Boolean> clauses() {
        return List.of(benefic, mutual, saturn, moonVenusSecond, jupiterAspect, rahuKendra, withRahu);
    }
}
