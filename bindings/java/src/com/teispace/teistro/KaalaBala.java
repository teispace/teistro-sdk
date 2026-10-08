package com.teispace.teistro;

/**
 * A graha's Kaala bala by component, virupas.
 *
 * @param nathonnatha From the hour, 0 to 60.
 * @param paksha From the Moon's elongation, the Moon's doubled.
 * @param tribhaga 60 to the lord of the third of the day or night, and to Jupiter.
 * @param abda 15 to the year's lord.
 * @param masa 30 to the month's lord.
 * @param vara 45 to the weekday's lord.
 * @param hora 60 to the hour's lord.
 * @param ayana From its declination.
 * @param yuddha Gained by the victor and lost by the vanquished of a planetary war.
 */
public record KaalaBala(
        double nathonnatha,
        double paksha,
        double tribhaga,
        double abda,
        double masa,
        double vara,
        double hora,
        double ayana,
        double yuddha) {
    /**
     * The nine together, summed in the order every binding sums them.
     *
     * @return virupas
     */
    public double total() {
        return nathonnatha + paksha + tribhaga + vara + hora + ayana + abda + masa + yuddha;
    }
}
