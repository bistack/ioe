const IBLK_SIZE_SHIFT: u32 = 21;
pub const IBLK_MASK: u32 = (1 << IBLK_SIZE_SHIFT) - 1;
pub const PAGE_SHIFT: u32 = 12;
pub const PAGE_MASK: u32 = (1 << PAGE_SHIFT) - 1;
pub const IBLK2PAGE_SHIFT: u32 = IBLK_SIZE_SHIFT - PAGE_SHIFT;
pub const IBLK2PAGE_MASK: u32 = (1 << IBLK2PAGE_SHIFT) - 1;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Addr {
    cid: u32, // container id
    off: u32, // offset in parent
}

impl Addr {
    pub fn new(id: u32, off: u32) -> Addr {
        Addr { cid: id, off: off }
    }

    pub fn get_cid(&self) -> u32 {
        self.cid
    }

    pub fn get_offset(&self) -> u32 {
        self.off
    }
}

pub type Paddr = Addr;

pub type Laddr = Addr;

impl Laddr {
    pub fn align(&mut self) {
        self.off &= IBLK_MASK;
    }
}
