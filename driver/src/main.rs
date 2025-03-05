#![allow(dead_code)]

use crate::{debug::*, hw::*};
use clap::Parser;
use rs_common::{scene::*, *};
use std::{fs::File, io::Read, time::Instant};

pub mod debug;
pub mod hw;

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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    textures_off: bool,
    #[arg(long)]
    show_fps: bool,
    #[arg(long)]
    show_stats: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut hw = Hw::new().unwrap();

    let (mut render_fb, mut display_fb) = (FRAMEBUFFER1, FRAMEBUFFER2);

    if !cli.textures_off {
        File::open("texture.raw")
            .unwrap()
            .read_exact(hw.mem_mut(TEXTUREBUFFER, 512 * 512 * 4))
            .unwrap();

        hw.set_reg(R_TEXTURE_ADDR, TEXTUREBUFFER);
    }
    hw.set_reg(R_TEXTURE_EN, !cli.textures_off as u32);

    hw.set_reg(R_STATS_ENABLED, 0);

    let mut stats_collection = StatsCollection::new();
    let mut text_display = TextDisplay::new();
    text_display.init(&mut hw);

    let mut scene = CatRoom::default();

    loop {
        if cli.show_stats {
            stats_collection.reset(&mut hw);
        }

        let frame_start = Instant::now();

        let mut len = 0;
        for p in scene.prep().flat_map(|p| CoarseRasterIn::new(&p)) {
            let t = Triangle::new(&p);
            hw.write(MEM_START + 2 * 1048576 + 68 * len, t);
            len += 1;
        }

        let prep_done = Instant::now();

        hw.clear(render_fb, 640, 640, 480, 0x66666666u32);
        hw.clear(DEPTHBUFFER, 2048, 2048, 256, 0);
        hw.set_reg(R_CONTROL, B_CONTROL_INVALIDATE_DEPTH);

        let clear_done = Instant::now();

        hw.set_reg(R_RENDER_TARGET, render_fb);
        if cli.show_stats {
            stats_collection.start(&mut hw);
        }
        hw.set_reg(R_CONTROL, B_CONTROL_START | len << 16);
        hw.flush_pipeline();
        if cli.show_stats {
            stats_collection.stop(&mut hw);
        }

        let render_done = Instant::now();

        hw.wait_for_vsync();
        std::mem::swap(&mut render_fb, &mut display_fb);
        hw.set_reg(R_DISPLAY_FB, display_fb);

        text_display.clear_all(&mut hw);
        if cli.show_fps {
            text_display.print(
                &mut hw,
                1,
                1,
                &format!(
                    "FPS:     {:5.1}\nPrep:   {:5.1} ms\nClear:  {:5.1} ms\nRender: {:5.1} ms",
                    1.0 / (render_done - frame_start).as_secs_f64(),
                    (prep_done - frame_start).as_secs_f64() * 1000.0,
                    (clear_done - prep_done).as_secs_f64() * 1000.0,
                    (render_done - clear_done).as_secs_f64() * 1000.0
                ),
            );
        }
        if cli.show_stats {
            stats_collection.print(&mut hw);
        }

        scene.update(0.01);
    }
}
