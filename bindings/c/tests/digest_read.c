/*
 * The digest reader's own test (docs/03-design/digest-format.md).
 *
 *   digest_read dump FILE    prints every field; `check-c` holds it to the
 *                            dump the library wrote through its own reader
 *   digest_read refuse FILE  each refusal the format names, by its status
 *   digest_read mutate FILE  the bindings' blob mutator, the seal mended so
 *                            a mutant reaches the checks behind it
 *
 * Built with -fsanitize=address,undefined where the compiler has them.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "teistro_digest.h"

static unsigned char *load(const char *path, size_t *length) {
    FILE *file = fopen(path, "rb");
    long size;
    unsigned char *bytes;
    if (file == NULL || fseek(file, 0, SEEK_END) != 0 || (size = ftell(file)) < 0) {
        fprintf(stderr, "cannot read %s\n", path);
        exit(2);
    }
    rewind(file);
    bytes = malloc((size_t)size + 1u);
    if (bytes == NULL || fread(bytes, 1, (size_t)size, file) != (size_t)size) {
        fprintf(stderr, "cannot read %s\n", path);
        exit(2);
    }
    fclose(file);
    *length = (size_t)size;
    return bytes;
}

static void name(const ts_digest *digest, unsigned index) {
    ts_digest_name found;
    if (index == TS_DIGEST_NO_NAME) {
        printf(" -");
        return;
    }
    if (ts_digest_name_at(digest, (uint16_t)index, &found) != TS_DIGEST_OK) {
        printf(" ?");
        return;
    }
    printf(" %.*s=%.*s", (int)found.key_length, found.key, (int)found.text_length, found.text);
}

static int dump(const ts_digest *digest) {
    uint32_t i;
    uint32_t j;
    printf("days %u base %lld offset %d calendar %u\n", (unsigned)digest->day_count,
           (long long)digest->base_unix_seconds, (int)digest->utc_offset_seconds,
           (unsigned)digest->calendar);
    for (i = 0; i < digest->day_count; i++) {
        ts_digest_day day;
        if (ts_digest_day_at(digest, i, &day) != TS_DIGEST_OK) {
            return 1;
        }
        printf("day %d-%u-%u vara %u state %u fixed %d sun %d %d moon %d %d phase %u month", (int)day.year,
               (unsigned)day.month, (unsigned)day.day, (unsigned)day.vara, (unsigned)day.state,
               (int)day.fixed_day, (int)day.sunrise, (int)day.sunset, (int)day.moonrise,
               (int)day.moonset, (unsigned)day.moon_phase);
        name(digest, day.lunar_month);
        printf("\n");
        for (j = 0; j < day.span_count; j++) {
            ts_digest_span span;
            if (ts_digest_span_at(digest, &day, j, &span) != TS_DIGEST_OK) {
                return 1;
            }
            printf("span %d %d %d", (int)span.kind, (int)span.from, (int)span.to);
            name(digest, span.name);
            printf("\n");
        }
        for (j = 0; j < day.observance_count; j++) {
            ts_digest_observance observance;
            if (ts_digest_observance_at(digest, &day, j, &observance) != TS_DIGEST_OK) {
                return 1;
            }
            printf("observance %u", (unsigned)observance.flags);
            name(digest, observance.name);
            printf("\n");
        }
    }
    return 0;
}

static void reseal(unsigned char *bytes, size_t length) {
    if (length >= TS_DIGEST_SEAL_LEN) {
        ts_digest_seal(bytes, length - TS_DIGEST_SEAL_LEN, (char *)(bytes + length - TS_DIGEST_SEAL_LEN));
    }
}

static void put_u32(unsigned char *at, unsigned value) {
    at[0] = (unsigned char)value;
    at[1] = (unsigned char)(value >> 8);
    at[2] = (unsigned char)(value >> 16);
    at[3] = (unsigned char)(value >> 24);
}

static int expect(const char *what, ts_digest_status got, ts_digest_status wanted) {
    if (got != wanted) {
        printf("FAIL  %s: %s, wanted %s\n", what, ts_digest_status_text(got), ts_digest_status_text(wanted));
        return 1;
    }
    return 0;
}

/* The offset of section id's table entry, or 0. */
static size_t entry_of(const unsigned char *bytes, unsigned id) {
    unsigned count = bytes[8] | (unsigned)bytes[9] << 8;
    unsigned i;
    for (i = 0; i < count; i++) {
        size_t at = 32u + (size_t)i * 16u;
        if (bytes[at] == id) {
            return at;
        }
    }
    return 0;
}

static int refuse(const unsigned char *good, size_t length) {
    ts_digest digest;
    ts_digest_day day;
    ts_digest_span span;
    ts_digest_observance observance;
    ts_digest_name found;
    unsigned char *bytes = malloc(length);
    unsigned offset;
    int failed = 0;
    if (bytes == NULL) {
        return 1;
    }
#define FRESH() memcpy(bytes, good, length)
    failed |= expect("null digest", ts_digest_open(NULL, good, length), TS_DIGEST_E_ARG);
    failed |= expect("null bytes", ts_digest_open(&digest, NULL, length), TS_DIGEST_E_ARG);
    failed |= expect("short", ts_digest_open(&digest, good, 3), TS_DIGEST_E_TRUNCATED);
    failed |= expect("header cut", ts_digest_open(&digest, good, 20), TS_DIGEST_E_TRUNCATED);
    failed |= expect("file cut", ts_digest_open(&digest, good, length - 1), TS_DIGEST_E_TRUNCATED);
    failed |= expect("file cut far", ts_digest_open(&digest, good, length / 2), TS_DIGEST_E_TRUNCATED);
    FRESH(); bytes[0] ^= 1;
    failed |= expect("magic", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_MAGIC);
    FRESH(); bytes[4] = 2;
    failed |= expect("layout", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_LAYOUT);
    FRESH(); bytes[16] = 4;
    failed |= expect("schema", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_SCHEMA);
    FRESH();
    offset = bytes[entry_of(bytes, 1) + 4] | (unsigned)bytes[entry_of(bytes, 1) + 5] << 8;
    bytes[offset] = 2;
    failed |= expect("a later major", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_VERSION);
    FRESH(); bytes[length / 2] ^= 0x40;
    failed |= expect("one bit", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_SEAL);
    FRESH(); bytes[length - 1] ^= 1;
    failed |= expect("the seal's own bit", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_SEAL);
    FRESH();
    put_u32(bytes + 12, (unsigned)length - 8u);
    failed |= expect("a length that lies", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_MALFORMED);
    FRESH(); put_u32(bytes + 8, 0xFFFFFFFFu);
    failed |= expect("a table too long", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_MALFORMED);
    FRESH(); put_u32(bytes + 8, 0);
    failed |= expect("no sections", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_MALFORMED);
    FRESH(); bytes[entry_of(bytes, 9)] = 10;
    failed |= expect("no seal", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_MALFORMED);
    FRESH(); bytes[entry_of(bytes, 3)] = 30;
    reseal(bytes, length);
    failed |= expect("no spans", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_MALFORMED);
    FRESH(); put_u32(bytes + entry_of(bytes, 2) + 12, 0xFFFFu);
    reseal(bytes, length);
    failed |= expect("more days than rows", ts_digest_open(&digest, bytes, length), TS_DIGEST_E_MALFORMED);
    failed |= expect("good", ts_digest_open(&digest, good, length), TS_DIGEST_OK);
    failed |= expect("day past the end", ts_digest_day_at(&digest, digest.day_count, &day), TS_DIGEST_E_RANGE);
    failed |= expect("null day", ts_digest_day_at(NULL, 0, &day), TS_DIGEST_E_ARG);
    failed |= expect("day 0", ts_digest_day_at(&digest, 0, &day), TS_DIGEST_OK);
    failed |= expect("span past the end", ts_digest_span_at(&digest, &day, day.span_count, &span), TS_DIGEST_E_RANGE);
    failed |= expect("null span", ts_digest_span_at(&digest, &day, 0, NULL), TS_DIGEST_E_ARG);
    failed |= expect("observance past the end",
                     ts_digest_observance_at(&digest, &day, day.observance_count, &observance), TS_DIGEST_E_RANGE);
    failed |= expect("null observance", ts_digest_observance_at(&digest, &day, 0, NULL), TS_DIGEST_E_ARG);
    failed |= expect("name past the end", ts_digest_name_at(&digest, 0xFFFEu, &found), TS_DIGEST_E_RANGE);
    failed |= expect("null name", ts_digest_name_at(&digest, 0, NULL), TS_DIGEST_E_ARG);
    day.span_first = digest.spans.count;
    failed |= expect("a forged day's span", ts_digest_span_at(&digest, &day, 0, &span), TS_DIGEST_E_MALFORMED);
    day.observance_first = digest.observances.count;
    day.observance_count = 1;
    failed |= expect("a forged day's observance",
                     ts_digest_observance_at(&digest, &day, 0, &observance), TS_DIGEST_E_MALFORMED);
    {
        ts_digest_status all[] = {TS_DIGEST_OK, TS_DIGEST_E_ARG, TS_DIGEST_E_TRUNCATED, TS_DIGEST_E_MAGIC,
                                  TS_DIGEST_E_LAYOUT, TS_DIGEST_E_SCHEMA, TS_DIGEST_E_VERSION, TS_DIGEST_E_SEAL,
                                  TS_DIGEST_E_MALFORMED, TS_DIGEST_E_RANGE};
        size_t i;
        for (i = 0; i < sizeof all / sizeof all[0]; i++) {
            if (strcmp(ts_digest_status_text(all[i]), "unknown status") == 0) {
                failed = 1;
            }
        }
    }
    if (ts_digest_unix_seconds(&digest, 0) != digest.base_unix_seconds) {
        failed = 1;
    }
#undef FRESH
    free(bytes);
    return failed;
}

static unsigned long state = 0x12345678ul;

static unsigned next_random(void) {
    unsigned long x = state & 0xFFFFFFFFul;
    x ^= (x << 13) & 0xFFFFFFFFul;
    x ^= x >> 17;
    x ^= (x << 5) & 0xFFFFFFFFul;
    state = x;
    return (unsigned)x;
}

static int walk(const ts_digest *digest) {
    uint32_t i;
    for (i = 0; i < digest->day_count && i < 400u; i++) {
        ts_digest_day day;
        uint32_t j;
        if (ts_digest_day_at(digest, i, &day) != TS_DIGEST_OK) {
            return 0;
        }
        for (j = 0; j < day.span_count; j++) {
            ts_digest_span span;
            ts_digest_name found;
            if (ts_digest_span_at(digest, &day, j, &span) != TS_DIGEST_OK) {
                return 0;
            }
            (void)ts_digest_name_at(digest, span.name, &found);
        }
        for (j = 0; j < day.observance_count; j++) {
            ts_digest_observance observance;
            if (ts_digest_observance_at(digest, &day, j, &observance) != TS_DIGEST_OK) {
                return 0;
            }
        }
    }
    return 1;
}

static int mutate(const unsigned char *good, size_t length) {
    static const unsigned edges[] = {0u, 1u, 3u, 4u, 8u, 0x7FFFFFFFu, 0x80000000u, 0xFFFFFFFFu};
    unsigned char *bytes = malloc(length + 64u);
    unsigned rounds;
    unsigned read = 0;
    unsigned refused = 0;
    if (bytes == NULL) {
        return 1;
    }
    for (rounds = 0; rounds < 20000u; rounds++) {
        ts_digest digest;
        size_t size = length;
        memcpy(bytes, good, length);
        switch (next_random() % 4u) {
        case 0: {
            unsigned flips = 1u + next_random() % 8u;
            while (flips-- > 0u) {
                bytes[32u + next_random() % (length - 64u)] ^= (unsigned char)(1u << (next_random() % 8u));
            }
            break;
        }
        case 1:
            put_u32(bytes + ((next_random() % ((unsigned)length - 32u)) & ~3u),
                    next_random() % 9u == 8u ? (unsigned)length : edges[next_random() % 8u]);
            break;
        case 2:
            size = 33u + next_random() % ((unsigned)length - 33u);
            put_u32(bytes + 12, (unsigned)size);
            break;
        default:
            size = length + 8u * (1u + next_random() % 8u);
            memset(bytes + length, 0, size - length);
            put_u32(bytes + 12, (unsigned)size);
            break;
        }
        reseal(bytes, size);
        if (ts_digest_open(&digest, bytes, size) == TS_DIGEST_OK && walk(&digest)) {
            read++;
        } else {
            refused++;
        }
    }
    free(bytes);
    printf("%u mutants read, %u refused\n", read, refused);
    return read == 0u || refused == 0u;
}

int main(int count, char **given) {
    size_t length = 0;
    unsigned char *bytes;
    ts_digest digest;
    int failed;
    if (count != 3) {
        fprintf(stderr, "usage: digest_read dump|refuse|mutate FILE\n");
        return 2;
    }
    bytes = load(given[2], &length);
    if (strcmp(given[1], "refuse") == 0) {
        failed = refuse(bytes, length);
    } else if (strcmp(given[1], "mutate") == 0) {
        failed = mutate(bytes, length);
    } else {
        ts_digest_status status = ts_digest_open(&digest, bytes, length);
        if (status != TS_DIGEST_OK) {
            printf("FAIL  %s\n", ts_digest_status_text(status));
            return 1;
        }
        failed = dump(&digest);
    }
    free(bytes);
    return failed;
}
