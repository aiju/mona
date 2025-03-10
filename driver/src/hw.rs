use std::{
    fs::{File, OpenOptions},
    os::unix::fs::OpenOptionsExt,
};

use memmap::MmapOptions;

pub const MEM_START: u32 = 0x10000000;
pub const MEM_LEN: u32 = 16 * 1024 * 1024;
pub const REG_START: u32 = 0xFF200000;
pub const REG_LEN: u32 = 4096;

pub const R_STATUS: u32 = 0x000;
pub const R_CONTROL: u32 = 0x004;
pub const R_DISPLAY_FB: u32 = 0x008;
pub const R_RENDER_TARGET: u32 = 0x00C;

pub const B_STATUS_SEEN_VSYNC: u32 = 1;
pub const B_STATUS_IN_VSYNC: u32 = 2;
pub const B_STATUS_FLUSHED: u32 = 4;
pub const B_STATUS_CLEAR_BUSY: u32 = 8;

pub const B_CONTROL_START: u32 = 1;
pub const B_CONTROL_FLUSH: u32 = 2;
pub const B_CONTROL_INVALIDATE_DEPTH: u32 = 4;
pub const B_CONTROL_CLEAR: u32 = 8;

pub const R_DEPTH_BUFFER: u32 = 0x010;
pub const R_DEPTH_MODE: u32 = 0x014;
pub const R_CLEAR_ADDR: u32 = 0x018;
pub const R_CLEAR_STRIDE: u32 = 0x01C;
pub const R_CLEAR_WIDTH_HEIGHT: u32 = 0x020;
pub const R_CLEAR_DATA: u32 = 0x024;

pub const R_TEXTURE_EN: u32 = 0x028;
pub const R_TEXTURE_ADDR: u32 = 0x02C;

pub const R_STATS_ENABLED: u32 = 0x030;

pub const R_TEXT_EN: u32 = 0x080;
pub const B_TEXT_EN: u32 = 1;
pub const R_TEXT_ACCESS: u32 = 0x084;
pub const B_TEXT_ACCESS_FONT: u32 = 3 << 30;
pub const B_TEXT_ACCESS_TEXT: u32 = 2 << 30;
pub const R_TEXT_TRANSPARENT: u32 = 0x088;

pub struct Hw {
    file: File,
    regs: memmap::MmapMut,
    mem: memmap::MmapMut,
}

impl Hw {
    pub fn new() -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_SYNC)
            .open("/dev/mem")?;
        let regs = unsafe {
            MmapOptions::new()
                .len(REG_LEN as usize)
                .offset(REG_START as u64)
                .map_mut(&file)?
        };
        let mem = unsafe {
            MmapOptions::new()
                .len(MEM_LEN as usize)
                .offset(MEM_START as u64)
                .map_mut(&file)?
        };
        Ok(Hw { file, regs, mem })
    }
    pub unsafe fn read<T>(&mut self, addr: u32) -> T {
        assert!(addr >= MEM_START);
        assert!(addr + size_of::<T>() as u32 <= MEM_START + MEM_LEN);
        unsafe {
            let ptr = self.mem.as_mut_ptr().byte_add((addr - MEM_START) as usize) as *mut T;
            ptr.read_volatile()
        }
    }
    pub fn write<T>(&mut self, addr: u32, value: T) {
        assert!(addr >= MEM_START);
        assert!(addr + size_of::<T>() as u32 <= MEM_START + MEM_LEN);
        unsafe {
            let ptr = self.mem.as_mut_ptr().byte_add((addr - MEM_START) as usize) as *mut T;
            ptr.write_volatile(value);
        }
    }
    pub fn mem_mut(&mut self, addr: u32, len: u32) -> &mut [u8] {
        assert!(addr >= MEM_START);
        assert!(addr + len <= MEM_START + MEM_LEN);
        &mut self.mem[(addr - MEM_START) as usize..(addr + len - MEM_START) as usize]
    }
    pub fn get_reg(&mut self, addr: u32) -> u32 {
        assert!(addr < REG_LEN && (addr & 3) == 0);
        unsafe {
            let ptr = self.regs.as_mut_ptr().byte_add(addr as usize) as *mut u32;
            ptr.read_volatile()
        }
    }
    pub fn set_reg(&mut self, addr: u32, value: u32) {
        assert!(addr < REG_LEN && (addr & 3) == 0);
        unsafe {
            let ptr = self.regs.as_mut_ptr().byte_add(addr as usize) as *mut u32;
            ptr.write_volatile(value);
        }
    }
    pub fn clear(&mut self, addr: u32, stride: u16, width: u16, height: u16, value: u32) {
        assert!(addr % 64 == 0 && stride % 4 == 0 && width % 4 == 0);
        self.set_reg(R_CLEAR_ADDR, addr);
        self.set_reg(R_CLEAR_STRIDE, stride as u32 / 4);
        self.set_reg(
            R_CLEAR_WIDTH_HEIGHT,
            width as u32 / 4 | (height as u32) << 16,
        );
        self.set_reg(R_CLEAR_DATA, value);
        self.set_reg(R_CONTROL, B_CONTROL_CLEAR);
        while self.get_reg(R_STATUS) & B_STATUS_CLEAR_BUSY != 0 {
            std::thread::yield_now();
        }
    }
    pub fn flush_pipeline(&mut self) {
        self.set_reg(R_CONTROL, B_CONTROL_FLUSH);
        while self.get_reg(R_STATUS) & B_STATUS_FLUSHED == 0 {
            std::thread::yield_now();
        }
        self.set_reg(R_STATUS, B_STATUS_FLUSHED);
    }
    pub fn wait_for_vsync(&mut self) {
        while self.get_reg(R_STATUS) & B_STATUS_IN_VSYNC == 0
            || self.get_reg(R_STATUS) & B_STATUS_SEEN_VSYNC == 0
        {
            std::thread::yield_now();
        }
        self.set_reg(R_STATUS, B_STATUS_SEEN_VSYNC);
    }
}

#[derive(Debug)]
pub struct VramAlloc {
    free: Vec<(usize, usize)>
}

impl VramAlloc {
    pub fn new(start: usize, len: usize) -> Self {
        Self {
            free: vec![(start, len)]
        }
    }
    pub fn alloc(&mut self, len: usize) -> Option<u32> {
        for i in 0..self.free.len() {
            let (r_start, r_len) = &mut self.free[i];
            if len <= *r_len {
                let ret = *r_start;
                *r_start += len;
                *r_len -= len;
                if *r_len == 0 {
                    self.free.remove(i);
                }
                return Some(ret as u32);
            }
        }
        return None;
    }
    pub fn reserve(&mut self, start: usize, len: usize) {
        let end = start + len;
        self.free = self.free.iter().copied().flat_map(|(r_start, r_len)| {
            let r_end = r_start + r_len;
            if r_start >= end || r_end < start {
                vec![(r_start, r_len)]
            } else if start <= r_start && end >= r_end {
                vec![]
            } else if start <= r_start && r_start < end && end < r_end {
                vec![(end, r_end - end)]
            } else if r_start < start && start < r_end && r_end <= end {
                vec![(r_start, start - r_start)]
            } else {
                vec![(r_start, start - r_start), (end, r_end - end)]
            }
        }).collect();
    }
}
