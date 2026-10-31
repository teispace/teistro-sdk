/*
 * The N-day digest's reader (docs/03-design/digest-format.md): no
 * dependency, no allocation, no assumption about the bytes' alignment or
 * the machine's byte order. Every field is read a byte at a time.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include "teistro_digest.h"

#define MAGIC 0x42525354u /* "TSRB", little-endian */
#define LAYOUT 1u
#define SCHEMA 6u
#define HEADER_LEN 32u
#define ENTRY_LEN 16u
#define SLOT 8u

/* Section ids, the schema's. */
enum {
    SECTION_SUMMARY = 1,
    SECTION_DAYS = 2,
    SECTION_SPANS = 3,
    SECTION_OBSERVANCES = 4,
    SECTION_NAMES = 5,
    SECTION_TEXT = 6,
    SECTION_LOCALE = 7,
    SECTION_PROVENANCE = 8,
    SECTION_SEAL = 9
};

/* Field indices of the summary, the schema's. */
enum {
    SUMMARY_MAJOR,
    SUMMARY_MINOR,
    SUMMARY_DAYS,
    SUMMARY_BASE,
    SUMMARY_OFFSET,
    SUMMARY_CALENDAR,
    SUMMARY_LATITUDE,
    SUMMARY_LONGITUDE,
    SUMMARY_ALTITUDE,
    SUMMARY_FIELDS
};

/* Columns, in the schema's order, and each one's width. */
enum {
    DAY_YEAR, DAY_MONTH, DAY_DAY, DAY_VARA, DAY_STATE, DAY_FIXED, DAY_SUNRISE,
    DAY_SUNSET, DAY_MOONRISE, DAY_MOONSET, DAY_PHASE, DAY_LUNAR_MONTH,
    DAY_SPAN_FIRST, DAY_SPAN_COUNT, DAY_OBSERVANCE_FIRST, DAY_OBSERVANCE_COUNT,
    DAY_COLUMNS
};
static const uint8_t DAY_WIDTHS[DAY_COLUMNS] = {4, 1, 1, 1, 1, 4, 4, 4, 4, 4, 2, 2, 4, 2, 4, 2};

enum { SPAN_KIND, SPAN_NAME, SPAN_FROM, SPAN_TO, SPAN_COLUMNS };
static const uint8_t SPAN_WIDTHS[SPAN_COLUMNS] = {1, 2, 4, 4};

enum { OBSERVANCE_NAME, OBSERVANCE_FLAGS, OBSERVANCE_COLUMNS };
static const uint8_t OBSERVANCE_WIDTHS[OBSERVANCE_COLUMNS] = {2, 1};

enum { NAME_KEY_AT, NAME_KEY_LENGTH, NAME_TEXT_AT, NAME_TEXT_LENGTH, NAME_COLUMNS };
static const uint8_t NAME_WIDTHS[NAME_COLUMNS] = {4, 2, 4, 2};

static uint16_t read_u16(const uint8_t *at) {
    return (uint16_t)((uint16_t)at[0] | (uint16_t)((uint16_t)at[1] << 8));
}

static uint32_t read_u32(const uint8_t *at) {
    return (uint32_t)at[0] | ((uint32_t)at[1] << 8) | ((uint32_t)at[2] << 16) | ((uint32_t)at[3] << 24);
}

static uint64_t read_u64(const uint8_t *at) {
    return (uint64_t)read_u32(at) | ((uint64_t)read_u32(at + 4) << 32);
}

/* Two's complement by arithmetic, so no conversion is implementation-defined. */
static int32_t read_i32(const uint8_t *at) {
    uint32_t bits = read_u32(at);
    return bits <= 0x7FFFFFFFu ? (int32_t)bits : -(int32_t)(0xFFFFFFFFu - bits) - 1;
}

static int64_t read_i64(const uint8_t *at) {
    uint64_t bits = read_u64(at);
    return bits <= 0x7FFFFFFFFFFFFFFFull ? (int64_t)bits
                                         : -(int64_t)(0xFFFFFFFFFFFFFFFFull - bits) - 1;
}

static double read_f64(const uint8_t *at) {
    union {
        uint64_t bits;
        double value;
    } both;
    both.bits = read_u64(at);
    return both.value;
}

/* The start of a column after checking it holds every row. */
static const uint8_t *column(const ts_digest_section *section, uint32_t index, uint32_t width) {
    uint32_t offset = read_u32(section->start + (size_t)index * 4u);
    uint64_t end = (uint64_t)offset + (uint64_t)section->count * width;
    if (offset % width != 0u || end > section->length) {
        return NULL;
    }
    return section->start + offset;
}

/* Whether a columns section has a directory and every column whole. */
static int columns_fit(const ts_digest_section *section, const uint8_t *widths, uint32_t columns) {
    uint32_t i;
    if ((uint64_t)columns * 4u > section->length) {
        return 0;
    }
    for (i = 0; i < columns; i++) {
        if (column(section, i, widths[i]) == NULL) {
            return 0;
        }
    }
    return 1;
}

/* SHA-256 (FIPS 180-4). */
static const uint32_t K[64] = {
    0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u, 0x923f82a4u,
    0xab1c5ed5u, 0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u, 0x72be5d74u, 0x80deb1feu,
    0x9bdc06a7u, 0xc19bf174u, 0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu,
    0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau, 0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u,
    0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u, 0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu,
    0x53380d13u, 0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u, 0xa2bfe8a1u, 0xa81a664bu,
    0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u, 0x19a4c116u,
    0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
    0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u, 0x90befffau, 0xa4506cebu, 0xbef9a3f7u,
    0xc67178f2u};

static uint32_t rotate(uint32_t value, unsigned bits) {
    return (value >> bits) | (value << (32u - bits));
}

static void sha256_block(uint32_t state[8], const uint8_t *block) {
    uint32_t w[64];
    uint32_t v[8];
    unsigned i;
    for (i = 0; i < 16; i++) {
        const uint8_t *at = block + i * 4u;
        w[i] = ((uint32_t)at[0] << 24) | ((uint32_t)at[1] << 16) | ((uint32_t)at[2] << 8) | (uint32_t)at[3];
    }
    for (i = 16; i < 64; i++) {
        uint32_t s0 = rotate(w[i - 15], 7) ^ rotate(w[i - 15], 18) ^ (w[i - 15] >> 3);
        uint32_t s1 = rotate(w[i - 2], 17) ^ rotate(w[i - 2], 19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16] + s0 + w[i - 7] + s1;
    }
    for (i = 0; i < 8; i++) {
        v[i] = state[i];
    }
    for (i = 0; i < 64; i++) {
        uint32_t s1 = rotate(v[4], 6) ^ rotate(v[4], 11) ^ rotate(v[4], 25);
        uint32_t choice = (v[4] & v[5]) ^ (~v[4] & v[6]);
        uint32_t t1 = v[7] + s1 + choice + K[i] + w[i];
        uint32_t s0 = rotate(v[0], 2) ^ rotate(v[0], 13) ^ rotate(v[0], 22);
        uint32_t majority = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
        uint32_t t2 = s0 + majority;
        v[7] = v[6];
        v[6] = v[5];
        v[5] = v[4];
        v[4] = v[3] + t1;
        v[3] = v[2];
        v[2] = v[1];
        v[1] = v[0];
        v[0] = t1 + t2;
    }
    for (i = 0; i < 8; i++) {
        state[i] += v[i];
    }
}

void ts_digest_sha256(const void *bytes, size_t length, uint8_t out[TS_DIGEST_HASH_LEN]) {
    uint32_t state[8] = {0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
                         0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u};
    const uint8_t *data = (const uint8_t *)bytes;
    uint64_t bits = (uint64_t)length * 8u;
    uint8_t tail[128];
    size_t remaining = length;
    size_t used;
    size_t total;
    unsigned i;
    while (remaining >= 64u) {
        sha256_block(state, data);
        data += 64;
        remaining -= 64u;
    }
    for (used = 0; used < remaining; used++) {
        tail[used] = data[used];
    }
    tail[used++] = 0x80u;
    total = used <= 56u ? 64u : 128u;
    while (used < total - 8u) {
        tail[used++] = 0u;
    }
    for (i = 0; i < 8; i++) {
        tail[total - 1u - i] = (uint8_t)(bits >> (i * 8u));
    }
    sha256_block(state, tail);
    if (total == 128u) {
        sha256_block(state, tail + 64);
    }
    for (i = 0; i < 8; i++) {
        out[i * 4u] = (uint8_t)(state[i] >> 24);
        out[i * 4u + 1u] = (uint8_t)(state[i] >> 16);
        out[i * 4u + 2u] = (uint8_t)(state[i] >> 8);
        out[i * 4u + 3u] = (uint8_t)state[i];
    }
}

/* Finds a section by id in a checked table. */
static int section_of(const uint8_t *bytes, uint32_t entries, uint32_t id, ts_digest_section *out) {
    uint32_t i;
    for (i = 0; i < entries; i++) {
        const uint8_t *entry = bytes + HEADER_LEN + (size_t)i * ENTRY_LEN;
        if (read_u32(entry) == id) {
            out->start = bytes + read_u32(entry + 4);
            out->length = read_u32(entry + 8);
            out->count = read_u32(entry + 12);
            return 1;
        }
    }
    return 0;
}

void ts_digest_seal(const void *bytes, size_t length, char out[TS_DIGEST_SEAL_LEN]) {
    static const char DIGITS[] = "0123456789abcdef";
    uint8_t hash[TS_DIGEST_HASH_LEN];
    uint32_t i;
    ts_digest_sha256(bytes, length, hash);
    for (i = 0; i < TS_DIGEST_HASH_LEN; i++) {
        out[2u * i] = DIGITS[hash[i] >> 4];
        out[2u * i + 1u] = DIGITS[hash[i] & 15u];
    }
}

ts_digest_status ts_digest_open(ts_digest *digest, const void *bytes, size_t length) {
    const uint8_t *data = (const uint8_t *)bytes;
    uint32_t entries;
    uint32_t i;
    uint32_t major;
    ts_digest_section summary;
    ts_digest_section seal;
    char computed[TS_DIGEST_SEAL_LEN];
    uint8_t differs = 0;
    if (digest == NULL || data == NULL) {
        return TS_DIGEST_E_ARG;
    }
    if (length < 4u) {
        return TS_DIGEST_E_TRUNCATED;
    }
    if (read_u32(data) != MAGIC) {
        return TS_DIGEST_E_MAGIC;
    }
    if (length < HEADER_LEN) {
        return TS_DIGEST_E_TRUNCATED;
    }
    if (read_u32(data + 4) != LAYOUT) {
        return TS_DIGEST_E_LAYOUT;
    }
    if (read_u32(data + 16) != SCHEMA) {
        return TS_DIGEST_E_SCHEMA;
    }
    if ((uint64_t)read_u32(data + 12) > length) {
        return TS_DIGEST_E_TRUNCATED;
    }
    if ((uint64_t)read_u32(data + 12) != length) {
        return TS_DIGEST_E_MALFORMED;
    }
    entries = read_u32(data + 8);
    if ((uint64_t)entries * ENTRY_LEN > length - HEADER_LEN) {
        return TS_DIGEST_E_MALFORMED;
    }
    for (i = 0; i < entries; i++) {
        const uint8_t *entry = data + HEADER_LEN + (size_t)i * ENTRY_LEN;
        if ((uint64_t)read_u32(entry + 4) + read_u32(entry + 8) > length) {
            return TS_DIGEST_E_MALFORMED;
        }
    }
    /* The version is read before the seal, so a later format is refused as
       later rather than as damaged, whatever it seals with. */
    if (!section_of(data, entries, SECTION_SUMMARY, &summary) ||
        summary.length < SUMMARY_FIELDS * SLOT) {
        return TS_DIGEST_E_MALFORMED;
    }
    major = read_u32(summary.start + SUMMARY_MAJOR * SLOT);
    if (major != TS_DIGEST_FORMAT_MAJOR) {
        return TS_DIGEST_E_VERSION;
    }
    if (!section_of(data, entries, SECTION_SEAL, &seal) || seal.length != TS_DIGEST_SEAL_LEN ||
        (size_t)(seal.start - data) + TS_DIGEST_SEAL_LEN != length) {
        return TS_DIGEST_E_MALFORMED;
    }
    ts_digest_seal(data, (size_t)(seal.start - data), computed);
    for (i = 0; i < TS_DIGEST_SEAL_LEN; i++) {
        differs |= (uint8_t)((uint8_t)computed[i] ^ seal.start[i]);
    }
    if (differs != 0u) {
        return TS_DIGEST_E_SEAL;
    }
    digest->bytes = data;
    digest->length = length;
    digest->format_minor = read_u32(summary.start + SUMMARY_MINOR * SLOT);
    digest->day_count = read_u32(summary.start + SUMMARY_DAYS * SLOT);
    digest->base_unix_seconds = read_i64(summary.start + SUMMARY_BASE * SLOT);
    digest->utc_offset_seconds = read_i32(summary.start + SUMMARY_OFFSET * SLOT);
    digest->calendar = read_u32(summary.start + SUMMARY_CALENDAR * SLOT);
    digest->latitude_deg = read_f64(summary.start + SUMMARY_LATITUDE * SLOT);
    digest->longitude_deg = read_f64(summary.start + SUMMARY_LONGITUDE * SLOT);
    digest->altitude_m = read_f64(summary.start + SUMMARY_ALTITUDE * SLOT);
    if (!section_of(data, entries, SECTION_DAYS, &digest->days) ||
        !section_of(data, entries, SECTION_SPANS, &digest->spans) ||
        !section_of(data, entries, SECTION_OBSERVANCES, &digest->observances) ||
        !section_of(data, entries, SECTION_NAMES, &digest->names) ||
        !section_of(data, entries, SECTION_TEXT, &digest->text) ||
        !section_of(data, entries, SECTION_LOCALE, &digest->locale) ||
        !section_of(data, entries, SECTION_PROVENANCE, &digest->provenance)) {
        return TS_DIGEST_E_MALFORMED;
    }
    if (digest->days.count != digest->day_count ||
        !columns_fit(&digest->days, DAY_WIDTHS, DAY_COLUMNS) ||
        !columns_fit(&digest->spans, SPAN_WIDTHS, SPAN_COLUMNS) ||
        !columns_fit(&digest->observances, OBSERVANCE_WIDTHS, OBSERVANCE_COLUMNS) ||
        !columns_fit(&digest->names, NAME_WIDTHS, NAME_COLUMNS)) {
        return TS_DIGEST_E_MALFORMED;
    }
    return TS_DIGEST_OK;
}

ts_digest_status ts_digest_day_at(const ts_digest *digest, uint32_t index, ts_digest_day *day) {
    const ts_digest_section *days;
    size_t at = index;
#define DAY_COLUMN(which) (column(days, (which), DAY_WIDTHS[(which)]) + at * DAY_WIDTHS[(which)])
    if (digest == NULL || day == NULL) {
        return TS_DIGEST_E_ARG;
    }
    days = &digest->days;
    if (index >= days->count) {
        return TS_DIGEST_E_RANGE;
    }
    day->year = read_i32(DAY_COLUMN(DAY_YEAR));
    day->month = *DAY_COLUMN(DAY_MONTH);
    day->day = *DAY_COLUMN(DAY_DAY);
    day->vara = *DAY_COLUMN(DAY_VARA);
    day->state = *DAY_COLUMN(DAY_STATE);
    day->fixed_day = read_i32(DAY_COLUMN(DAY_FIXED));
    day->sunrise = read_i32(DAY_COLUMN(DAY_SUNRISE));
    day->sunset = read_i32(DAY_COLUMN(DAY_SUNSET));
    day->moonrise = read_i32(DAY_COLUMN(DAY_MOONRISE));
    day->moonset = read_i32(DAY_COLUMN(DAY_MOONSET));
    day->moon_phase = read_u16(DAY_COLUMN(DAY_PHASE));
    day->lunar_month = read_u16(DAY_COLUMN(DAY_LUNAR_MONTH));
    day->span_first = read_u32(DAY_COLUMN(DAY_SPAN_FIRST));
    day->span_count = read_u16(DAY_COLUMN(DAY_SPAN_COUNT));
    day->observance_first = read_u32(DAY_COLUMN(DAY_OBSERVANCE_FIRST));
    day->observance_count = read_u16(DAY_COLUMN(DAY_OBSERVANCE_COUNT));
#undef DAY_COLUMN
    if ((uint64_t)day->span_first + day->span_count > digest->spans.count ||
        (uint64_t)day->observance_first + day->observance_count > digest->observances.count) {
        return TS_DIGEST_E_MALFORMED;
    }
    return TS_DIGEST_OK;
}

ts_digest_status ts_digest_span_at(const ts_digest *digest, const ts_digest_day *day,
                                   uint32_t index, ts_digest_span *span) {
    const ts_digest_section *spans;
    size_t at;
    if (digest == NULL || day == NULL || span == NULL) {
        return TS_DIGEST_E_ARG;
    }
    if (index >= day->span_count) {
        return TS_DIGEST_E_RANGE;
    }
    spans = &digest->spans;
    at = (size_t)day->span_first + index;
    if (at >= spans->count) {
        return TS_DIGEST_E_MALFORMED;
    }
    span->kind = (ts_digest_span_kind)column(spans, SPAN_KIND, 1)[at];
    span->name = read_u16(column(spans, SPAN_NAME, 2) + at * 2u);
    span->from = read_i32(column(spans, SPAN_FROM, 4) + at * 4u);
    span->to = read_i32(column(spans, SPAN_TO, 4) + at * 4u);
    return TS_DIGEST_OK;
}

ts_digest_status ts_digest_observance_at(const ts_digest *digest, const ts_digest_day *day,
                                         uint32_t index, ts_digest_observance *observance) {
    const ts_digest_section *observances;
    size_t at;
    if (digest == NULL || day == NULL || observance == NULL) {
        return TS_DIGEST_E_ARG;
    }
    if (index >= day->observance_count) {
        return TS_DIGEST_E_RANGE;
    }
    observances = &digest->observances;
    at = (size_t)day->observance_first + index;
    if (at >= observances->count) {
        return TS_DIGEST_E_MALFORMED;
    }
    observance->name = read_u16(column(observances, OBSERVANCE_NAME, 2) + at * 2u);
    observance->flags = column(observances, OBSERVANCE_FLAGS, 1)[at];
    return TS_DIGEST_OK;
}

ts_digest_status ts_digest_name_at(const ts_digest *digest, uint16_t index, ts_digest_name *name) {
    const ts_digest_section *names;
    size_t at = index;
    uint32_t key_at;
    uint32_t text_at;
    uint32_t key_length;
    uint32_t text_length;
    if (digest == NULL || name == NULL) {
        return TS_DIGEST_E_ARG;
    }
    names = &digest->names;
    if (index >= names->count) {
        return TS_DIGEST_E_RANGE;
    }
    key_at = read_u32(column(names, NAME_KEY_AT, 4) + at * 4u);
    key_length = read_u16(column(names, NAME_KEY_LENGTH, 2) + at * 2u);
    text_at = read_u32(column(names, NAME_TEXT_AT, 4) + at * 4u);
    text_length = read_u16(column(names, NAME_TEXT_LENGTH, 2) + at * 2u);
    if ((uint64_t)key_at + key_length > digest->text.length ||
        (uint64_t)text_at + text_length > digest->text.length) {
        return TS_DIGEST_E_MALFORMED;
    }
    name->key = (const char *)digest->text.start + key_at;
    name->key_length = key_length;
    name->text = (const char *)digest->text.start + text_at;
    name->text_length = text_length;
    return TS_DIGEST_OK;
}

int64_t ts_digest_unix_seconds(const ts_digest *digest, int32_t seconds) {
    return digest->base_unix_seconds + seconds;
}

const char *ts_digest_status_text(ts_digest_status status) {
    switch (status) {
    case TS_DIGEST_OK:
        return "ok";
    case TS_DIGEST_E_ARG:
        return "a null pointer where a value was needed";
    case TS_DIGEST_E_TRUNCATED:
        return "the bytes were cut short";
    case TS_DIGEST_E_MAGIC:
        return "not a result blob";
    case TS_DIGEST_E_LAYOUT:
        return "a result blob layout this reader does not know";
    case TS_DIGEST_E_SCHEMA:
        return "a result blob of another schema";
    case TS_DIGEST_E_VERSION:
        return "a digest format this reader does not know";
    case TS_DIGEST_E_SEAL:
        return "the seal does not match the bytes";
    case TS_DIGEST_E_MALFORMED:
        return "the digest is not shaped as its format says";
    case TS_DIGEST_E_RANGE:
        return "an index past the end";
    }
    return "unknown status";
}
