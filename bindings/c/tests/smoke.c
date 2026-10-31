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

/* A chart founded from the request struct, read back from its blob: the
 * widest struct a caller lays out, so a C compiler and the library must
 * agree on every offset in it. */
static void a_chart_is_founded_from_its_struct(void) {
    ts_chart_request request;
    ts_blob blob;
    double instant = 2451545.0;
    memset(&request, 0, sizeof request);
    request.struct_size = sizeof request;
    request.kind = TS_CHART_KIND_NATAL;
    request.instants = &instant;
    request.instant_count = 1;
    request.latitude_deg = 27.7172;
    request.longitude_deg = 85.324;
    request.altitude_m = 1337.0;
    request.utc_offset_seconds = 20700;
    memset(&blob, 0, sizeof blob);
    check(ts_chart_found(ctx, &request, &blob), "chart");
    expect(blob.data != NULL && blob.len > 32, "a chart's blob");
    ts_blob_free(&blob);
}

/* A week of days from the panchanga request struct. */
static void a_week_of_days_comes_back(void) {
    ts_panchanga_request request;
    ts_blob blob;
    memset(&request, 0, sizeof request);
    request.struct_size = sizeof request;
    request.calendar = TS_CALENDAR_GREGORIAN;
    request.from_year = 2026;
    request.from_month = 1;
    request.from_day = 1;
    request.to_year = 2026;
    request.to_month = 1;
    request.to_day = 7;
    request.latitude_deg = 27.7172;
    request.longitude_deg = 85.324;
    request.altitude_m = 1337.0;
    request.utc_offset_seconds = 20700;
    memset(&blob, 0, sizeof blob);
    check(ts_panchanga_days(ctx, &request, &blob), "days");
    expect(blob.data != NULL && blob.len > 32, "a week's blob");
    ts_blob_free(&blob);
}

/* An almanac rite asked as JSON, answered as JSON. */
static void a_sankalpa_is_answered_as_json(void) {
    ts_string answer;
    memset(&answer, 0, sizeof answer);
    check(ts_almanac_sankalpa(ctx, "{\"at\":2461055.59375,\"latitudeDeg\":27.7172,\"longitudeDeg\":85.324,\"altitudeM\":1400,\"utcOffsetSeconds\":20700,\"ayana\":\"TROPICAL\"}", &answer), "sankalpa");
    expect(answer.data != NULL && strstr((const char *)answer.data, "\"tithi\"") != NULL,
           "the sankalpa names its tithi");
    ts_string_free(&answer);
}

/* The published pack the parity runners verify, signed by a test key made
 * from a fixed seed: verified against that key, and refused for a version
 * newer than it carries. */
static const char signed_pack[] =
"{\"format\":\"teistro.published/1\",\"publisher\":\"a patro printed in Nepal\",\"id\":\""
    "np-2083\",\"version\":3,\"entries\":[{\"action\":\"SET\",\"rule\":\"GHATASTHAPANA\",\"da"
    "y\":{\"calendar\":\"GREGORIAN\",\"year\":2026,\"month\":10,\"day\":11},\"source\":\"p. 4"
    "1\"}]}";
static const uint8_t signed_signature[64] = {
    0xca, 0xe8, 0x7f, 0x1a, 0x99, 0xd4, 0x86, 0x9d, 0x5b, 0xa4, 0x10, 0x86,
    0xcd, 0x6c, 0xeb, 0xa8, 0xcd, 0xb0, 0x35, 0xfd, 0x11, 0x22, 0x5f, 0x53,
    0xe0, 0x98, 0x82, 0xe9, 0x84, 0x23, 0xe2, 0x13, 0x32, 0xfb, 0x40, 0xc9,
    0xad, 0x56, 0x43, 0x80, 0xfb, 0x9a, 0xbc, 0xdf, 0x4c, 0x21, 0xcb, 0x27,
    0x26, 0xf8, 0x0f, 0xb6, 0x92, 0x51, 0x1b, 0x95, 0x93, 0xf0, 0xf9, 0xe6,
    0x0f, 0xbe, 0x32, 0x0e,
};
static const uint8_t signed_key[32] = {
    0xea, 0x4a, 0x6c, 0x63, 0xe2, 0x9c, 0x52, 0x0a, 0xbe, 0xf5, 0x50, 0x7b,
    0x13, 0x2e, 0xc5, 0xf9, 0x95, 0x47, 0x76, 0xae, 0xbe, 0xbe, 0x7b, 0x92,
    0x42, 0x1e, 0xea, 0x69, 0x14, 0x46, 0xd2, 0x2c,
};

static void a_signed_pack_is_verified(void) {
    ts_string answer;
    size_t pack_len = sizeof signed_pack - 1;
    memset(&answer, 0, sizeof answer);
    check(ts_published_verify(ctx, (const uint8_t *)signed_pack, pack_len, signed_signature,
                              sizeof signed_signature, signed_key, sizeof signed_key, 3, &answer),
          "verify");
    expect(answer.data != NULL && strstr((const char *)answer.data, "np-2083") != NULL,
           "the verified set names its pack");
    ts_string_free(&answer);
    expect(ts_published_verify(ctx, (const uint8_t *)signed_pack, pack_len, signed_signature,
                               sizeof signed_signature, signed_key, sizeof signed_key, 4,
                               &answer) != TS_STATUS_OK,
           "an older version than asked is refused");
}

/* A context that cannot be built has no context to keep its refusal, so
 * the call writes the whole record, owning its strings until freed. */
static void a_context_that_cannot_be_built_says_why(void) {
    ts_context_options options;
    ts_context *none = NULL;
    ts_error error;
    memset(&options, 0, sizeof options);
    options.struct_size = sizeof options;
    options.profile = "vedic-classic";
    memset(&error, 0, sizeof error);
    error.struct_size = sizeof error;
    expect(ts_context_new(&options, NULL, NULL, &none, &error) == TS_STATUS_UNSUPPORTED,
           "an unknown profile is unsupported");
    expect(none == NULL, "no handle on failure");
    expect(error.flags & TS_ERROR_OWNED, "the record owns its strings");
    expect(error.field && strcmp(error.field, "profile") == 0, "the field");
    expect(error.hint && strstr(error.hint, "parashari-classical") != NULL,
           "the shipped profiles as a hint");
    ts_error_free(&error);
    expect(error.message == NULL && error.flags == 0, "freed and zeroed");
    ts_error_free(&error); /* a second free does nothing */
}

int main(void) {
    ts_context_options options;
    ts_error error;
    memset(&options, 0, sizeof options);
    options.struct_size = sizeof options;
    options.flags = TS_CONTEXT_TEST_PROVIDER;
    options.profile = "nepali-drik";
    options.locale = "ne-Deva-NP";
    memset(&error, 0, sizeof error);
    error.struct_size = sizeof error;
    if (ts_context_new(&options, NULL, NULL, &ctx, &error) != TS_STATUS_OK) {
        printf("FAIL context: %s\n", error.message ? error.message : "?");
        ts_error_free(&error);
        return 1;
    }
    a_context_that_cannot_be_built_says_why();
    the_versions_agree();
    a_date_converts_into_bikram_sambat();
    a_nepali_birth_time_resolves();
    a_message_renders_in_nepali();
    positions_come_back_in_the_canonical_frame();
    a_chart_is_founded_from_its_struct();
    a_week_of_days_comes_back();
    a_sankalpa_is_answered_as_json();
    a_signed_pack_is_verified();
    a_refusal_names_its_field_and_hints();
    ts_context_free(ctx);
    if (failures > 0) {
        printf("%d failure(s)\n", failures);
        return 1;
    }
    printf("the C binding's smoke test passed\n");
    return 0;
}
