package com.teispace.teistro;

import com.teispace.teistro.request.GeoSource;
import com.teispace.teistro.request.PlaceListing;
import com.teispace.teistro.request.PlaceLookup;
import com.teispace.teistro.request.PlaceNamesRequest;
import com.teispace.teistro.request.PlaceRequest;
import com.teispace.teistro.request.PlaceReverse;
import com.teispace.teistro.request.PlaceSearch;
import com.teispace.teistro.request.ResolvedPlace;
import java.util.List;
import java.util.Map;

/**
 * {@code sky.geo()}: a place's IANA zone, ground height and ISO 3166 admin
 * codes, from the context's geo provider ({@code 03-design/geo-port.md}).
 * The SDK never consults it on its own: a context has one only once
 * {@link #load} gives it a table.
 */
public final class GeoArea {
    private final Context context;

    GeoArea(Context context) {
        this.context = context;
    }

    /**
     * Installs a geo table as this context's provider, in place of any it
     * had: {@code {source, areas}}, each area a box
     * {@code {south, west, north, east}} with any of {@code zone},
     * {@code height} and {@code admin}, written coarse to fine.
     *
     * @param tableJson the table's JSON
     * @return the table's source: its name, version and data version
     * @throws TeistroException with {@link Status#INVALID_ARG} naming the part
     *     under {@code geo}; the context keeps the provider it had
     */
    public GeoSource load(String tableJson) {
        return GeoSource.of(Json.read(context.locked((lib, raw) -> Calls.geoLoad(lib, raw, tableJson))));
    }

    /**
     * A place resolved: the IANA zone, the ground height with its source,
     * resolution and datum, the admin units coarse to fine with their ISO
     * 3166 codes, and the provider's name and data version. A chart
     * request's {@code place} takes it as it is, so a receipt replays
     * without the provider.
     *
     * @param request the point, in degrees
     * @return the place
     * @throws TeistroException with {@link Status#CAPABILITY} on {@code geo}
     *     for a context with no provider, {@link Status#INVALID_ARG} on a
     *     point out of range
     */
    public ResolvedPlace resolve(PlaceRequest request) {
        String json = Json.write(request);
        return ResolvedPlace.of(Json.read(context.locked((lib, raw) -> Calls.geoResolve(lib, raw, json))));
    }

    /**
     * The places matching a name in any script the provider's data holds,
     * best first, as the provider ranks them: {@code "Name, Parent"} puts
     * the named parent's places first, and a {@code focus} prefers places
     * near it. Each is {@code {place, score}}, the place
     * {@code {id, name, nameLang, label, kind, level, code, latitudeDeg,
     * longitudeDeg, population, parents}}.
     *
     * @param request the query, with its focus, limit and language
     * @return the places found, best first
     * @throws TeistroException with {@link Status#CAPABILITY} on {@code geo}
     *     for a context with no provider, {@link Status#INVALID_ARG} on a
     *     malformed request
     */
    public List<?> search(PlaceSearch request) {
        String json = Json.write(request);
        return JsonRead.list(Json.read(context.locked((lib, raw) -> Calls.geoSearch(lib, raw, json))));
    }

    /**
     * The named places nearest a point, nearest first, and the admin units
     * holding it, coarse to fine: {@code {near, admin}}. A level the data
     * holds no outline for is absent, never a refusal.
     *
     * @param request the point, with its limit and language
     * @return the places near and the admin units
     * @throws TeistroException with {@link Status#CAPABILITY} on {@code geo}
     *     for a context with no provider
     */
    public Map<String, Object> reverse(PlaceReverse request) {
        String json = Json.write(request);
        return JsonRead.object(Json.read(context.locked((lib, raw) -> Calls.geoReverse(lib, raw, json))));
    }

    /**
     * A place's names, each with its language, script, type and origin,
     * and the one chosen for each language asked: {@code {id, best, all}}.
     *
     * @param request the place's id and the languages asked
     * @return the names
     * @throws TeistroException with {@link Status#CAPABILITY} on {@code geo}
     *     for a context with no provider
     */
    public Map<String, Object> names(PlaceNamesRequest request) {
        String json = Json.write(request);
        return JsonRead.object(Json.read(context.locked((lib, raw) -> Calls.geoNames(lib, raw, json))));
    }

    /**
     * A place by its id, named in {@code lang}, with its id line, zone,
     * population and landmark class: {@code {asked, place, moved}},
     * {@code moved} when the id asked was retired and {@code place} is the
     * one that replaced it, whose id a caller stores instead; {@code place}
     * null for an id the provider does not know.
     *
     * @param request the place's id and the language to name it in
     * @return the place looked up
     * @throws TeistroException with {@link Status#CAPABILITY} on {@code geo}
     *     for a context with no provider
     */
    public Map<String, Object> place(PlaceLookup request) {
        String json = Json.write(request);
        return JsonRead.object(Json.read(context.locked((lib, raw) -> Calls.geoPlace(lib, raw, json))));
    }

    /**
     * The places of a unit (its id or ISO 3166 code), by admin level and
     * kind, ranked by population, each with {@code {people, kind}},
     * {@code kind} {@code COUNTED} or {@code MODELLED}.
     *
     * @param request the unit, level, kinds, depth and limit
     * @return the places, most people first
     * @throws TeistroException with {@link Status#CAPABILITY} on {@code geo}
     *     for a context with no provider
     */
    public List<?> list(PlaceListing request) {
        String json = Json.write(request);
        return JsonRead.list(Json.read(context.locked((lib, raw) -> Calls.geoList(lib, raw, json))));
    }
}
