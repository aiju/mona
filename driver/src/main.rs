#![allow(dead_code)]

use crate::{debug::*, hw::*};
use clap::Parser;
use evdev::EvdevSource;
use rs_common::{
    assets::AssetLoader, input::{InputSource, InputState}, render::{Backend, Context}, *
};
use std::time::Instant;

pub mod debug;
pub mod evdev;
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
    depth_buffer: u32,
    cmd_ptr: u32,
    cmd_len: u32,
    stats_collection: StatsCollection,
    text_display: TextDisplay,
    vram_alloc: VramAlloc,
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
        hw.set_reg(R_TEXTURE_EN, 0);
        hw.set_reg(R_STATS_ENABLED, 0);
        let mut vram_alloc = VramAlloc::new(MEM_START as usize, MEM_LEN as usize);
        // FIXME: don't statically allocate command buffer
        vram_alloc.reserve(0x10200000, 1024 * 1024);
        let render_fb = vram_alloc.alloc(WIDTH * HEIGHT * 4).unwrap();
        let display_fb = vram_alloc.alloc(WIDTH * HEIGHT * 4).unwrap();
        // FIXME: don't reserve so much depth buffer
        let depth_buffer = vram_alloc.alloc(2048 * 256 * 4).unwrap();
        hw.set_reg(R_DEPTH_BUFFER, depth_buffer);
        let stats_collection = StatsCollection::new();
        let mut text_display = TextDisplay::new();
        text_display.init(&mut hw);
        HwBackend {
            hw,
            cli,
            render_fb,
            display_fb,
            depth_buffer,
            cmd_ptr: 0,
            cmd_len: 0,
            stats_collection,
            text_display,
            vram_alloc,
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

        self.hw.clear(self.render_fb, 640, 640, 480, 0x66666666u32);
        self.hw.clear(self.depth_buffer, 2048, 2048, 256, 0);
        self.hw.set_reg(R_CONTROL, B_CONTROL_INVALIDATE_DEPTH);

        self.hw.set_reg(R_RENDER_TARGET, self.render_fb);
        if self.cli.show_stats {
            self.stats_collection.start(&mut self.hw);
        }
    }
    fn render_frame(&mut self) {
        let mut hw = &mut self.hw;

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
                    "FPS:     {:5.1}\n",
                    1.0 / (render_done - self.frame_start).as_secs_f64(),
                ),
            );
        }
        if self.cli.show_stats {
            self.stats_collection.print(&mut hw);
        }
    }
}

fn translate_texture_type(ty: &render::TextureType) -> Option<u32> {
    let f = |x: usize| {
        if x & (x - 1) != 0 || x < 8 || x > 1024 {
            None
        } else {
            Some(x.trailing_zeros() - 3)
        }
    };
    Some(1 | f(ty.width)? << 4 | f(ty.height)? << 8 | f(ty.stride)? << 12)
}

impl Backend for HwBackend {
    type Texture = (u32, u32);
    type Error = ();

    fn load_texture(&mut self, texture: render::Texture) -> Result<Self::Texture, Self::Error> {
        if self.cli.textures_off {
            Ok((0, 0))
        } else {
            let en = translate_texture_type(&texture.ty).unwrap();
            let size = texture.ty.stride * texture.ty.height * 4;
            let vram = self.vram_alloc.alloc(size).unwrap();
            self.mem_mut(vram, size as u32)
                .copy_from_slice(&texture.data);
            Ok((vram, en))
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
        self.cmd_ptr = 0x10200000;
        self.cmd_len = 0;
        for t in triangles {
            self.hw.write(self.cmd_ptr, HwTriangle::new(t));
            self.cmd_ptr += size_of::<HwTriangle>() as u32;
            self.cmd_len += 1;
        }
        self.hw
            .set_reg(R_CONTROL, B_CONTROL_START | self.cmd_len << 16);
        self.hw.flush_pipeline();
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

    let mut scene = scene::create(&scene, &mut context, &mut AssetLoader::default())
        .expect(&format!("unknown scene {}", &scene));

    let mut input_source = EvdevSource::new();
    let mut input_state = InputState::default();

    loop {
        while let Some(event) = input_source.poll_event() {
            input_state.update(event.clone());
            scene.input(event);
        }
        context.backend_mut().start_frame();
        scene.render(&mut context);
        context.backend_mut().render_frame();
        scene.update(0.01, &input_state);
    }
}
