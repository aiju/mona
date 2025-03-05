#![allow(dead_code)]

use clap::Parser;
use rs_common::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const TILE_SIZE: usize = 4;

#[derive(Default, Debug)]
struct Context {
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

fn coarse_raster(ctx: &mut Context, p: &CoarseRasterIn) -> Vec<Tile> {
    ctx.primitives += 1;
    let mut tiles = Vec::new();
    let mut e_left = [0, 1, 2].map(|i| p.edge_mat[2][i]);
    for y in p.bbox.min_y..=p.bbox.max_y {
        let mut e = e_left;
        for x in p.bbox.min_x..=p.bbox.max_x {
            ctx.coarse_tiles += 1;
            let mut c = [[0.0; 3]; 4];
            for i in 0..3 {
                c[0][i] = e[i];
                c[1][i] = e[i] + p.edge_mat[0][i] * (TILE_SIZE - 1) as f64;
                c[2][i] = e[i] + p.edge_mat[1][i] * (TILE_SIZE - 1) as f64;
                c[3][i] = e[i] + (p.edge_mat[0][i] + p.edge_mat[1][i]) * (TILE_SIZE - 1) as f64;
            }
            let ok = (0..3).all(|j| (0..4).any(|i| c[i][j] >= 0.0));
            if ok {
                ctx.fine_tiles += 1;
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
    ctx: &mut Context,
    p: &CoarseRasterIn,
    tile: &Tile,
    buffer: &mut [u32],
    depth: &mut [f64],
    texture: &RgbImage,
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
            ctx.inside_pixels += 1;
            let wr = e[0] + e[1] + e[2];
            let depth_ptr = &mut depth[y * WIDTH + x];
            if wr <= *depth_ptr {
                continue;
            }
            *depth_ptr = wr;
            ctx.depth_pass_pixels += 1;
            let u = (0..3).map(|i| p.uv[i][0] * e[i]).sum::<f64>() / wr;
            let v = (0..3).map(|i| p.uv[i][1] * e[i]).sum::<f64>() / wr;
            let tx = ((u * (texture.width() as f64)) as u32).clamp(0, texture.width() - 1);
            let ty = ((v * (texture.height() as f64)) as u32).clamp(0, texture.height() - 1);
            let rgb = texture.get_pixel(tx, ty);
            buffer[y * WIDTH + x] =
                rgb.0[2] as u32 | (rgb.0[1] as u32) << 8 | (rgb.0[0] as u32) << 16;
        }
    }
}

fn render_frame(
    ctx: &mut Context,
    primitives: &[BarePrimitive],
    buffer: &mut [u32],
    texture: &RgbImage,
    show_bbox: bool,
) {
    buffer.fill(0);
    let mut depth = [-f64::INFINITY; WIDTH * HEIGHT];
    let data = primitives
        .iter()
        .flat_map(|p| CoarseRasterIn::new(&p))
        .collect::<Vec<_>>();
    for p in &data {
        for tile in coarse_raster(ctx, p) {
            fine_raster(ctx, &p, &tile, buffer, &mut depth, texture);
        }
    }
    if show_bbox {
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

use image::{ImageReader, RgbImage};
use minifb::{Key, Window, WindowOptions};

fn main() {
    let cli = Cli::parse();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let texture = ImageReader::open("/home/aiju/cat.jpg")
        .unwrap()
        .decode()
        .unwrap()
        .into_rgb8();

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

    let mut scene = scene::create(&cli.scene).expect(&format!("unknown scene {}", &cli.scene));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let primitives = scene.prep();
        let mut ctx = Context::default();
        render_frame(&mut ctx, &primitives, &mut buffer, &texture, cli.show_bbox);
        if cli.show_stats {
            println!("{:?}", ctx);
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        scene.update(10.0 / 60.0);
    }
}
