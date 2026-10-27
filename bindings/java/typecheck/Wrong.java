// The usages that must not compile, each with the error it must raise.
//
// `cargo xtask check-java` compiles this file on its own against the
// module and fails unless every expectation below is reported, on the
// statement after it, and no other error is: the Java half of "a swapped
// latitude and longitude does not compile", and of "a request is typed".

import java.util.List;
import java.util.Map;

import com.teispace.teistro.Almanac;
import com.teispace.teistro.Altitude;
import com.teispace.teistro.CalendarDate;
import com.teispace.teistro.ChartOptions;
import com.teispace.teistro.Context;
import com.teispace.teistro.Latitude;
import com.teispace.teistro.Longitude;
import com.teispace.teistro.NaamMilan;
import com.teispace.teistro.Observer;
import com.teispace.teistro.RashifalRequest;

final class Wrong {
    private Wrong() {
    }

    static void usages(Context sky, CalendarDate day, Observer place) {
        // expect: Latitude cannot be converted to Longitude
        new Observer(new Latitude(27.7172), new Longitude(85.324), new Altitude(1400));
        // expect: double cannot be converted to Latitude
        new Observer(new Longitude(85.324), 27.7172, new Altitude(1400));
        // A reading takes its request type, not a map of one.
        // expect: conforms to KpRequest
        ChartOptions.builder().kp(Map.of("number", 108));
        // expect: String cannot be converted to int
        com.teispace.teistro.request.KpRequest.builder().number("108");
        // The areas take theirs the same way.
        // expect: conforms to NaamRules
        NaamMilan milan = sky.matching().naam("Sita", "Rama", Map.of("name", Map.of("latin", "IAST")));
        // expect: conforms to Snapshot
        RashifalRequest.of(day, place, 20_700).withSnapshot(Map.of("at", "CLOCK", "hour", 6, "minute", 0));
        // expect: conforms to MuhurtaRequest
        Almanac week = sky.almanac().of(day, day, place, 20_700, Map.of("rules", "RAMAN_MARRIAGE"), null,
                false, false, false);
        // expect: conforms to NumerologyRules
        sky.numerology().profile("Ram", java.time.LocalDate.of(1990, 4, 14), Map.of("masters", "NONE"));
        // expect: conforms to ResearchDesign
        sky.research().counts(List.of(), null, Map.of("groups", List.of(0, 1)), null);
        // expect: conforms to RuleRequest
        sky.research().counts(List.of(), Map.of("shipped", List.of("YOGAS")), null, null);
        // expect: conforms to ResearchGroupTest
        sky.research().compare(List.of(), null, null, Map.of("seed", 7, "permutations", 999), null);
        // A member of one kind does not stand in for another's.
        // expect: PakshiSubLengths cannot be converted to PakshiClock
        com.teispace.teistro.request.PakshiRules.builder().clock(com.teispace.teistro.request.PakshiSubLengths.AGASTYA);
        System.out.println(List.of(milan, week));
    }
}
