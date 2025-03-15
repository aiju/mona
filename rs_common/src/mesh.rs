use std::{
    cell::RefCell,
    collections::HashMap,
    io::{BufRead, BufReader, Cursor, Seek},
    path::PathBuf,
    rc::{Rc, Weak},
};

use crate::{
    assets::{AssetLoader, resolve_path},
    geometry::{Vec2, Vec3},
    render::{self, Backend, Context, TextureId},
};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    pub fn as_u32(self) -> u32 {
        (self.r as u32) | (self.g as u32) << 8 | (self.b as u32) << 16
    }
}

impl std::ops::Mul<f64> for Color {
    type Output = Color;
    fn mul(self, rhs: f64) -> Self::Output {
        let r = ((self.r as f64) * rhs) as u8;
        let g = ((self.g as f64) * rhs) as u8;
        let b = ((self.b as f64) * rhs) as u8;
        Color { r, g, b }
    }
}

impl From<[f64; 3]> for Color {
    fn from(value: [f64; 3]) -> Self {
        let f = |v: f64| (v * 255.0) as u8;
        Color {
            r: f(value[0]),
            g: f(value[1]),
            b: f(value[2]),
        }
    }
}

impl From<[f64; 4]> for Color {
    fn from(value: [f64; 4]) -> Self {
        let f = |v: f64| (v * 255.0) as u8;
        Color {
            r: f(value[0]),
            g: f(value[1]),
            b: f(value[2]),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TextureState {
    File(PathBuf),
    Memory(Vec<u8>),
    RenderTexture(render::Texture<'static>),
    Backend(TextureId),
    Error,
}

#[derive(Debug)]
pub struct Texture {
    pub state: RefCell<TextureState>,
}

#[derive(Clone, Debug, Default)]
pub struct Material {
    pub texture: Option<Rc<Texture>>,
}

#[derive(Clone, Debug)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
    pub uv: [Vec2; 3],
    pub color: [Color; 3],
}

#[derive(Default, Clone, Debug)]
pub struct Mesh {
    pub triangles: Vec<Vec<Triangle>>,
    pub materials: Vec<Rc<Material>>,
}

fn image_reader_to_render_texture<R: BufRead + Seek>(
    image_reader: image::ImageReader<R>,
) -> render::Texture<'static> {
    let image = image_reader
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8()
        .into_flat_samples();
    render::Texture {
        data: image.samples.into(),
        ty: render::TextureType {
            width: image.layout.width as usize,
            height: image.layout.height as usize,
            stride: image.layout.height_stride / 4,
        },
    }
}

fn replace_with<T>(dest: &mut T, substitute: T, fun: impl FnOnce(T) -> T) {
    let value = std::mem::replace(dest, substitute);
    *dest = fun(value);
}

// FIXME: should be global, not thread local
thread_local! {
    pub static TEXTURE_BY_NAME: RefCell<HashMap<PathBuf, Weak<Texture>>> = RefCell::new(HashMap::new());
}

impl Texture {
    pub fn from_file(name: &str, parent: Option<&str>) -> Rc<Texture> {
        let path = resolve_path(name, parent);
        TEXTURE_BY_NAME.with_borrow_mut(|texture_by_name| {
            if let Some(t) = texture_by_name.get(&path).and_then(Weak::upgrade) {
                t
            } else {
                let texture = Rc::new(Texture {
                    state: RefCell::new(TextureState::File(path.clone())),
                });
                texture_by_name.insert(path, Rc::downgrade(&texture));
                texture
            }
        })
    }
    pub fn from_vec(vec: Vec<u8>) -> Rc<Texture> {
        Rc::new(Texture {
            state: RefCell::new(TextureState::Memory(vec)),
        })
    }
    pub fn load(&self, loader: &mut AssetLoader) {
        replace_with(
            &mut *self.state.borrow_mut(),
            TextureState::Error,
            |state| match state {
                TextureState::File(path) => {
                    let file = loader.open_file(&path).unwrap();
                    println!("{:?}", path);
                    let image_reader = image::ImageReader::new(BufReader::new(file));
                    TextureState::RenderTexture(image_reader_to_render_texture(image_reader))
                }
                TextureState::Memory(data) => {
                    let image_reader = image::ImageReader::new(Cursor::new(data));
                    TextureState::RenderTexture(image_reader_to_render_texture(image_reader))
                }
                TextureState::RenderTexture(_) | TextureState::Backend(_) | TextureState::Error => {
                    state
                }
            },
        )
    }
    pub fn load_backend<B: Backend>(&self, context: &mut Context<B>, loader: &mut AssetLoader) {
        self.load(loader);
        replace_with(
            &mut *self.state.borrow_mut(),
            TextureState::Error,
            |state| match state {
                TextureState::File(_) | TextureState::Memory(_) => unreachable!(),
                TextureState::RenderTexture(texture) => {
                    TextureState::Backend(context.load_texture(texture).unwrap())
                }
                TextureState::Backend(_) | TextureState::Error => state,
            },
        )
    }
    pub fn texture_id(&self) -> TextureId {
        match *self.state.borrow() {
            TextureState::Backend(texture_id) => texture_id,
            _ => unreachable!(),
        }
    }
}

impl Material {
    pub fn texture_id(&self) -> Option<TextureId> {
        self.texture.as_ref().map(|t| t.texture_id())
    }
}

impl Mesh {
    pub fn load<B: Backend>(&self, context: &mut Context<B>, loader: &mut AssetLoader) {
        for material in &self.materials {
            if let Some(tex) = &material.texture {
                tex.load_backend(context, loader);
            }
        }
    }
}
