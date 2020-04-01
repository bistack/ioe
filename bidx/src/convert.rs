use std::convert::{From, Into};
use std::ffi::CString;
use std::os::raw::{c_char, c_ulonglong};

use bytevec::{ByteDecodable, ByteEncodable};

use super::addr::{Laddr, Paddr};
use super::addr::{IBLK_MASK, PAGE_SHIFT};
use super::cmod_index::{self, internal_value};
use super::index::Pattr;

impl From<&internal_value> for Paddr {
    fn from(vc: &internal_value) -> Paddr {
        Paddr::from_cv(vc)
    }
}

impl Paddr {
    pub fn from_cv(vc: &internal_value) -> Paddr {
        vc.into_paddr()
    }

    pub fn into_cv(&self) -> internal_value {
        internal_value::from_paddr(self)
    }
}

impl From<&Paddr> for internal_value {
    fn from(paddr: &Paddr) -> internal_value {
        internal_value::from_paddr(paddr)
    }
}

impl internal_value {
    pub fn from_paddr(paddr: &Paddr) -> internal_value {
        let mut vc = internal_value { data: 0 };
        unsafe {
            vc.v.set_size(1 << PAGE_SHIFT);
            vc.v.set_block(paddr.get_cid() as c_ulonglong);
            vc.v.set_offset(paddr.get_offset() as c_ulonglong);
        }
        vc
    }

    pub fn into_paddr(&self) -> Paddr {
        let mut cid: u32 = 0;
        let mut offset: u32 = 0;
        unsafe {
            cid = self.v.block() as u32;
            offset = self.v.offset() as u32;
        }
        Paddr::new(cid, offset)
    }
}

type CKey = CString;

impl From<CKey> for Laddr {
    fn from(ck: CKey) -> Laddr {
        Laddr::from_ckey(ck)
    }
}

impl From<Laddr> for CKey {
    fn from(laddr: Laddr) -> CKey {
        laddr.into_ckey()
    }
}

impl Laddr {
    pub fn into_ckey(self) -> CKey {
        let v = <Laddr as Into<Vec<u8>>>::into(self);
        let ck = match CKey::new(v) {
            Ok(k) => k,
            Err(rerr) => {
                panic!();
            }
        };
        ck
    }

    pub fn from_ckey(ck: CKey) -> Laddr {
        let b = ck.into_bytes();
        <Laddr as From<&[u8]>>::from(&b)
    }
}

impl Into<Vec<u8>> for Laddr {
    fn into(self) -> Vec<u8> {
        let la = ((self.get_cid() as u64) << 32) & (self.get_offset() as u64);
        match u64::encode::<u64>(&la) {
            Ok(c) => c,
            Err(rerr) => {
                let err = format!("{}", &rerr);
                panic!(err);
            }
        }
    }
}

impl From<&[u8]> for Laddr {
    fn from(v: &[u8]) -> Laddr {
        let addr = match u64::decode::<u64>(v) {
            Ok(d) => d,
            Err(rerr) => {
                let err = format!("v {:?} convert to Laddr failed, err: {}", v, &rerr);
                panic!(err);
            }
        };

        let caddr = addr & (IBLK_MASK as u64);
        Laddr::new((addr >> 32) as u32, (addr << 32 >> 32) as u32)
    }
}

impl From<&[u8]> for Pattr {
    fn from(v: &[u8]) -> Pattr {
        if v.len() != cmod_index::kLayout_KEY_HEADER as usize {
            let err = format!("v {:?} convert to Pattr failed, len: {}", v, v.len());
            panic!(err);
        }
        let ktype = v[cmod_index::kLayout_KEY_HEADER as usize -1];

        Pattr {
            keyType: ktype,
            seqnum: 0,
        }
    }
}