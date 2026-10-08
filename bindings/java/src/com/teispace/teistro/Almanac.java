package com.teispace.teistro;

import java.util.AbstractList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.IntToLongFunction;
import java.util.function.Supplier;

import com.teispace.teistro.blob.Panchanga;
import com.teispace.teistro.record.Provenance;

/**
 * A batch of daily panchangas at one place, read one day at a time. A
 * {@link List} of its days, so it indexes and iterates; each day is a view
 * over the batch's columns, not a copy.
 *
 * <p>Every per-day list is concatenated across the batch, so a day's rows
 * are found by adding up every earlier day's count. That sum is done
 * <b>once</b>, when the batch is built, rather than per access: the
 * alternative is quadratic over a year of days, which is the shape an
 * almanac is actually asked for.
 */
public final class Almanac extends AbstractList<AlmanacDay> {
    /** The per-day lists, by the names {@link #range} takes. */
    private static final List<String> LISTS = List.of(
            "tithi", "nakshatra", "yoga", "karana", "panchaka", "moon_signs", "sun_signs", "kaalas", "choghadiya",
            "horas", "muhurtas", "moon_events", "muhurta_yogas");

    private final Panchanga decoded;
    private final Map<String, int[]> starts = new HashMap<>();
    private final Supplier<Provenance> provenance;
    private final Supplier<Optional<MuhurtaAnswer>> muhurta;
    private final Supplier<Optional<FestivalAnswer>> festivals;
    private final Supplier<Optional<LunarYears>> years;
    private final Supplier<Optional<Eclipses>> eclipses;
    private final Supplier<Optional<NepalSambatDates>> nepalSambat;

    /**
     * An almanac over a decoded panchanga blob.
     *
     * @param decoded the blob as its generated decoder read it
     */
    public Almanac(Panchanga decoded) {
        this.decoded = decoded;
        Panchanga.Counts counts = decoded.counts();
        for (String name : LISTS) {
            IntToLongFunction column = switch (name) {
                case "tithi" -> counts::tithi;
                case "nakshatra" -> counts::nakshatra;
                case "yoga" -> counts::yoga;
                case "karana" -> counts::karana;
                case "panchaka" -> counts::panchaka;
                case "moon_signs" -> counts::moonSigns;
                case "sun_signs" -> counts::sunSigns;
                case "kaalas" -> counts::kaalas;
                case "choghadiya" -> counts::choghadiya;
                case "horas" -> counts::horas;
                case "muhurtas" -> counts::muhurtas;
                case "moon_events" -> counts::moonEvents;
                case "muhurta_yogas" -> counts::muhurtaYogas;
                default -> throw new IllegalStateException(name);
            };
            starts.put(name, Reads.starts(counts.length(), column));
        }
        this.provenance = Lazy.of(() -> Provenance.of(Json.read(decoded.provenanceJson())));
        this.muhurta = Lazy.of(() -> present(decoded.muhurta()).map(AlmanacReads::muhurtaAnswer));
        this.festivals = Lazy.of(() -> present(decoded.festivals()).map(AlmanacReads::festivalsAnswer));
        this.years = Lazy.of(() -> present(decoded.years()).map(AlmanacReads::yearsAnswer));
        this.eclipses = Lazy.of(() -> present(decoded.eclipses()).map(AlmanacReads::eclipsesAnswer));
        this.nepalSambat = Lazy.of(() -> present(decoded.nepalSambat()).map(AlmanacReads::nepalSambatAnswer));
    }

    private static Optional<String> present(String text) {
        return text == null || text.isEmpty() ? Optional.empty() : Optional.of(text);
    }

    /**
     * The blob as its generated decoder read it.
     *
     * @return the decoded blob
     */
    public Panchanga decoded() {
        return decoded;
    }

    /**
     * How many days the batch holds.
     *
     * @return the count
     */
    @Override
    public int size() {
        return Math.toIntExact(decoded.dayCount());
    }

    /**
     * One day of the batch; the same as {@link #at}.
     *
     * @param index where it sits, from 0
     * @return the day
     * @throws IndexOutOfBoundsException for an index outside the batch
     */
    @Override
    public AlmanacDay get(int index) {
        return at(index);
    }

    /**
     * One day of the batch, by index.
     *
     * @param index where it sits, from 0
     * @return the day
     * @throws IndexOutOfBoundsException for an index outside the batch
     */
    public AlmanacDay at(int index) {
        if (index < 0 || index >= size()) {
            throw new IndexOutOfBoundsException("day " + index + " is outside a batch of " + size());
        }
        return new AlmanacDay(this, index);
    }

    /**
     * Where day {@code index}'s rows of a per-day list begin and end.
     *
     * @param listName the list: {@code tithi}, {@code nakshatra}, {@code yoga},
     *     {@code karana}, {@code panchaka}, {@code moon_signs}, {@code sun_signs},
     *     {@code kaalas}, {@code choghadiya}, {@code horas}, {@code muhurtas},
     *     {@code moon_events} or {@code muhurta_yogas}
     * @param index the day
     * @return the first row and the row after the last, in that order
     * @throws IllegalArgumentException for a list the almanac does not hold
     */
    public int[] range(String listName, int index) {
        int[] found = starts.get(listName);
        if (found == null) {
            throw new IllegalArgumentException("`" + listName + "` is not a per-day list; they are "
                    + String.join(", ", LISTS));
        }
        return new int[] {found[index], found[index + 1]};
    }

    /**
     * The place they were all founded at.
     *
     * @return the place
     */
    public Observer place() {
        return new Observer(new Longitude(decoded.longitudeDeg()), new Latitude(decoded.latitudeDeg()),
                new Altitude(decoded.altitudeM()));
    }

    /**
     * The civil calendar the days' dates are read in.
     *
     * @return the calendar
     */
    public Calendar calendar() {
        return Calendar.of(decoded.calendar());
    }

    /**
     * The solar model that reckoned the days, as it describes itself.
     *
     * @return the model
     */
    public String model() {
        return decoded.model();
    }

    /**
     * What computed these, and under what; its content hash is the whole batch's.
     *
     * @return the provenance
     */
    public Provenance provenance() {
        return provenance.get();
    }

    /**
     * The muhurta search {@code muhurta} asked for over these days, or empty
     * when it asked for none ({@code 03-design/muhurta-at-the-boundary.md}):
     * the windows judged clause by clause, the days the season closed, and
     * what computed it. Parsed once.
     *
     * @return the answer, if a search was asked
     */
    public Optional<MuhurtaAnswer> muhurta() {
        return muhurta.get();
    }

    /**
     * The days the rules {@code festivals} asked for fall on over these days,
     * or empty when it asked for none ({@code 03-design/festival-rules.md}
     * §7): each observance with the case between its tithi's two days and
     * the guard that decided. Parsed once.
     *
     * @return the answer, if festivals were asked
     */
    public Optional<FestivalAnswer> festivals() {
        return festivals.get();
    }

    /**
     * The lunar years these days fall in, or empty when {@code years} was not
     * asked ({@code 03-design/calendar-indian-lunisolar.md} §10): each with
     * the samvatsara it carries, its bounds from one Chaitra Shukla
     * Pratipada's sunrise to the next, the Jovian years that ran in it and
     * the one it expunged. Parsed once.
     *
     * @return the years, if asked
     */
    public Optional<LunarYears> years() {
        return years.get();
    }

    /**
     * The eclipses whose greatest moment falls in these days, or empty when
     * {@code eclipses} was not asked ({@code 03-design/eclipses.md}): each
     * lunar and solar eclipse with its contacts and how the place sees it.
     * Parsed once.
     *
     * @return the eclipses, if asked
     */
    public Optional<Eclipses> eclipses() {
        return eclipses.get();
    }

    /**
     * Each day's Nepal Sambat date, or empty when {@code nepalSambat} was not
     * asked ({@code 03-design/calendar-indian-lunisolar.md} §11): one a day
     * in the days' order, with its year, its month counted from Kachhala,
     * the month's kind (adhika is Anala) and its half. Parsed once.
     *
     * @return the dates, if asked
     */
    public Optional<NepalSambatDates> nepalSambat() {
        return nepalSambat.get();
    }

    /**
     * The provenance envelope as the canonical JSON the library stamped: the
     * bytes to store beside the result, byte-identical in every binding.
     *
     * @return the JSON
     */
    public String provenanceJson() {
        return decoded.provenanceJson();
    }
}
