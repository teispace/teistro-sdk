package com.teispace.teistro;

import com.teispace.teistro.request.AayadiSystem;
import com.teispace.teistro.request.Tara;
import java.util.List;
import java.util.Map;

/**
 * A building's measure read by one vastu text ({@code 03-design/aayadi.md}):
 * every reckoning it names, the vyaya's kind and the tara where it reads
 * them, and each clause it judges, never one verdict.
 *
 * @param system the system
 * @param quantities every reckoning the system names, in its text's order
 * @param vyaya <i>Samarangana</i>'s vyaya by its size beside the aaya, for that system alone; may be null
 * @param tara the tara from the owner's star to the house's, where read; may be null
 * @param clauses each clause the system judges, in its text's order
 * @param unjudged what the system asks that is not judged
 */
public record Aayadi(
        AayadiSystem system,
        List<AayadiReckoned> quantities,
        VyayaKind vyaya,
        Tara tara,
        List<AayadiClause> clauses,
        List<AayadiUnjudged> unjudged) {

    /**
     * The reckoning of a quantity, if the system names it.
     *
     * @param quantity the quantity
     * @return its reckoning, or null
     */
    public AayadiReckoned get(AayadiQuantity quantity) {
        return quantities.stream().filter(reckoned -> reckoned.quantity() == quantity).findFirst().orElse(null);
    }

    /**
     * Whether a clause holds, if the system judged it.
     *
     * @param clause the clause
     * @return whether it holds, or null when the system does not judge it
     */
    public Boolean holds(AayadiClauseKey clause) {
        return clauses.stream().filter(judged -> judged.clause() == clause).findFirst().map(AayadiClause::holds)
                .orElse(null);
    }

    static Aayadi read(Object raw) {
        Map<?, ?> o = Reads.object(raw);
        String system = Reads.string(o, "system");
        Object vyaya = o.get("vyaya");
        Object tara = o.get("tara");
        return new Aayadi(
                AayadiSystem.byKey(system).orElseThrow(
                        () -> Reads.internal("the library drew a system this build does not know: " + system)),
                Reads.each(o, "quantities", AayadiReckoned::read),
                vyaya == null ? null : Reads.member(VyayaKind.byKey(Reads.string(vyaya)), "VyayaKind", vyaya),
                tara == null ? null : Tara.byKey(Reads.string(tara)).orElseThrow(
                        () -> Reads.internal("the library drew a tara this build does not know: " + tara)),
                Reads.each(o, "clauses", AayadiClause::read),
                Reads.each(o, "unjudged", AayadiUnjudged::read));
    }
}
