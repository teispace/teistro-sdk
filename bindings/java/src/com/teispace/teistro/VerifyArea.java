package com.teispace.teistro;

import java.util.List;
import java.util.Map;

import com.teispace.teistro.record.Provenance;

/**
 * {@code sky.verify()}: what a text claims of the time, checked against what
 * the almanac computes at an instant and a place ({@code 03-design/lexicon.md} §3).
 *
 * <pre>{@code
 * ClaimsAnswer checked = sky.verify().claims(ClaimsRequest.builder()
 *         .text("आज एकादशी हो").at(2461055.59375).latitudeDeg(27.7172).longitudeDeg(85.324)
 *         .utcOffsetSeconds(20700).build());
 * }</pre>
 */
public final class VerifyArea {
    private final Context context;

    /**
     * The verify area of a context.
     *
     * @param context the context every call goes through
     */
    VerifyArea(Context context) {
        this.context = context;
    }

    /**
     * Every claim a text makes of the time, each {@code HELD}, {@code CONTRADICTED}
     * (with the member computed) or {@code UNREAD} (with why), judged against
     * {@code almanac.sankalpa} at the instant.
     *
     * @param request the text, the instant and the place
     * @return the claims, sealed with the sankalpa's provenance
     */
    public ClaimsAnswer claims(com.teispace.teistro.request.ClaimsRequest request) {
        if (request == null) {
            throw Reads.invalid("request is a ClaimsRequest", "request");
        }
        String json = Json.write(request);
        Map<?, ?> envelope = Reads.object(Json.read(context.locked((lib, raw) -> Calls.verifyClaims(lib, raw, json))));
        Map<?, ?> value = Reads.object(envelope, "value");
        List<ClaimsAnswer.Claim> claims = Reads.array(value, "claims").stream().map(raw -> {
            Map<?, ?> c = Reads.object(raw);
            List<String> claimed = Reads.array(c, "claimed").stream().map(Reads::string).toList();
            return new ClaimsAnswer.Claim((int) Reads.number(c, "start"), (int) Reads.number(c, "end"),
                    Reads.string(c, "text"), Reads.optionalString(c, "kind"), Reads.optionalString(c, "graha"),
                    claimed, Reads.string(c, "verdict"), Reads.optionalString(c, "computed"),
                    Reads.optionalString(c, "reason"));
        }).toList();
        return new ClaimsAnswer(claims, Provenance.of(Reads.field(envelope, "provenance")));
    }
}
