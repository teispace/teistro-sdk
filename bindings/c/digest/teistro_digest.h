/*
 * The N-day digest's reader (docs/03-design/digest-format.md).
 *
 * One C file and this header, with no dependency and no allocation: a
 * home-screen widget or a watch complication opens the bytes the SDK wrote
 * and reads days out of them without the engine. Swift calls it through a
 * bridging header, Kotlin through the NDK.
 *
 * A ts_digest holds pointers into the bytes it was opened over, so the
 * bytes must outlive it. Every name and key points into them too, and is
 * UTF-8 with no terminator: use its length.
 *
 *     ts_digest digest;
 *     if (ts_digest_open(&digest, bytes, length) != TS_DIGEST_OK) return;
 *     ts_digest_day day;
 *     ts_digest_day_at(&digest, 0, &day);
 *     for (uint32_t i = 0; i < day.span_count; i++) {
 *         ts_digest_span span;
 *         ts_digest_span_at(&digest, &day, i, &span);
 *     }
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef TEISTRO_DIGEST_H
#define TEISTRO_DIGEST_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* The format major version this reader reads. */
#define TS_DIGEST_FORMAT_MAJOR 1u
/* An instant a day does not have: no moonrise, no sunset. */
#define TS_DIGEST_NO_TIME INT32_MIN
/* A span with no name of its own: Abhijit, a brahma muhurta. */
#define TS_DIGEST_NO_NAME 0xFFFFu
/* The length of ts_digest_sha256's output. */
#define TS_DIGEST_HASH_LEN 32u
/* The length of the seal: that hash in lowercase hexadecimal, unterminated. */
#define TS_DIGEST_SEAL_LEN 64u

/* Why a digest was refused. */
typedef enum ts_digest_status {
    TS_DIGEST_OK = 0,
    /* A null pointer where a value was needed. */
    TS_DIGEST_E_ARG = -1,
    /* Fewer bytes than the header says: a file cut short. */
    TS_DIGEST_E_TRUNCATED = -2,
    /* Not a result blob at all. */
    TS_DIGEST_E_MAGIC = -3,
    /* A result blob of a layout version this reader does not know. */
    TS_DIGEST_E_LAYOUT = -4,
    /* A result blob of another schema. */
    TS_DIGEST_E_SCHEMA = -5,
    /* A digest of a format major version this reader does not know. */
    TS_DIGEST_E_VERSION = -6,
    /* The seal does not match the bytes. */
    TS_DIGEST_E_SEAL = -7,
    /* Sealed, and still not shaped as the format says. */
    TS_DIGEST_E_MALFORMED = -8,
    /* An index past the end of what was asked of. */
    TS_DIGEST_E_RANGE = -9
} ts_digest_status;

/* What a span is. The numbers are the format's, and never change. */
typedef enum ts_digest_span_kind {
    TS_DIGEST_SPAN_TITHI = 1,
    TS_DIGEST_SPAN_NAKSHATRA = 2,
    TS_DIGEST_SPAN_YOGA = 3,
    TS_DIGEST_SPAN_KARANA = 4,
    TS_DIGEST_SPAN_RAHU_KAALA = 5,
    TS_DIGEST_SPAN_YAMAGHANDA = 6,
    TS_DIGEST_SPAN_GULIKA_KAALA = 7,
    TS_DIGEST_SPAN_CHOGHADIYA = 8,
    TS_DIGEST_SPAN_HORA = 9,
    TS_DIGEST_SPAN_ABHIJIT = 10,
    TS_DIGEST_SPAN_BHADRA = 11,
    TS_DIGEST_SPAN_BRAHMA_MUHURTA = 12,
    TS_DIGEST_SPAN_DURMUHURTA = 13,
    TS_DIGEST_SPAN_VARJYAM = 14,
    TS_DIGEST_SPAN_AMRIT_KALAM = 15,
    TS_DIGEST_SPAN_PANCHAKA = 16,
    TS_DIGEST_SPAN_MOON_SIGN = 17,
    TS_DIGEST_SPAN_SUN_SIGN = 18
} ts_digest_span_kind;

/* One section's place in the bytes; a reader's own. */
typedef struct ts_digest_section {
    const uint8_t *start;
    uint32_t length;
    uint32_t count;
} ts_digest_section;

/* An open digest. Fixed in size, so it lives on the caller's stack. */
typedef struct ts_digest {
    const uint8_t *bytes;
    size_t length;
    uint32_t format_minor;
    uint32_t day_count;
    int64_t base_unix_seconds;
    /* The first day's offset; a day's own is in ts_digest_day. */
    int32_t utc_offset_seconds;
    uint32_t calendar;
    double latitude_deg;
    double longitude_deg;
    double altitude_m;
    ts_digest_section days;
    ts_digest_section spans;
    ts_digest_section observances;
    ts_digest_section names;
    ts_digest_section text;
    ts_digest_section locale;
    ts_digest_section provenance;
    /* Each day's offset; start is null in a format 1.0 digest. */
    ts_digest_section clock;
} ts_digest;

/* One day. Every time is whole seconds from base_unix_seconds. */
typedef struct ts_digest_day {
    /* The date in the digest's calendar. */
    int32_t year;
    uint8_t month;
    uint8_t day;
    /* 0 for Sunday. */
    uint8_t vara;
    /* 0 an ordinary day, 1 the Sun did not rise, 2 it did not set. */
    uint8_t state;
    int32_t fixed_day;
    int32_t sunrise;
    int32_t sunset;
    int32_t moonrise;
    int32_t moonset;
    /* Thousandths of a lunation at sunrise: 0 new, 500 full. */
    uint16_t moon_phase;
    /* The lunar month's name. */
    uint16_t lunar_month;
    uint32_t span_first;
    uint32_t span_count;
    uint32_t observance_first;
    uint32_t observance_count;
    /* The offset the day was read on, seconds east of UTC, at its opening.
       A format 1.0 digest has only the first day's, given for every day. */
    int32_t utc_offset_seconds;
} ts_digest_day;

/* A span of a day. */
typedef struct ts_digest_span {
    ts_digest_span_kind kind;
    /* Into ts_digest_name_at, or TS_DIGEST_NO_NAME. */
    uint16_t name;
    int32_t from;
    int32_t to;
} ts_digest_span;

/* An observance of a day. */
typedef struct ts_digest_observance {
    uint16_t name;
    /* Bit 0: a fast. Bit 1: a published set decided the day. */
    uint8_t flags;
} ts_digest_observance;

/* A name: its catalogue or rule key, and the text in the digest's locale. */
typedef struct ts_digest_name {
    const char *key;
    uint32_t key_length;
    const char *text;
    uint32_t text_length;
} ts_digest_name;

/* Reads the bytes' header, table and seal. Nothing is copied. */
ts_digest_status ts_digest_open(ts_digest *digest, const void *bytes, size_t length);

/* A day, 0 to day_count - 1. */
ts_digest_status ts_digest_day_at(const ts_digest *digest, uint32_t index, ts_digest_day *day);

/* A day's span, 0 to its span_count - 1. */
ts_digest_status ts_digest_span_at(const ts_digest *digest, const ts_digest_day *day,
                                   uint32_t index, ts_digest_span *span);

/* A day's observance, 0 to its observance_count - 1. */
ts_digest_status ts_digest_observance_at(const ts_digest *digest, const ts_digest_day *day,
                                         uint32_t index, ts_digest_observance *observance);

/* A name, by a span's or an observance's name or a day's lunar_month. */
ts_digest_status ts_digest_name_at(const ts_digest *digest, uint16_t index, ts_digest_name *name);

/* A time of a day as seconds since 1970. */
int64_t ts_digest_unix_seconds(const ts_digest *digest, int32_t seconds);

/* What a status means, in English, for a log. */
const char *ts_digest_status_text(ts_digest_status status);

/* SHA-256 of the bytes, into out. */
void ts_digest_sha256(const void *bytes, size_t length, uint8_t out[TS_DIGEST_HASH_LEN]);

/* The seal of the bytes, into out: SHA-256 in lowercase hexadecimal, no NUL. */
void ts_digest_seal(const void *bytes, size_t length, char out[TS_DIGEST_SEAL_LEN]);

#ifdef __cplusplus
}
#endif

#endif /* TEISTRO_DIGEST_H */
