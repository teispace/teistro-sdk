/* The digest reader's structs as the C compiler lays them out, for the
   test that holds the fuzz harness's Rust mirrors to them. */
#include <stddef.h>

#include "teistro_digest.h"

size_t ts_fuzz_sizeof_digest(void) { return sizeof(ts_digest); }
size_t ts_fuzz_sizeof_day(void) { return sizeof(ts_digest_day); }
size_t ts_fuzz_sizeof_span(void) { return sizeof(ts_digest_span); }
size_t ts_fuzz_sizeof_observance(void) { return sizeof(ts_digest_observance); }
size_t ts_fuzz_sizeof_name(void) { return sizeof(ts_digest_name); }
