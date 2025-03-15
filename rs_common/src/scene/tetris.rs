use rand::{Rng, rng, rngs::ThreadRng};

use crate::{
    BarePrimitive, HEIGHT, WIDTH,
    assets::AssetLoader,
    geometry::Matrix,
    input::InputState,
    mesh::Color,
    render::{Backend, Context},
    scene::{CUBE, Scene},
};

type Block = u8;
const FALL_TIME: f64 = 0.25;

const COLORS: [Color; 8] = [
    Color { r: 255, g: 0, b: 0 },
    Color { r: 0, g: 255, b: 0 },
    Color { r: 0, g: 0, b: 255 },
    Color {
        r: 255,
        g: 255,
        b: 0,
    },
    Color {
        r: 0,
        g: 255,
        b: 255,
    },
    Color {
        r: 255,
        g: 0,
        b: 255,
    },
    Color {
        r: 255,
        g: 128,
        b: 128,
    },
    Color {
        r: 128,
        g: 255,
        b: 128,
    },
];

const PIECES: [[[u8; 2]; 4]; 7] = [
    [[0, 0], [1, 0], [2, 0], [3, 0]],
    [[0, 0], [1, 0], [2, 0], [1, 1]],
    [[0, 0], [1, 0], [0, 1], [1, 1]],
    [[0, 0], [1, 0], [1, 1], [2, 1]],
    [[0, 1], [1, 1], [1, 0], [2, 0]],
    [[0, 0], [1, 0], [2, 0], [2, 1]],
    [[0, 1], [1, 1], [2, 1], [2, 0]],
];

struct Field {
    width: usize,
    height: usize,
    blocks: Vec<Block>,
}

impl Field {
    fn empty(width: usize, height: usize) -> Self {
        Field {
            width,
            height,
            blocks: vec![0; width * height],
        }
    }
}

pub struct Tetris {
    field: Field,
    active_piece: u8,
    timer: f64,
    piece_x: usize,
    piece_y: usize,
    piece_moving: bool,
    rng: ThreadRng,
}

impl Tetris {
    pub fn new<B: Backend>(_context: &mut Context<B>, _loader: impl AssetLoader) -> Self {
        Tetris {
            field: Field::empty(8, 13),
            active_piece: 0,
            timer: 0.0,
            piece_x: 0,
            piece_y: 0,
            piece_moving: true,
            rng: rng(),
        }
    }
    fn draw_cube<B: Backend>(
        &mut self,
        context: &mut crate::render::Context<B>,
        view: Matrix,
        x: f64,
        y: f64,
        col: u8,
    ) {
        let object = Matrix::rotate(
            (x - (self.field.width as f64 - 1.0) / 2.0) * 15.0,
            [0.0, 1.0, 0.0],
        ) * Matrix::translate(0.0, y - (self.field.height as f64 - 1.0) / 2.0, 5.0)
            * Matrix::scale(0.4, 0.4, 0.4);
        let tris = CUBE
            .iter()
            .map(|x| {
                let mut t = BarePrimitive::new(*x).transform(object);
                t.color = [COLORS[col as usize]; 3];
                t.lighting(0.5, 0.5, [0.0, 0.0, 1.0].into()).transform(view)
            })
            .collect::<Vec<_>>();
        context.draw().run(&tris);
    }
    fn intersection_check(&mut self, px: usize, py: usize) -> bool {
        for [dx, dy] in PIECES[self.active_piece as usize] {
            let x = (dx as usize) + px;
            let y = (dy as usize) + py;
            if y >= self.field.height || self.field.blocks[y * self.field.width + x] != 0 {
                return true;
            }
        }
        return false;
    }
    fn set_piece(&mut self) {
        for [dx, dy] in PIECES[self.active_piece as usize] {
            let x = (dx as usize) + self.piece_x;
            let y = (dy as usize) + self.piece_y;
            self.field.blocks[y * self.field.width + x] = self.active_piece + 1;
        }
    }
}

impl<B: Backend> Scene<B> for Tetris {
    fn render(&mut self, context: &mut crate::render::Context<B>) {
        let view = Matrix::projection(90.0, WIDTH as f64, HEIGHT as f64, 0.1, 100.0)
            * Matrix::translate(0.0, -0.0, 5.0);
        for y in 0..self.field.height {
            for x in 0..self.field.width {
                let col = self.field.blocks[y * self.field.width + x];
                if col != 0 {
                    self.draw_cube(context, view, x as f64, y as f64, col);
                }
            }
        }
        for [x, y] in PIECES[self.active_piece as usize] {
            self.draw_cube(
                context,
                view,
                (self.piece_x + x as usize) as f64,
                (self.piece_y + y as usize) as f64
                    + if self.piece_moving {
                        self.timer / FALL_TIME
                    } else {
                        0.0
                    },
                self.active_piece + 1,
            );
        }
    }
    fn update(&mut self, delta: f64, _input: &InputState) {
        self.timer += delta;
        if self.timer >= FALL_TIME {
            if self.intersection_check(self.piece_x, self.piece_y + 1) {
                self.set_piece();
                self.active_piece = self.rng.random_range(0..PIECES.len() as u8);
                self.piece_x = 0;
                self.piece_y = 0;
                self.piece_moving = !self.intersection_check(self.piece_x, self.piece_y + 1);
                if self.intersection_check(0, 0) {
                    self.field.blocks.fill(0);
                    self.piece_moving = true;
                }
            } else {
                self.piece_y += 1;
                self.piece_moving = !self.intersection_check(self.piece_x, self.piece_y + 1);
            }
            self.timer -= FALL_TIME;
        }
    }
}
