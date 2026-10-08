package com.teispace.teistro;

/**
 * What one of the ten considerations read, one record each, each carrying its catalogue
 * {@code koota} (C282).
 */
public sealed interface PoruthamReading
        permits DhinamPorutham, GanamPorutham, MahendraPorutham, DeerghaPorutham, YoniPorutham, RasiPorutham,
                RasyadhipathiPorutham, VasyamPorutham, RajjuPorutham, VedhaiPorutham {
    /**
     * Which consideration this is, as its catalogue koota.
     *
     * @return the koota
     */
    Koota koota();
}
