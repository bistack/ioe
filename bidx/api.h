#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

struct Index;

extern "C" {

Index *index_new();

void index_free(Index *idx);

enum replace_ret index_put(struct Index* idx, char *key, size_t kenlen, union internal_value value,
                union internal_value *old_value, struct kv_comparator* comp, bool shrink);

bool index_get(struct Index* idx, char* key, size_t keylen,
                union internal_value* value, struct kv_comparator* comp);

void index_key_count(const Index *ptr);

void index_mem_used(const Index *ptr);

void index_reserve_entry(Index *ptr);

void index_shrink(Index *ptr);

} // extern "C"
