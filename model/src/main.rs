#![allow(dead_code)]

use clap::Parser;
use rs_common::{
    assets::AssetLoader,
    render::{Backend, Context, Texture},
    *,
};
use std::sync::Arc;

#[derive(Default, Debug)]
struct Stats {
    primitives: u64,
    coarse_tiles: u64,
    fine_tiles: u64,
    inside_pixels: u64,
    depth_pass_pixels: u64,
}

#[derive(Debug, Clone)]
struct Tile {
    pos: [usize; 2],
    edge_vec: [f64; 3],
}

fn coarse_raster(stats: &mut Stats, p: &CoarseRasterIn) -> Vec<Tile> {
    stats.primitives += 1;
    let mut tiles = Vec::new();
    let mut e_left = [0, 1, 2].map(|i| p.edge_mat[2][i]);
    for y in p.bbox.min_y..=p.bbox.max_y {
        let mut e = e_left;
        for x in p.bbox.min_x..=p.bbox.max_x {
            stats.coarse_tiles += 1;
            let mut c = [[0.0; 3]; 4];
            for i in 0..3 {
                c[0][i] = e[i];
                c[1][i] = e[i] + p.edge_mat[0][i] * (TILE_SIZE - 1) as f64;
                c[2][i] = e[i] + p.edge_mat[1][i] * (TILE_SIZE - 1) as f64;
                c[3][i] = e[i] + (p.edge_mat[0][i] + p.edge_mat[1][i]) * (TILE_SIZE - 1) as f64;
            }
            let ok = (0..3).all(|j| (0..4).any(|i| c[i][j] >= 0.0));
            if ok {
                stats.fine_tiles += 1;
                tiles.push(Tile {
                    pos: [x, y],
                    edge_vec: e,
                });
            }
            for i in 0..3 {
                e[i] += p.edge_mat[0][i] * TILE_SIZE as f64;
            }
        }
        for i in 0..3 {
            e_left[i] += p.edge_mat[1][i] * TILE_SIZE as f64;
        }
    }
    tiles
}

fn fine_raster(
    stats: &mut Stats,
    p: &CoarseRasterIn,
    tile: &Tile,
    buffer: &mut [u32],
    depth: &mut [f64],
    texture: Option<&Texture>,
) {
    for oy in 0..TILE_SIZE {
        for ox in 0..TILE_SIZE {
            let y = tile.pos[1] * TILE_SIZE + oy;
            let x = tile.pos[0] * TILE_SIZE + ox;
            let e = [0, 1, 2].map(|i| {
                tile.edge_vec[i] + p.edge_mat[0][i] * ox as f64 + p.edge_mat[1][i] * oy as f64
            });
            let inside = e.iter().all(|&p| p >= 0.0);
            if !inside {
                continue;
            }
            stats.inside_pixels += 1;
            let wr = e[0] + e[1] + e[2];
            let depth_ptr = &mut depth[y * WIDTH + x];
            if wr <= *depth_ptr {
                continue;
            }
            *depth_ptr = wr;
            stats.depth_pass_pixels += 1;
            let u = (0..3).map(|i| p.uv[i][0] * e[i]).sum::<f64>() / wr;
            let v = (0..3).map(|i| p.uv[i][1] * e[i]).sum::<f64>() / wr;
            let rgb = if let Some(texture) = texture {
                let tx = ((u * (texture.ty.width as f64)) as usize).clamp(0, texture.ty.width - 1);
                let ty =
                    ((v * (texture.ty.height as f64)) as usize).clamp(0, texture.ty.height - 1);
                let addr = (tx + ty * texture.ty.stride) * 4;
                let d = &texture.data[addr..addr + 4];
                [d[0], d[1], d[2]]
            } else {
                [255, 255, 255]
            };
            buffer[y * WIDTH + x] = rgb[2] as u32 | (rgb[1] as u32) << 8 | (rgb[0] as u32) << 16;
        }
    }
}

struct ModelBackend {
    texture: Option<Arc<Texture<'static>>>,
    frame: Vec<u32>,
    depth: Vec<f64>,
    stats: Stats,
}

impl ModelBackend {
    fn new() -> Self {
        ModelBackend {
            texture: None,
            frame: vec![0; WIDTH * HEIGHT],
            depth: vec![-f64::INFINITY; WIDTH * HEIGHT],
            stats: Stats::default(),
        }
    }
    fn start_frame(&mut self) {
        self.frame.fill(0);
        self.depth.fill(-f64::INFINITY);
        self.stats = Stats::default();
    }
}

impl Backend for ModelBackend {
    type Texture = Arc<Texture<'static>>;
    type Error = ();

    fn load_texture(&mut self, texture: Texture<'_>) -> Result<Self::Texture, Self::Error> {
        Ok(Arc::new(Texture {
            data: texture.data.into_owned().into(),
            ty: texture.ty,
        }))
    }

    fn use_texture(&mut self, texture: Option<&Self::Texture>) {
        self.texture = texture.cloned();
    }

    fn draw(&mut self, triangles: &[CoarseRasterIn]) {
        let ctx = &mut self.stats;
        for p in triangles {
            for tile in coarse_raster(ctx, p) {
                fine_raster(
                    ctx,
                    &p,
                    &tile,
                    &mut self.frame,
                    &mut self.depth,
                    self.texture.as_deref(),
                );
            }
        }
        /*if show_bbox {
            for p in &data {
                for x in p.bbox.min_x * TILE_SIZE..(p.bbox.max_x + 1) * TILE_SIZE {
                    buffer[p.bbox.min_y * TILE_SIZE * WIDTH + x] = 0xffffff;
                    buffer[((p.bbox.max_y + 1) * TILE_SIZE - 1) * WIDTH + x] = 0xffffff;
                }
                for y in p.bbox.min_y * TILE_SIZE..(p.bbox.max_y + 1) * TILE_SIZE {
                    buffer[p.bbox.min_x * TILE_SIZE + y * WIDTH] = 0xffffff;
                    buffer[((p.bbox.max_x + 1) * TILE_SIZE - 1) + y * WIDTH] = 0xffffff;
                }
            }
        }*/
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    show_stats: bool,
    #[arg(long)]
    show_bbox: bool,
    #[arg(long, default_value = "CatRoom")]
    scene: String,
}

use minifb::{Key, Window, WindowOptions};

fn main() {
    let cli = Cli::parse();

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);

    let mut context = Context::new(ModelBackend::new());
    let mut scene = scene::create(&cli.scene, &mut context, &mut AssetLoader::default())
        .expect(&format!("unknown scene {}", &cli.scene));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        context.backend_mut().start_frame();
        scene.render(&mut context);
        if cli.show_stats {
            println!("{:?}", context.backend().stats);
        }
        window
            .update_with_buffer(&context.backend().frame, WIDTH, HEIGHT)
            .unwrap();
        scene.update(10.0 / 60.0, &Default::default());
    }
}
