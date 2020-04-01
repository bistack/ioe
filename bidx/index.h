#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#define FIXED_KEY 1

enum kLayout {
    KEY_SEQNUM = 7,
    KEY_TYPE = 1,
    KEY_HEADER = KEY_SEQNUM + KEY_TYPE,
#ifdef FIXED_KEY
    KEY_REAL_SIZE = 8,
    INTERNAL_KEY_SIZE = KEY_HEADER + KEY_REAL_SIZE,
#else
    KEY_SIZE = 5,
    INTERNAL_KEY_SIZE = KEY_HEADER,
#endif
    INTERNAL_VAL_SIZE = 8,
    VAL_SIZE_ONDISK = 5
};

/*
struct dict_entry {
#ifdef FIXED_KEY
    char key[INTERNAL_KEY_SIZE];
#else
    void *key;
    size_t keylen;
#endif
    uint64_t vmap;
    char lock;
    struct dict_entry *next;
    struct dict_entry *prev_node;
    struct dict_entry **next_nodes;
};

struct list {
    struct list* prev;
    struct list* next;
};

struct request {
    char* mem_buf;         // in memory memtable buffer
    uint32_t mem_size;     // valid data size of memtable buffer

    union internal_value value;

    char* log_buf;         // in memory value log buffer
    uint32_t log_size;     // valid data size of value log buffer

    char* l2_item_buf;     // < l2 item buffer
    uint32_t l2_item_size; // < l2 item size

    struct dict_entry* *entry;  // update entry
    bool lock;             // for transaction only

    struct list list;      // request list
};
*/
union internal_value {
    struct {
        uint64_t block : 11;    // zone id
        uint64_t offset : 30;   // bytes offset in zone
        uint64_t size : 23;     // value bytes len
    } v;
    uint64_t data;
};

struct kv_comparator {
    void* state;
    void (*destructor)(void*);
    int (*compare)(void*, const char* a, size_t alen, const char* b, size_t blen);
    const char* (*name)(void*);
};

enum replace_ret {
    S_NULL = 0,
    S_OLD = 1,
    S_NEW = 2
};
struct Index;
struct Index* index_new();
void index_free(struct Index* idx);
enum replace_ret index_put(struct Index* idx, char *key, size_t kenlen, union internal_value value,
                union internal_value *old_value, struct kv_comparator* comp, bool shrink);
bool index_get(struct Index* idx, char* key, size_t keylen,
                union internal_value* value, struct kv_comparator* comp);
enum replace_ret index_del(struct Index* idx, char *key, size_t kenlen, struct kv_comparator* comp);
void index_shrink(struct Index* idx, struct kv_comparator* comp);
uint64_t index_memory_used(struct Index* idx);
uint64_t index_key_count(struct Index* idx);
