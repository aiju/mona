#![allow(dead_code)]

use rs_common::*;
use std::{
    fs::{File, OpenOptions},
    io::Read,
    os::unix::fs::OpenOptionsExt,
    time,
};

const MEM_START: u32 = 0x10000000;
const MEM_LEN: u32 = 16 * 1024 * 1024;
const REG_START: u32 = 0xFF200000;
const REG_LEN: u32 = 4096;

const R_STATUS: u32 = 0x000;
const R_CONTROL: u32 = 0x004;
const R_DISPLAY_FB: u32 = 0x008;
const R_RENDER_TARGET: u32 = 0x00C;

const B_STATUS_CLEAR_BUSY: u32 = 8;
const B_CONTROL_CLEAR: u32 = 8;

const R_DEPTH_MODE: u32 = 0x014;
const R_CLEAR_ADDR: u32 = 0x018;
const R_CLEAR_STRIDE: u32 = 0x01C;
const R_CLEAR_WIDTH_HEIGHT: u32 = 0x020;
const R_CLEAR_DATA: u32 = 0x024;

const R_TEXTURE_EN: u32 = 0x028;
const R_TEXTURE_ADDR: u32 = 0x02C;

const FRAMEBUFFER1: u32 = MEM_START;
const FRAMEBUFFER2: u32 = MEM_START + 4 * 1048576;
const TEXTUREBUFFER: u32 = MEM_START + 6 * 1048576;

const DEPTHBUFFER: u32 = MEM_START + 8 * 1048576;

use memmap::MmapOptions;

struct Hw {
    file: File,
    regs: memmap::MmapMut,
    mem: memmap::MmapMut,
}

impl Hw {
    fn new() -> Result<Self, std::io::Error> {
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
    unsafe fn read<T>(&mut self, addr: u32) -> T {
        assert!(addr >= MEM_START);
        assert!(addr + size_of::<T>() as u32 <= MEM_START + MEM_LEN);
        unsafe {
            let ptr = self.mem.as_mut_ptr().byte_add((addr - MEM_START) as usize) as *mut T;
            ptr.read_volatile()
        }
    }
    fn write<T>(&mut self, addr: u32, value: T) {
        assert!(addr >= MEM_START);
        assert!(addr + size_of::<T>() as u32 <= MEM_START + MEM_LEN);
        unsafe {
            let ptr = self.mem.as_mut_ptr().byte_add((addr - MEM_START) as usize) as *mut T;
            ptr.write_volatile(value);
        }
    }
    fn mem_mut(&mut self, addr: u32, len: u32) -> &mut [u8] {
        assert!(addr >= MEM_START);
        assert!(addr + len <= MEM_START + MEM_LEN);
        &mut self.mem[(addr - MEM_START) as usize..(addr + len - MEM_START) as usize]
    }
    fn get_reg(&mut self, addr: u32) -> u32 {
        assert!(addr < REG_LEN && (addr & 3) == 0);
        unsafe {
            let ptr = self.regs.as_mut_ptr().byte_add(addr as usize) as *mut u32;
            ptr.read_volatile()
        }
    }
    fn set_reg(&mut self, addr: u32, value: u32) {
        assert!(addr < REG_LEN && (addr & 3) == 0);
        unsafe {
            let ptr = self.regs.as_mut_ptr().byte_add(addr as usize) as *mut u32;
            ptr.write_volatile(value);
        }
    }
    fn clear(&mut self, addr: u32, stride: u16, width: u16, height: u16, value: u32) {
        assert!(addr % 64 == 0 && stride % 4 == 0 && width % 4 == 0);
        self.set_reg(R_CLEAR_ADDR, addr);
        self.set_reg(R_CLEAR_STRIDE, stride as u32 / 4);
        self.set_reg(
            R_CLEAR_WIDTH_HEIGHT,
            width as u32 / 4 | (height as u32) << 16,
        );
        self.set_reg(R_CLEAR_DATA, value);
        self.set_reg(R_CONTROL, B_CONTROL_CLEAR);
        while self.get_reg(R_STATUS) & B_STATUS_CLEAR_BUSY != 0 {}
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
struct Triangle {
    edge_vec: [[i32; 3]; 3],
    uv: [[i32; 2]; 3],
    min_x: u16,
    min_y: u16,
    max_x: u16,
    max_y: u16,
}

impl Triangle {
    fn new(c: &CoarseRasterIn) -> Self {
        let mut edge_vec = [[0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                edge_vec[i][j] = (c.edge_mat[j][i] * (1 << 20) as f64) as i32;
            }
        }
        let uv = c.uv.map(|row| row.map(|n| (n * (1 << 26) as f64) as i32));
        Triangle {
            edge_vec,
            uv,
            min_x: c.bbox.min_x as u16,
            min_y: c.bbox.min_y as u16,
            max_x: c.bbox.max_x as u16,
            max_y: c.bbox.max_y as u16,
        }
    }
}

fn main() {
    let mut hw = Hw::new().unwrap();

    let (mut render_fb, mut display_fb) = (FRAMEBUFFER1, FRAMEBUFFER2);

    let mut t = 0.0;

    File::open("texture.raw")
        .unwrap()
        .read_exact(hw.mem_mut(TEXTUREBUFFER, 512 * 512 * 4))
        .unwrap();

    hw.set_reg(R_TEXTURE_ADDR, TEXTUREBUFFER);
    hw.set_reg(R_TEXTURE_EN, 1);

    loop {
        let matrix = matmul(&[
            projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0),
            translate(0.0, -1.0, 3.0),
            rotate(-20.0, [1.0, 0.0, 0.0]),
            rotate(30.0 * t, [0.0, 1.0, 0.0]),
        ]);
        let primitives: Vec<_> = include!("../../model/src/model.rs")
            .iter()
            .map(|v| BarePrimitive::new(*v))
            .collect();

        let mut len = 0;
        for p in primitives
            .iter()
            .map(|p| p.transform(matrix))
            .flat_map(|p| CoarseRasterIn::new(&p))
        {
            let t = Triangle::new(&p);
            hw.write(MEM_START + 2 * 1048576 + 68 * len, t);
            len += 1;
        }

        hw.clear(render_fb, 640, 640, 480, 0x66666666u32);
        hw.clear(DEPTHBUFFER, 2048, 2048, 256, 0);
        hw.set_reg(R_CONTROL, 4);

        hw.set_reg(R_RENDER_TARGET, render_fb);
        hw.set_reg(R_CONTROL, 1 | len << 16);
        hw.set_reg(R_CONTROL, 2);
        while hw.get_reg(R_STATUS) & 4 == 0 {}
        hw.set_reg(R_STATUS, 4);

        while hw.get_reg(R_STATUS) & 1 == 0 || hw.get_reg(R_STATUS) & 2 == 0 {}
        hw.set_reg(R_STATUS, 1);
        std::mem::swap(&mut render_fb, &mut display_fb);
        hw.set_reg(R_DISPLAY_FB, display_fb);

        t += 0.01;
    }
}
