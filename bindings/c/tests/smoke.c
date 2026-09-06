/* The C binding's own test: a consumer that uses nothing but the generated
 * header and the built library. It proves what a Rust test cannot, that a
 * C compiler agrees with the header (struct layouts, enum values, the
 * calling convention) and that the library links and answers.
 *
 * Run it with `cargo xtask check-c`, which builds the library and compiles
 * this file against the header with warnings as errors. */

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "teistro.h"

static ts_context *ctx;
static int failures;

static void check(ts_status status, const char *what) {
    if (status == TS_STATUS_OK) return;
    ts_error error;
    memset(&error, 0, sizeof error);
    error.struct_size = sizeof error;
    ts_context_last_error(ctx, &error);
    printf("FAIL %s: %s (%d) %s\n", what, ts_status_message(status), (int)status,
           error.message ? error.message : "");
    failures++;
}

static void expect(int condition, const char *what) {
    if (condition) return;
    printf("FAIL %s\n", what);
    failures++;
}

/* One section of a result blob, found by its id in the table of contents. */
static const unsigned char *section(const ts_blob *blob, unsigned int wanted, unsigned int *count) {
    unsigned int sections, i, id, offset;
    memcpy(&sections, blob->data + 8, 4);
    for (i = 0; i < sections; i++) {
        const unsigned char *entry = blob->data + 32 + i * 16;
        memcpy(&id, entry, 4);
        if (id != wanted) continue;
        memcpy(&offset, entry + 4, 4);
        memcpy(count, entry + 12, 4);
        return blob->data + offset;
    }
    *count = 0;
    return NULL;
}

static void the_versions_agree(void) {
    printf("abi %u, sdk %s, catalogue %u, default profile %s\n", ts_abi_version(),
           ts_sdk_version(), ts_catalogue_version(), ts_default_profile());
    expect(ts_abi_version() == TS_ABI_VERSION, "the library's ABI is the header's");
    expect(strcmp(ts_status_message(TS_STATUS_PROVIDER), "provider failed") == 0,
           "a status has its phrase");
}

static void a_date_converts_into_bikram_sambat(void) {
    ts_calendar_date gregorian, bs;
    memset(&gregorian, 0, sizeof gregorian);
    gregorian.struct_size = sizeof gregorian;
    gregorian.calendar = TS_CALENDAR_GREGORIAN;
    gregorian.era = 0xFFFF;
    gregorian.year = 2015;
    gregorian.month = 4;
    gregorian.day = 14;
    memset(&bs, 0, sizeof bs);
    bs.struct_size = sizeof bs;
    check(ts_calendar_convert(ctx, &gregorian, TS_CALENDAR_BIKRAM_SAMBAT, &bs), "convert");
    printf("14 April 2015 is %d-%02u-%02u BS, era %u year %d, resolution %u\n", bs.year,
           bs.month, bs.day, bs.era, bs.era_year, bs.resolution);
    expect(bs.year == 2072 && bs.month == 1 && bs.day == 1, "1 Baisakh 2072");
    expect(bs.era == TS_ERA_VIKRAMA && bs.era_year == 2072, "the Vikrama era");
    expect(bs.resolution == TS_RESOLUTION_TABULAR, "inside the official table");
}

static void a_nepali_birth_time_resolves(void) {
    ts_civil_date_time civil;
    ts_zone_spec zone;
    ts_zone_resolution resolution;
    memset(&civil, 0, sizeof civil);
    civil.struct_size = sizeof civil;
    civil.date.struct_size = sizeof civil.date;
    civil.date.calendar = TS_CALENDAR_GREGORIAN;
    civil.date.era = 0xFFFF;
    civil.date.year = 1986;
    civil.date.month = 1;
    civil.date.day = 1;
    civil.time.struct_size = sizeof civil.time;
    civil.time.minute = 20;
    civil.time.has_time = 1;
    memset(&zone, 0, sizeof zone);
    zone.struct_size = sizeof zone;
    zone.kind = TS_ZONE_KIND_IANA;
    zone.zone = "Asia/Kathmandu";
    memset(&resolution, 0, sizeof resolution);
    resolution.struct_size = sizeof resolution;
    check(ts_time_resolve(ctx, &civil, &zone, &resolution), "resolve");
    printf("00:20 on 1 January 1986 in Kathmandu is JD %.6f UTC, offset %+d s (%s), tzdb %s\n",
           resolution.instant_jd_utc, resolution.offset_seconds,
           resolution.abbreviation ? resolution.abbreviation : "-", resolution.tzdb_version);
    expect(resolution.offset_seconds == 20700, "+05:45, the offset that began that midnight");
    expect(resolution.era == TS_ZONE_ERA_CURRENT, "the zone's current rules");
    expect(resolution.warnings == 0, "no warning");
}

static void a_message_renders_in_nepali(void) {
    ts_blob blob;
    unsigned int count = 0;
    const unsigned char *text;
    memset(&blob, 0, sizeof blob);
    check(ts_intl_render(ctx, "sdk.reason.grahaInBhava",
                         "{\"graha\": {\"$entity\": \"graha.JUPITER\"}, \"bhava\": 7}", &blob),
          "render");
    text = section(&blob, 2, &count);
    expect(text != NULL && count > 0, "the render blob carries its text");
    if (text) printf("sdk.reason.grahaInBhava in ne-Deva-NP: %.*s\n", (int)count, text);
    ts_blob_free(&blob);
    expect(blob.data == NULL, "freeing zeroes the descriptor");
}

static void positions_come_back_in_the_canonical_frame(void) {
    ts_frame frame;
    uint32_t bits = 0;
    double jds[2];
    uint16_t bodies[2];
    ts_position_request request;
    ts_blob blob;
    unsigned int cells = 0, fields = 0;
    const unsigned char *directory;
    const unsigned char *summary;
    memset(&frame, 0, sizeof frame);
    frame.struct_size = sizeof frame;
    check(ts_frame_canonical(&frame), "canonical frame");
    expect(frame.centre == TS_CENTRE_GEOCENTRIC && frame.coordinates == TS_COORDINATES_ECLIPTIC,
           "the canonical frame is geocentric ecliptic");
    check(ts_frame_pack(&frame, &bits), "pack");

    jds[0] = 2451545.0;
    jds[1] = 2451546.0;
    bodies[0] = TS_BODY_SUN;
    bodies[1] = TS_BODY_MOON;
    memset(&request, 0, sizeof request);
    request.struct_size = sizeof request;
    request.scale = TS_TIME_SCALE_UT1;
    request.frame_bits = bits;
    request.speeds = 1;
    request.jds = jds;
    request.jd_count = 2;
    request.bodies = bodies;
    request.body_count = 2;
    memset(&blob, 0, sizeof blob);
    check(ts_positions(ctx, &request, &blob), "positions");
    if (blob.data) {
        double lon = 0.0;
        unsigned int column = 0;
        summary = section(&blob, 1, &fields);
        directory = section(&blob, 4, &cells);
        expect(summary != NULL && cells == 4, "four cells over two instants and two bodies");
        /* The `cells` section's first column is `lon`: its offset is the
         * first word of the section's directory. */
        if (directory) {
            memcpy(&column, directory, 4);
            memcpy(&lon, directory + column, 8);
            printf("the Sun at J2000 is at %.4f degrees, %u cells\n", lon, cells);
            expect(lon >= 0.0 && lon < 360.0, "a longitude in range");
        }
        ts_blob_free(&blob);
    }
}

static void a_refusal_names_its_field_and_hints(void) {
    uint32_t id = 0;
    ts_str name;
    ts_error error;
    memset(&name, 0, sizeof name);
    memset(&error, 0, sizeof error);
    error.struct_size = sizeof error;
    expect(ts_key_parse(ctx, "graha.SUNN", &id) == TS_STATUS_UNSUPPORTED, "an unknown key");
    ts_context_last_error(ctx, &error);
    printf("refused: %s | detail %s | hint %s\n", error.message ? error.message : "-",
           error.detail ? error.detail : "-", error.hint ? error.hint : "-");
    expect(error.detail && strcmp(error.detail, "UNKNOWN_KEY") == 0, "the detail");
    expect(error.hint && strstr(error.hint, "SUN") != NULL, "the nearest key as a hint");
    check(ts_key_parse(ctx, "graha.SUN", &id), "key");
    check(ts_key_name(ctx, id, &name), "key name");
    expect(strcmp(name.data, "graha.SUN") == 0, "the key round trips");
    expect(id == ((uint32_t)TS_KIND_GRAHA << 16 | TS_GRAHA_SUN), "the packed id is kind and member");
}

int main(void) {
    ts_context_options options;
    ts_string error;
    memset(&options, 0, sizeof options);
    options.struct_size = sizeof options;
    options.flags = TS_CONTEXT_TEST_PROVIDER;
    options.profile = "nepali-default";
    options.locale = "ne-Deva-NP";
    memset(&error, 0, sizeof error);
    if (ts_context_new(&options, NULL, NULL, &ctx, &error) != TS_STATUS_OK) {
        printf("FAIL context: %s\n", error.data ? (const char *)error.data : "?");
        ts_string_free(&error);
        return 1;
    }
    the_versions_agree();
    a_date_converts_into_bikram_sambat();
    a_nepali_birth_time_resolves();
    a_message_renders_in_nepali();
    positions_come_back_in_the_canonical_frame();
    a_refusal_names_its_field_and_hints();
    ts_context_free(ctx);
    if (failures > 0) {
        printf("%d failure(s)\n", failures);
        return 1;
    }
    printf("the C binding's smoke test passed\n");
    return 0;
}
