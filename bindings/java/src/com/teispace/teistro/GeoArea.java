package com.teispace.teistro;

import com.teispace.teistro.request.GeoSource;
import com.teispace.teistro.request.PlaceRequest;
import com.teispace.teistro.request.ResolvedPlace;

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
}
