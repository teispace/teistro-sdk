package com.teispace.teistro;

import java.util.LinkedHashMap;
import java.util.Map;

import com.teispace.teistro.blob.Naam;

/**
 * {@code sky.matching()}: what matches without a chart, two names star to
 * star ({@code 03-design/matching.md}, C291 to C296). A match of two
 * births is asked of the charts, through {@code matching} beside a chart
 * request.
 *
 * <pre>{@code
 * NaamMilan read = sky.matching().naam("सीता", "राम");
 * }</pre>
 */
public final class MatchingArea {
    private final Context context;

    /**
     * The matching area of a context.
     *
     * @param context the context every call goes through
     */
    MatchingArea(Context context) {
        this.context = context;
    }

    /**
     * Two names matched star to star under the sources' own readings.
     *
     * @param bride the bride's name
     * @param groom the groom's name
     * @return the match
     * @see #naam(String, String, Map)
     */
    public NaamMilan naam(String bride, String groom) {
        return naam(bride, groom, null);
    }

    /**
     * Two names matched star to star (naam milan): each name's first
     * syllable in the śatapada cakra, the varga koota of <i>Muhurta
     * Chintamani</i> VI.35, and the Ashta Koota and the ten considerations
     * read from the two name stars.
     *
     * <p>A name is read in Devanagari, or in IAST when
     * {@code rules.name.latin} is {@code "IAST"}; an English spelling is
     * refused rather than guessed. A name in Abhijit's row is refused unless
     * {@code rules.name.abhijit} places it. Each refusal is named,
     * {@code naam.groom.abhijit} and the like.
     *
     * @param bride the bride's name
     * @param groom the groom's name
     * @param rules the readings, such as {@code Map.of("name", Map.of("latin", "IAST"))},
     *     as {@link Json#write} writes them; may be null for the sources' own
     * @return the match
     */
    public NaamMilan naam(String bride, String groom, Map<String, ?> rules) {
        Map<String, Object> request = new LinkedHashMap<>();
        request.put("bride", bride);
        request.put("groom", groom);
        String written = Reads.recordJson(rules, "rules", "Map.of(\"name\", Map.of(\"latin\", \"IAST\"))");
        if (written != null) {
            request.put("rules", Json.read(written));
        }
        String json = Json.write(request);
        Naam d = Naam.decode(context.locked((lib, raw) -> Calls.naamMilan(lib, raw, json)));
        Naam.NaamNames n = d.naamNames();
        NameSyllable brideName = name(n, 0);
        NameSyllable groomName = name(n, 1);
        return new NaamMilan(
                brideName,
                groomName,
                new VargaKoota(brideName.varga(), groomName.varga(), VargaRelation.of(d.relation())),
                MatchReads.matchingsIn(d.matchings(), d.matchingKootas(), 1).get(0),
                MatchReads.poruthamsIn(d.poruthams(), d.poruthamRows(), 1).get(0));
    }

    private static NameSyllable name(Naam.NaamNames n, int at) {
        return new NameSyllable(n.cell(at), n.abhijit(at) == 1 ? null : Nakshatra.of(n.nakshatra(at)), n.quarter(at),
                NameVarga.of(n.varga(at)));
    }
}
