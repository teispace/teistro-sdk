package com.teispace.teistro;

import java.util.List;

/**
 * What {@code verify.receipt} answers: what a receipt was found to be, and
 * what moved between its side and this one
 * ({@code 03-design/receipts-and-datasets.md} §1.3).
 *
 * @param verdict {@code HELD}, {@code DIFFERS}, {@code NOT_RERUN} or {@code BROKEN}
 * @param record the record the receipt names
 * @param moved every field that moved, {@code output} first when the digest did
 * @param broken the digests that no longer hold, on a receipt {@code BROKEN}
 * @param issued what issued the receipt, as it says
 * @param now what checked it
 * @param reason why it was not answered again, on {@code NOT_RERUN}, or null
 */
public record ReceiptCheck(String verdict, String record, List<Move> moved, List<String> broken, Side issued,
        Side now, Reason reason) {

    /**
     * One field that moved, with both values as text.
     *
     * @param field {@code output}, {@code sdkVersion}, {@code calculationVersion}, {@code catalogueVersion},
     *        {@code provider} or {@code tzdb}
     * @param issued the receipt's
     * @param now this side's
     */
    public record Move(String field, String issued, String now) {}

    /**
     * One side of the check: its versions, ephemeris and zone database.
     *
     * @param sdkVersion the SDK's version, {@code major.minor.patch}
     * @param calculationVersion the calculation version
     * @param catalogueVersion the catalogue's version
     * @param provider the ephemeris, or null where none was read
     * @param tzdb the zone database's version
     */
    public record Side(String sdkVersion, int calculationVersion, int catalogueVersion, Provider provider,
            String tzdb) {}

    /**
     * An ephemeris, as a receipt names it.
     *
     * @param name its name
     * @param version its version
     * @param dataVersion its data's version
     */
    public record Provider(String name, String version, String dataVersion) {}

    /**
     * Why a receipt was not answered again.
     *
     * @param cause {@code RECORD}, {@code PROVIDER}, {@code SETTINGS}, {@code LOCALE}, {@code PACKS},
     *        {@code OUT_OF_RANGE} or {@code REFUSED}
     * @param detail what was missing or what refused, in words
     */
    public record Reason(String cause, String detail) {}
}
