#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

struct db;
typedef struct db   kvnvmedb_t;

struct kv_comparator {
    void* state;
    void (*destructor)(void*);
    int (*compare)(void* state, const char* a, size_t alen, const char* b, size_t blen);
    const char* (*name)(void*);
};

struct kv_merge {
    void* state;
    void (*merge_func)(void* state,
                    const char* key, size_t keylen,
                    const char* old_val, size_t old_vallen,
                    const char* merge_val, size_t merge_vallen,
                    char** val_out, size_t* vallen_out);
};

struct kv_compaction_filter {
    void* state;
    bool (*filter_func)(void* state,
                    const char* key, size_t keylen,
                    const char* val, size_t vallen,
                    char** val_out, size_t* vallen_out,
                    bool* value_changed);
};

struct kv_cache;
/* database open options */
struct kv_options {
    unsigned char create_if_missing;       /* create database if it is not exist */
    unsigned char error_if_exists;         /* return error if database is already exist */
    unsigned char memory_index;            /* no sst, only use memtable */
    unsigned char low_level_fmt;           /* do low level format or not */
    double dirty_ratio_thr;                /* dirty ratio threshold */
    uint64_t memtable_size;                /* memtable size */
    uint64_t wal_ver_gap;                  /* max version gap for wal */
    uint64_t wal_sec_gap;                  /* max seconds gap for wal */
    uint32_t wal_cache_size;               /* wal cache size */
    uint32_t wal_pre_rd_size;              /* wal pre-read size */
    uint32_t cp_downloader_threadnum;      /* checkpoint downloader threadnum */
    struct kv_comparator* comparator;
    struct kv_merge* merge;
    struct kv_compaction_filter* compaction_filter;
    struct kv_cache* cache;
    char*  iostat_path;                    /* excutable iostat tool path , e.g: "/bin/ocnvme" */
};

struct list {
    struct list* prev;
    struct list* next;
};

struct kv_snapshot {
    uint64_t seqnum;
    struct list list;
};

/* key/value write options */
struct kv_write_options {
    unsigned char sync;                    /* sync operation */
};

/* key/value read options */
struct kv_read_options {
    struct kv_snapshot* version;
};

typedef struct kv_options               kvnvme_options_t;
typedef struct kv_write_options         kvnvme_writeoptions_t;
typedef struct kv_read_options          kvnvme_readoptions_t;

extern kvnvmedb_t* kvnvme_open(const kvnvme_options_t* options,
                               const char* dbname, char** errptr);
extern void kvnvme_close(kvnvmedb_t* db);
extern char* kvnvme_dbname(kvnvmedb_t* db);
extern void kvnvme_close_err(kvnvmedb_t* db, char** errptr);
extern void kvnvme_put(kvnvmedb_t* db,
                        const kvnvme_writeoptions_t* options,
                        const char* key, size_t keylen, const char* val,
                        size_t vallen, char** errptr);
extern void kvnvme_delete(kvnvmedb_t* db,
                        const kvnvme_writeoptions_t* options,
                        const char* key, size_t keylen,
                        char** errptr);

extern char* kvnvme_get(kvnvmedb_t* db,
                        const kvnvme_readoptions_t* options,
                        const char* key, size_t keylen, size_t* vallen,
                        char** errptr);

