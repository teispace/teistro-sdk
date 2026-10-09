package com.teispace.teistro;

import java.util.Optional;

/**
 * The three reports a candidate birth time gives beside the purifier: the pranapada's house, the
 * nisheka BPHS ch. 3 vv. 25–29 counts back to, and <i>Brihat Jataka</i> IV.21's Moon at that
 * conception ({@code 03-design/rectification.md}).
 *
 * @param birth The candidate, a Julian day (UTC).
 * @param pranapadaHouse The pranapada's house, as Jha's print judges the birth.
 * @param nisheka The conception BPHS counts back to.
 * @param moon BJ IV.21 read at that conception against the candidate.
 */
public record Conception(
        double birth,
        PranapadaHouse pranapadaHouse,
        Nisheka nisheka,
        Moon moon) {
    /**
     * The pranapada's house from the lagna, and the birth it judges.
     *
     * @param pranapadaDeg The pranapada, degrees.
     * @param lagnaDeg The lagna, degrees.
     * @param house Its house from the lagna, one to twelve.
     * @param auspicious Whether that house is auspicious (Jha's ch. 3 vv. 73–74).
     */
    public record PranapadaHouse(
            double pranapadaDeg,
            double lagnaDeg,
            int house,
            boolean auspicious) {
    }

    /**
     * The conception, and its lagna judged.
     *
     * @param count The count back from the birth.
     * @param lagnaDeg The conception's lagna, degrees, in the conception chart's own zodiac.
     * @param verdict That lagna under the purifier, at the birth's place (v. 29: "purify it as
     *     before").
     */
    public record Nisheka(
            NishekaCount count,
            double lagnaDeg,
            Purified.Verdict verdict) {
    }

    /**
     * The conception a candidate birth counts back to: the points read at the birth, the span they
     * give, and the instant.
     *
     * @param points The points read at the birth.
     * @param span The span they give.
     * @param instant The conception, a Julian day (UTC): the birth less the span.
     * @param daysPerBirthMinute How many days the conception moves when the birth moves a minute
     *     later, measured across the minute: why it is read at an instant.
     */
    public record NishekaCount(
            NishekaPoints points,
            NishekaSpan span,
            double instant,
            double daysPerBirthMinute) {
    }

    /**
     * The points v. 27 reads, at the birth.
     *
     * @param mandiDeg Mandi, degrees.
     * @param saturnDeg Saturn's point, degrees.
     * @param lagnaDeg The lagna, degrees.
     * @param ninthDeg The 9th bhava's point, degrees.
     * @param lagnaLordDeg The lagna's lord's longitude, degrees.
     * @param moonDeg The Moon, degrees.
     */
    public record NishekaPoints(
            double mandiDeg,
            double saturnDeg,
            double lagnaDeg,
            double ninthDeg,
            double lagnaLordDeg,
            double moonDeg) {
    }

    /**
     * v. 27's two arcs, their sum, and the span before birth they give.
     *
     * @param saturnToMandiDeg From Saturn forward to Mandi, degrees.
     * @param lagnaToNinthDeg From the lagna forward to the 9th bhava, degrees.
     * @param moonAddedDeg The Moon's degrees elapsed in her sign, added when the lagna's lord is in
     *     the invisible half (v. 28); empty when not added.
     * @param arcDeg The whole arc, degrees.
     * @param written The arc written as months, days, ghatis and palas, to the second.
     * @param daysBefore The span before birth, in days, under the month length.
     */
    public record NishekaSpan(
            double saturnToMandiDeg,
            double lagnaToNinthDeg,
            Optional<Double> moonAddedDeg,
            double arcDeg,
            MonthsBefore written,
            double daysBefore) {
    }

    /**
     * The arc v. 28 reads, written as v. 28 reads it.
     *
     * @param months Signs, read as months.
     * @param days Degrees, read as days.
     * @param ghatis Arc-minutes, read as ghatis.
     * @param palas Arc-seconds, read as palas.
     */
    public record MonthsBefore(
            int months,
            int days,
            int ghatis,
            int palas) {
    }

    /**
     * BJ IV.21 read at a conception and set against the candidate birth.
     *
     * @param predicted The count from the conception's Moon.
     * @param moonSign The candidate's Moon sign.
     * @param moonNakshatra The candidate's Moon nakshatra; empty where none is read.
     * @param signAgrees Whether the candidate's Moon is in the predicted sign.
     * @param nakshatraAgrees Whether it is in the predicted nakshatra; empty where none is
     *     predicted.
     * @param rising The conception's rising sign or navamsha.
     * @param predictedPart Its class: {@code DAY}, {@code NIGHT} or {@code EITHER}.
     * @param bornByDay Whether the candidate is born by day.
     * @param partAgrees Whether the candidate's day or night is the predicted one.
     * @param risenFraction How much of the rising sign or navamsha had risen at conception, by
     *     rising time.
     * @param elapsedFraction How much of the candidate's day or night had passed at birth.
     */
    public record Moon(
            MoonCount predicted,
            Rashi moonSign,
            Optional<Nakshatra> moonNakshatra,
            boolean signAgrees,
            Optional<Boolean> nakshatraAgrees,
            Rashi rising,
            String predictedPart,
            boolean bornByDay,
            boolean partAgrees,
            double risenFraction,
            double elapsedFraction) {
    }

    /**
     * What BJ IV.21 predicts from the Moon at conception.
     *
     * @param dvadashamsha The dvadashamsha the Moon occupies in her sign, one to twelve.
     * @param sign The sign the Moon holds at birth.
     * @param nakshatra The nakshatra Bhattotpala's proportion places her in, under
     *     {@code NEXT_AFTER_DVADASHAMSHA} only; empty under the other counts.
     */
    public record MoonCount(
            int dvadashamsha,
            Rashi sign,
            Optional<Nakshatra> nakshatra) {
    }
}
