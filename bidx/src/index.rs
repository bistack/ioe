// in RAM index struct

use std::collections::HashMap;
use std::fmt;
use std::hash::BuildHasherDefault;

use twox_hash::XxHash;
use serde::{Serialize, Deserialize};

use super::addr::{Laddr, Paddr, IBLK2PAGE_SHIFT};

#[derive(Debug)]
pub struct L1Cache {
    index: HashMap<Laddr, IdxBlock, BuildHasherDefault<XxHash>>,
}

impl L1Cache {
    pub fn new() -> L1Cache {
        L1Cache {
            index: Default::default(),
        }
    }

    fn get_aligned_key(k: &Laddr) -> Laddr {
        let off = k.get_offset() >> IBLK2PAGE_SHIFT << IBLK2PAGE_SHIFT;
        Laddr::new(k.get_cid(), off)
    }

    pub fn insert(&mut self, k: Laddr, v: IdxBlock) -> Option<IdxBlock> {
        let rk = L1Cache::get_aligned_key(&k);
        //println!("insert l1: laddr: {:?}, key: {:?}", k, rk);
        let ov = self.index.insert(rk, v);

        // check
        //let ck = Laddr::new(k.get_cid(), k.get_offset());
        //let cv = self.index.get(&ck);
        //println!("insert l1 laddr: {:?} {:#?},\n ck {:?} {:#?}", k, ov, ck, cv);

        ov
    }

    pub fn get(&self, k: &Laddr) -> Option<&IdxBlock> {
        let rk = L1Cache::get_aligned_key(&k);
        let v = self.index.get(&rk);
        //println!("get l1: laddr: {:?}, key: {:?}, v: {:#?}", k, rk, v);

        v
    }

    pub fn get_mut(&mut self, k: &Laddr) -> Option<&mut IdxBlock> {
        let rk = L1Cache::get_aligned_key(&k);
        self.index.get_mut(&rk)
    }
}

#[derive(Debug)]
pub enum IdxBlock {
    InRam(L2Cache),
    OnDisk(Paddr),
}

const ITEMS_LIMITS: usize = 512;
const IDX_BLK_SIZE4K: usize = 1;

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Pattr {
    pub keyType: u8,
    pub seqnum: u64,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Pinfo {
    paddr: Paddr,
    attr: Option<Pattr>,       // maybe optimized and remove
}

impl Pinfo {
    pub fn new(paddr: Paddr) -> Pinfo {
        Pinfo {
            paddr: paddr,
            attr: None,
        }
    }

    pub fn set_attr(&mut self, attr: Pattr) {
        self.attr = Some(attr);
    }

    pub fn get_attr(&self) -> Option<Pattr> {
        self.attr
    }

    pub fn get_paddr(&self) -> &Paddr {
        &self.paddr
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct L2Cache {
    p_array: Vec<Option<Pinfo>>,
}

impl fmt::Debug for L2Cache {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.p_array[..].fmt(formatter)
    }
}

impl L2Cache {
    pub fn new() -> L2Cache {
        L2Cache {
            p_array: vec![None; ITEMS_LIMITS],
        }
    }

    pub fn get_pinfo(&self, offset: usize) -> Option<&Pinfo> {
        match self.p_array.get(offset) {
            Some(p) => p.as_ref(),
            None => None,
        }
    }

    pub fn insert_pinfo(&mut self, offset: usize, pinfo: Pinfo) -> Result<Option<Pinfo>, String> {
        if offset >= ITEMS_LIMITS {
            let err = format!("offset: {} >= limit: {}", offset, ITEMS_LIMITS);
            println!("{}", &err);
            return Err(err);
        }

        let old = self.p_array[offset];
        self.p_array[offset] = Some(pinfo);
        return Ok(old);
    }

    pub fn del_pinfo(&mut self, offset: usize) -> Option<Pinfo> {
        if offset >= ITEMS_LIMITS {
            return None;
        }
        let paddr = Paddr::new(0, 0);
        let pinfo = Pinfo::new(paddr);
        let old = self.p_array[offset];
        self.p_array[offset] = Some(pinfo);
        return old;
    }

    pub fn deserialize(buf: &Vec<u8>) -> Result<L2Cache, String> {
        let l2c = bincode::deserialize(buf);
        if l2c.is_err() {
            let raw_err = l2c.unwrap_err();
            let err = format!("deserialize err:{}", &raw_err);
            return Err(err);
        }
        let l2c = l2c.unwrap();
        Ok(l2c)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let bin = bincode::serialize(&self);
        if bin.is_err() {
            let raw_err = bin.unwrap_err();
            let err = format!("serialize err: {}", &raw_err);
            return Err(err);
        }
        let bin = bin.unwrap();
        Ok(bin)
    }
}
