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

    /**
     * A record's answer and its receipt: what was asked, under which settings,
     * ephemeris and versions, and the digest of what was answered
     * ({@code 03-design/receipts-and-datasets.md} §1). The receipt is kept as
     * its JSON text, to hand back whole to {@link #receipt(String)}.
     *
     * @param request the record's name and its request
     * @return the record's own answer and the receipt
     */
    public Issued issue(com.teispace.teistro.request.IssueRequest request) {
        if (request == null) {
            throw Reads.invalid("request is an IssueRequest", "request");
        }
        String json = Json.write(request);
        Map<?, ?> issued = Reads.object(Json.read(context.locked((lib, raw) -> Calls.verifyIssue(lib, raw, json))));
        return new Issued(Reads.field(issued, "answer"), Json.write(Reads.field(issued, "receipt")));
    }

    /**
     * A receipt checked: answered again under its own settings and compared,
     * {@code HELD}, {@code DIFFERS}, {@code NOT_RERUN} (with why) or
     * {@code BROKEN} (a field edited after issue), with every version that
     * moved between the two sides.
     *
     * @param receipt the receipt's JSON text, as {@link Issued#receipt()} keeps it
     * @return the check
     */
    public ReceiptCheck receipt(String receipt) {
        if (receipt == null) {
            throw Reads.invalid("receipt is a receipt's JSON text", "receipt");
        }
        Map<?, ?> raw = Reads.object(Json.read(context.locked((lib, ctx) -> Calls.verifyReceipt(lib, ctx, receipt))));
        List<ReceiptCheck.Move> moved = Reads.array(raw, "moved").stream().map(m -> {
            Map<?, ?> move = Reads.object(m);
            return new ReceiptCheck.Move(Reads.string(move, "field"), Reads.string(move, "issued"), Reads.string(move, "now"));
        }).toList();
        List<String> broken = Reads.array(raw, "broken").stream().map(Reads::string).toList();
        Object reason = Reads.field(raw, "reason");
        ReceiptCheck.Reason why = null;
        if (reason != null) {
            Map<?, ?> r = Reads.object(reason);
            why = new ReceiptCheck.Reason(Reads.string(r, "cause"), Reads.string(r, "detail"));
        }
        return new ReceiptCheck(Reads.string(raw, "verdict"), Reads.string(raw, "record"), moved, broken,
                side(Reads.object(raw, "issued")), side(Reads.object(raw, "now")), why);
    }

    private static ReceiptCheck.Side side(Map<?, ?> raw) {
        Map<?, ?> version = Reads.object(raw, "sdkVersion");
        Object provider = Reads.field(raw, "provider");
        ReceiptCheck.Provider ephemeris = null;
        if (provider != null) {
            Map<?, ?> p = Reads.object(provider);
            ephemeris = new ReceiptCheck.Provider(Reads.string(p, "name"), Reads.string(p, "version"),
                    Reads.string(p, "dataVersion"));
        }
        return new ReceiptCheck.Side(Reads.integer(version, "major") + "." + Reads.integer(version, "minor") + "."
                + Reads.integer(version, "patch"), Reads.integer(raw, "calculationVersion"),
                Reads.integer(raw, "catalogueVersion"), ephemeris, Reads.string(raw, "tzdb"));
    }
}
