#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

struct Index;

extern "C" {

replace_ret index_del(Index *idx, char *key, uintptr_t kenlen, kv_comparator *comp);

void index_free(Index *idx);

dict_entry *index_get(Index *idx,
                      char *key,
                      uintptr_t keylen,
                      internal_value *value,
                      kv_comparator *comp);

uint64_t index_key_count(Index *idx);

uint64_t index_memory_used(Index *idx);

Index *index_new();

replace_ret index_put(Index *idx,
                      char *key,
                      uintptr_t kenlen,
                      internal_value value,
                      internal_value *old_value,
                      kv_comparator *comp,
                      bool shrink);

void index_reserve_entry(Index *ptr);

void index_shrink(Index *idx, kv_comparator *comp);

} // extern "C"
