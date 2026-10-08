package com.teispace.teistro;

/**
 * What a koota read, one record a koota, each carrying its {@code koota}.
 */
public sealed interface KootaReading
        permits VarnaKoota, VashyaKoota, TaraKoota, YoniKoota, MaitriKoota, GanaKoota, BhakootKoota,
                NadiKoota {
    /**
     * Which koota this is.
     *
     * @return the koota
     */
    Koota koota();
}
