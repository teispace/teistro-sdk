package com.teispace.teistro;

/**
 * A graha's Sthana bala by component, virupas.
 *
 * @param uchcha From its distance to its debilitation point, 0 to 60.
 * @param saptavargaja From its dignity in the seven vargas.
 * @param ojayugma From its rasi's and navamsha's parity, 0, 15 or 30.
 * @param kendradi From its house: 60, 30 or 15.
 * @param drekkana From its decanate: 0 or 15.
 */
public record SthanaBala(
        double uchcha,
        double saptavargaja,
        double ojayugma,
        double kendradi,
        double drekkana) {
    /**
     * The five together.
     *
     * @return virupas
     */
    public double total() {
        return uchcha + saptavargaja + ojayugma + kendradi + drekkana;
    }
}
