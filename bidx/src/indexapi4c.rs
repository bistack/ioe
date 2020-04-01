/*
struct memtable* memtable_new(uint64_t memtable_size);
void memtable_free(struct memtable*);
enum status memtable_writebatch(struct vdb* vdb, struct writebatch* batch);
enum status memtable_read(struct vdb* vdb, struct memtable* mtable,
            char* key, size_t keylen, union internal_value* value);
void memtable_shrink(struct vdb* vdb);
uint64_t memtable_memory_used(struct memtable* mem);
uint64_t memtable_key_count(struct memtable* mem);
void memtable_reserve_entry(struct vdb* vdb, struct request* req);
*/

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::null_mut;

use super::addr::{IBLK2PAGE_MASK, Laddr, Paddr};
use super::cmod_index::{
    internal_value, kv_comparator, replace_ret, replace_ret_S_NEW, replace_ret_S_NULL,
    replace_ret_S_OLD,
};
use super::index::{L1Cache, IdxBlock, L2Cache, Pinfo, Pattr};

#[no_mangle]
pub struct Index {
    l1c: L1Cache,
    disk_used: u64,
}

impl Index {
    pub fn new() -> Index {
        Index {
            l1c: L1Cache::new(),
            disk_used: 0,
        }
    }

    pub fn get_l1c_mut(&mut self) -> &mut L1Cache {
        &mut self.l1c
    }

    pub fn get_l1c(&self) -> &L1Cache {
        &self.l1c
    }
}

#[no_mangle]
pub extern "C" fn index_new() -> *mut Index {
    println!("+++ new idx");
    Box::into_raw(Box::new(Index::new()))
}

#[no_mangle]
pub extern "C" fn index_free(idx: *mut Index) {
    unsafe {
    if idx.is_null() {
        return;
    }
    }
    unsafe {
        Box::from_raw(idx);
        println!("--- free idx");
    };
}

#[no_mangle]
pub extern "C" fn index_put(
    idx: *mut Index,
    key: *mut ::std::os::raw::c_char,
    keylen: usize,
    value: internal_value,
    old_value: *mut internal_value,
    comp: *mut kv_comparator,
    shrink: bool,
) -> replace_ret {
    let mut ret = replace_ret_S_NULL;
    if idx.is_null() {
        println!("idx is null");
        return ret;
    }
    let idx = unsafe { &mut *idx };
    if key.is_null() {
        println!("insert key is null");
        return ret;
    }
    if keylen != 17 {
        println!("internal key len is {}, want 17", keylen);
        return ret;
    }
    let kvec = unsafe { Vec::from_raw_parts(key as *mut u8, keylen, keylen) };
    let attr = &kvec[0..8];
    let ukey = &kvec[9..];
    let laddr = Laddr::from(ukey);
    let paddr = Paddr::from(&value);

    let pattr = Pattr::from(attr);
    let mut pinfo = Pinfo::new(paddr);
    pinfo.set_attr(pattr);

    let coff = laddr.get_offset() & IBLK2PAGE_MASK;

    //println!("*** idx {:p} put l2 coff: {}, laddr : {:?}", idx, coff, laddr);

    std::mem::forget(kvec);

    let mut l1c = idx.get_l1c_mut();
    let mut iblk = match l1c.get_mut(&laddr) {
        Some(iblk) => iblk,
        None => {
            //println!("*** idx put l1 no old entry for laddr {:?}", laddr);
            let mut l2c = L2Cache::new();
            l2c.insert_pinfo(coff as usize, pinfo);
            idx.l1c.insert(laddr, IdxBlock::InRam(l2c));
            ret = replace_ret_S_NULL;
            return ret;
        },
    };

    let mut l2c = match iblk {
        IdxBlock::InRam(l2c) => l2c,
        IdxBlock::OnDisk(addr) => {
            println!("TODO: read idx on disk: {:?}", addr);
            return ret;
        }
    };

    match l2c.get_pinfo(coff as usize) {
        Some(pinfo) => {
            let addr = pinfo.get_paddr();
            let mut oldval = internal_value::from(addr);
            unsafe {
                let retval = &mut *old_value;
                retval.data = oldval.data;
            }
            ret = replace_ret_S_NEW;
        },
        None => {
            //println!("idx l2 no old entry");
            ret = replace_ret_S_NULL;
        },
    };

    l2c.insert_pinfo(coff as usize, pinfo);
    return ret;
}

#[no_mangle]
pub extern "C" fn index_get(
    idx: *mut Index,
    key: *mut ::std::os::raw::c_char,
    keylen: usize,
    value: *mut internal_value,
    comp: *mut kv_comparator,
) -> bool {
    let mut ret: bool = false;
    if idx.is_null() {
        println!("idx is null");
        return ret;
    }
    let idx = unsafe { &*idx };
    if key.is_null() {
        println!("read key is null");
        return ret;
    }
    if keylen != 17 {
        println!("internal key len is {}, want 17", keylen);
        return ret;
    }
    let kvec = unsafe { Vec::from_raw_parts(key as *mut u8, keylen, keylen) };
    let attr = &kvec[0..8];
    let ukey = &kvec[9..];
    let laddr = Laddr::from(ukey);
    let coff = laddr.get_offset() & IBLK2PAGE_MASK;
    //println!("*** idx {:p} get l2 coff: {}, laddr : {:?}", idx, coff, laddr);
    std::mem::forget(kvec);

    let l1c = idx.get_l1c();
    let iblk = match l1c.get(&laddr) {
        Some(iblk) => iblk,
        None => {
            println!("idx l1 no entry for laddr: {:?}", laddr);
            return ret;
        },
    };

    let l2c = match iblk {
        IdxBlock::InRam(l2c) => l2c,
        IdxBlock::OnDisk(addr) => {
            println!("TODO: read idx from disk");
            return ret;
        },
    };

    match l2c.get_pinfo(coff as usize) {
        Some(pinfo) => {
            let addr = pinfo.get_paddr();
            let val = internal_value::from(addr);
            unsafe {
                let retval = &mut *value;
                retval.data = val.data;
            }
            ret = true;
            //println!("*** get val at paddr: {:?}", addr);
        },
        None => {
            //println!("idx l2 not found entry");
        },
    }

    return ret;
}

#[no_mangle]
pub extern "C" fn index_del(
    idx: *mut Index,
    key: *mut ::std::os::raw::c_char,
    keylen: usize,
    comp: *mut kv_comparator,
) -> replace_ret {
    let mut ret = replace_ret_S_NULL;
    if idx.is_null() {
        println!("idx is null");
        return ret;
    }
    let idx = unsafe { &mut *idx };
    if key.is_null() {
        println!("read key is null");
        return ret;
    }
    if keylen != 17 {
        println!("internal key len is {}, want 17", keylen);
        return ret;
    }
    let kvec = unsafe { Vec::from_raw_parts(key as *mut u8, keylen, keylen) };
    let attr = &kvec[0..8];
    let ukey = &kvec[9..];
    println!("*** del key is {:?}", ukey);
    let laddr = Laddr::from(ukey);
    let coff = laddr.get_offset() & IBLK2PAGE_MASK;

    std::mem::forget(kvec);

    let iblk = match idx.get_l1c_mut().get_mut(&laddr) {
        Some(iblk) => iblk,
        None => {
            println!("idx no key: {:?}", laddr);
            return replace_ret_S_NULL;
        }
    };

    let l2c = match iblk {
        IdxBlock::InRam(l2c) => l2c,
        IdxBlock::OnDisk(addr) => {
            println!("TODO: read idx from disk");
            return ret;
        }
    };

    match l2c.del_pinfo(coff as usize) {
        Some(_) => {
            ret = replace_ret_S_OLD;
        }
        None => {
            println!("idx not found old val");
        }
    }

    return ret;
}

#[no_mangle]
pub extern "C" fn index_shrink(idx: *mut Index, comp: *mut kv_comparator) {}

#[no_mangle]
pub extern "C" fn index_memory_used(idx: *mut Index) -> u64 {
    100
}

#[no_mangle]
pub extern "C" fn index_key_count(idx: *mut Index) -> u64 {
    1000
}

#[no_mangle]
pub extern "C" fn index_reserve_entry(ptr: *mut Index) {}
