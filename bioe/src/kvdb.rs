use super::cmod_kv::{
    kv_options, kv_read_options, kv_snapshot, kv_write_options, kvnvme_options_t,
    kvnvme_readoptions_t, kvnvme_writeoptions_t, list,
};
use std::ptr::null_mut;

impl kv_options {
    pub fn new() -> kvnvme_options_t {
        kvnvme_options_t {
            create_if_missing: 1,
            error_if_exists: 1,
            memory_index: 1,
            low_level_fmt: 1,
            dirty_ratio_thr: 0.1,
            memtable_size: 1024,
            wal_ver_gap: 0,
            wal_sec_gap: 0,
            wal_cache_size: 1024,
            wal_pre_rd_size: 16,
            cp_downloader_threadnum: 1,
            comparator: null_mut(),
            merge: null_mut(),
            compaction_filter: null_mut(),
            cache: null_mut(),
            iostat_path: null_mut(),
        }
    }
}

impl kv_write_options {
    pub fn new() -> kvnvme_writeoptions_t {
        kvnvme_writeoptions_t { sync: 1 }
    }
}

impl kv_snapshot {
    pub fn new() -> kv_snapshot {
        kv_snapshot {
            seqnum: 0,
            list: list {
                prev: null_mut(),
                next: null_mut(),
            },
        }
    }
}

impl kv_read_options {
    pub fn new(snap: &mut kv_snapshot) -> kvnvme_readoptions_t {
        kvnvme_readoptions_t {
            version: snap as *mut kv_snapshot,
        }
    }
}
