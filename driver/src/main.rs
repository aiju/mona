#![allow(dead_code)]

use crate::{debug::*, hw::*};
use clap::Parser;
use rs_common::{
    assets::AssetLoader,
    render::{Backend, Context, Texture, TextureType},
    *,
};
use std::{fs::File, io::Read, time::Instant};

pub mod debug;
pub mod hw;

#[repr(C)]
#[derive(Debug, Clone)]
struct HwTriangle {
    edge_vec: [[i32; 3]; 3],
    uv: [[i32; 2]; 3],
    rgb: [u32; 3],
    min_x: u16,
    min_y: u16,
    max_x: u16,
    max_y: u16,
}

impl HwTriangle {
    fn new(c: &CoarseRasterIn) -> Self {
        let mut edge_vec = [[0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                edge_vec[i][j] = (c.edge_mat[j][i] * (1 << 20) as f64) as i32;
            }
        }
        let uv =
            c.uv.map(|row| row.map(|n| (n * (1 << 11) as f64) as i32 + (1 << 17)));
        HwTriangle {
            edge_vec,
            uv,
            rgb: c.rgb,
            min_x: c.bbox.min_x as u16,
            min_y: c.bbox.min_y as u16,
            max_x: c.bbox.max_x as u16,
            max_y: c.bbox.max_y as u16,
        }
    }
}

struct HwBackend {
    hw: Hw,
    cli: Cli,
    render_fb: u32,
    display_fb: u32,
    cmd_ptr: u32,
    cmd_len: u32,
    stats_collection: StatsCollection,
    text_display: TextDisplay,
    frame_start: Instant,
}

impl std::ops::Deref for HwBackend {
    type Target = Hw;
    fn deref(&self) -> &Self::Target {
        &self.hw
    }
}
impl std::ops::DerefMut for HwBackend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hw
    }
}

impl HwBackend {
    fn new(mut hw: Hw, cli: Cli) -> HwBackend {
        hw.set_reg(R_DEPTH_MODE, if cli.disable_depth_buffer { 0 } else { 4 });
        hw.set_reg(R_STATS_ENABLED, 0);
        let stats_collection = StatsCollection::new();
        let mut text_display = TextDisplay::new();
        text_display.init(&mut hw);
        HwBackend {
            hw,
            cli,
            render_fb: FRAMEBUFFER1,
            display_fb: FRAMEBUFFER2,
            cmd_ptr: 0,
            cmd_len: 0,
            stats_collection,
            text_display,
            frame_start: Instant::now(),
        }
    }
    fn start_frame(&mut self) {
        if self.cli.show_stats {
            self.stats_collection.reset(&mut self.hw);
        }
        self.cmd_ptr = 0x10200000;
        self.cmd_len = 0;
        self.frame_start = Instant::now();
    }
    fn render_frame(&mut self) {
        let mut hw = &mut self.hw;
        let prep_done = Instant::now();

        hw.clear(self.render_fb, 640, 640, 480, 0x66666666u32);
        hw.clear(DEPTHBUFFER, 2048, 2048, 256, 0);
        hw.set_reg(R_CONTROL, B_CONTROL_INVALIDATE_DEPTH);

        let clear_done = Instant::now();

        hw.set_reg(R_RENDER_TARGET, self.render_fb);
        if self.cli.show_stats {
            self.stats_collection.start(&mut hw);
        }
        hw.set_reg(R_CONTROL, B_CONTROL_START | self.cmd_len << 16);
        hw.flush_pipeline();
        if self.cli.show_stats {
            self.stats_collection.stop(&mut hw);
        }

        let render_done = Instant::now();

        hw.wait_for_vsync();
        std::mem::swap(&mut self.render_fb, &mut self.display_fb);
        hw.set_reg(R_DISPLAY_FB, self.display_fb);

        self.text_display.clear_all(&mut hw);
        if self.cli.show_fps {
            self.text_display.print(
                &mut hw,
                1,
                1,
                &format!(
                    "FPS:     {:5.1}\nPrep:   {:5.1} ms\nClear:  {:5.1} ms\nRender: {:5.1} ms",
                    1.0 / (render_done - self.frame_start).as_secs_f64(),
                    (prep_done - self.frame_start).as_secs_f64() * 1000.0,
                    (clear_done - prep_done).as_secs_f64() * 1000.0,
                    (render_done - clear_done).as_secs_f64() * 1000.0
                ),
            );
        }
        if self.cli.show_stats {
            self.stats_collection.print(&mut hw);
        }
    }
}

impl Backend for HwBackend {
    type Texture = (u32, u32);
    type Error = ();

    fn load_texture(&mut self, texture: render::Texture) -> Result<Self::Texture, Self::Error> {
        if self.cli.textures_off {
            Ok((0, 0))
        } else {
            self.mem_mut(TEXTUREBUFFER, 512 * 512 * 4)
                .copy_from_slice(&texture.data);
            Ok((TEXTUREBUFFER, 0x6661))
        }
    }

    fn use_texture(&mut self, texture: Option<&Self::Texture>) {
        if let Some(&(addr, en)) = texture {
            self.hw.set_reg(R_TEXTURE_ADDR, addr);
            self.hw.set_reg(R_TEXTURE_EN, en);
        } else {
            self.hw.set_reg(R_TEXTURE_EN, 0);
        }
    }

    fn draw(&mut self, triangles: &[CoarseRasterIn]) {
        for t in triangles {
            self.hw.write(self.cmd_ptr, HwTriangle::new(t));
            self.cmd_ptr += size_of::<HwTriangle>() as u32;
            self.cmd_len += 1;
        }
    }
}

#[derive(Default)]
struct DriverAssetLoader {}
impl AssetLoader for DriverAssetLoader {
    type Error = ();
    fn load_texture(&mut self, name: &str) -> Result<render::Texture, Self::Error> {
        let mut data = Vec::with_capacity(512 * 512 * 4);
        File::open("texture.raw")
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        Ok(Texture {
            data: data.into(),
            ty: TextureType {
                width: 512,
                height: 512,
                stride: 512,
            },
        })
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
    #[arg(long)]
    disable_depth_buffer: bool,
    #[arg(long, default_value = "CatRoom")]
    scene: String,
}

fn main() {
    let mut cli = Cli::parse();
    let scene = std::mem::take(&mut cli.scene);
    let mut context = Context::new(HwBackend::new(Hw::new().unwrap(), cli));

    let mut scene = scene::create(&scene, &mut context, DriverAssetLoader::default())
        .expect(&format!("unknown scene {}", &scene));

    loop {
        context.backend_mut().start_frame();
        scene.render(&mut context);
        context.backend_mut().render_frame();
        scene.update(0.01);
    }
}
