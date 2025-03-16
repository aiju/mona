use super::*;
use crate::{
    animation::{Sampler, SamplerMode},
    geometry::{Vec3, Vec4},
};
use std::rc::Rc;

#[derive(Clone)]
pub struct Animation {
    name: Option<String>,
    channels: Vec<(usize, Rc<Node>, json::AnimationPath)>,
    data: Vec<AnimationData>,
}

#[derive(Clone, Debug)]
pub struct AnimationData {
    input: Vec<f64>,
    interpolation: json::AnimationInterpolation,
    output: AnimationOutput,
}

#[derive(Clone, Debug)]
pub enum AnimationOutput {
    Scalar(Vec<f64>),
    Vec3(Vec<Vec3>),
    Vec4(Vec<Vec4>),
}

#[derive(Clone)]
pub struct Skin {
    pub inverse_bind_matrices: Vec<Matrix>,
    pub skeleton: Option<Rc<Node>>,
    pub joints: Vec<Rc<Node>>,
    pub name: Option<String>,
}

impl<'a> super::GltfImporter<'a> {
    fn animation_output_accessor(&self, id: json::AccessorId) -> Result<AnimationOutput, Error> {
        let accessor = self.json.accessor(id)?;
        Ok(match accessor.type_ {
            json::AccessorType::SCALAR => AnimationOutput::Scalar(self.accessor(id)?),
            json::AccessorType::VEC3 => AnimationOutput::Vec3(self.accessor(id)?),
            json::AccessorType::VEC4 => AnimationOutput::Vec4(self.accessor(id)?),
            _ => gltf_abort!(),
        })
    }
    fn validate_animation_channel(
        &self,
        sampler: usize,
        data: &[AnimationData],
        path: json::AnimationPath,
    ) -> Result<(), Error> {
        let data = gltf_unwrap!(data.get(sampler));
        Ok(match (path, &data.output) {
            (json::AnimationPath::Translation, AnimationOutput::Vec3(_)) => {}
            (json::AnimationPath::Rotation, AnimationOutput::Vec4(_)) => {}
            (json::AnimationPath::Scale, AnimationOutput::Vec3(_)) => {}
            (json::AnimationPath::Weights, AnimationOutput::Scalar(_)) => {}
            _ => gltf_abort!(),
        })
    }
    pub fn animation(&self, id: json::AnimationId) -> Result<Rc<Animation>, Error> {
        self.animations.get_or_insert(id, || {
            let animation = self.json.animation(id)?;
            let data = animation
                .samplers
                .iter()
                .map(|s| {
                    let input = self.accessor(s.input)?;
                    let output = self.animation_output_accessor(s.output)?;
                    Ok(AnimationData {
                        input,
                        interpolation: s.interpolation,
                        output,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let channels = animation
                .channels
                .iter()
                .map(|c| {
                    let node = self.node(gltf_unwrap!(c.target.node))?;
                    node.used_by_animation.set(true);
                    self.validate_animation_channel(c.sampler, &data, c.target.path)?;
                    Ok((c.sampler, node, c.target.path))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            Ok(Rc::new(Animation {
                name: animation.name.clone(),
                channels,
                data,
            }))
        })
    }
    pub fn skin(&self, id: json::SkinId) -> Result<Rc<Skin>, Error> {
        self.skins.get_or_insert(id, || {
            let skin = self.json.skin(id)?;
            let inverse_bind_matrices = skin
                .inverse_bind_matrices
                .map_or(Ok(vec![]), |id| self.accessor(id))?;
            let skeleton = skin.skeleton.map(|id| self.node(id)).transpose()?;
            let joints = skin
                .joints
                .iter()
                .map(|id| self.node(*id))
                .collect::<Result<Vec<_>, _>>()?;
            joints.iter().for_each(|joint| joint.used_by_animation.set(true));
            Ok(Rc::new(Skin {
                inverse_bind_matrices,
                skeleton,
                joints,
                name: skin.name.clone(),
            }))
        })
    }
}

impl TryInto<Vec<f64>> for AnimationOutput {
    type Error = Error;
    fn try_into(self) -> Result<Vec<f64>, Self::Error> {
        match self {
            AnimationOutput::Scalar(vec) => Ok(vec),
            _ => gltf_abort!(),
        }
    }
}

impl TryInto<Vec<Vec3>> for AnimationOutput {
    type Error = Error;
    fn try_into(self) -> Result<Vec<Vec3>, Self::Error> {
        match self {
            AnimationOutput::Vec3(vec) => Ok(vec),
            _ => gltf_abort!(),
        }
    }
}

impl TryInto<Vec<Vec4>> for AnimationOutput {
    type Error = Error;
    fn try_into(self) -> Result<Vec<Vec4>, Self::Error> {
        match self {
            AnimationOutput::Vec4(vec) => Ok(vec),
            _ => gltf_abort!(),
        }
    }
}

impl AnimationData {
    pub fn to_sampler<T>(&self) -> Sampler<T>
    where
        AnimationOutput: TryInto<Vec<T>>,
    {
        let mode = match self.interpolation {
            json::AnimationInterpolation::LINEAR => SamplerMode::Linear,
            json::AnimationInterpolation::STEP => SamplerMode::Step,
            json::AnimationInterpolation::CUBICSPLINE => todo!(),
        };
        Sampler {
            mode,
            keyframes: self.input.clone(),
            samples: self
                .output
                .clone()
                .try_into()
                .unwrap_or_else(|_| unreachable!()),
            time: 0.0,
            index: 0,
        }
    }
}

impl Animation {
    pub fn to_game_object(&self) -> mesh::GameObject {
        let mut update_fns: Vec<Box<dyn FnMut(f64)>> = Vec::new();
        for (sampler_idx, node, path) in &self.channels {
            let data = &self.data[*sampler_idx];
            let go = node.game_object.borrow().as_ref().unwrap().clone();
            match path {
                json::AnimationPath::Translation => {
                    let mut sampler = data.to_sampler();
                    update_fns.push(Box::new(move |delay| {
                        *go.position.borrow_mut() = sampler.sample();
                        sampler.advance(delay);
                    }));
                }
                json::AnimationPath::Rotation => {
                    let mut sampler = data.to_sampler();
                    update_fns.push(Box::new(move |delay| {
                        *go.rotation.borrow_mut() = sampler.sample();
                        sampler.advance(delay);
                    }));
                }
                json::AnimationPath::Scale => {
                    let mut sampler = data.to_sampler();
                    update_fns.push(Box::new(move |delay| {
                        *go.scale.borrow_mut() = sampler.sample();
                        sampler.advance(delay);
                    }));
                }
                _ => {
                    eprintln!("unsupported animation: {path:?}");
                }
            }
        }
        GameObject {
            mesh: None,
            name: None,
            position: Vec3::from([0.0, 0.0, 0.0]).into(),
            rotation: Vec4::from([0.0, 0.0, 0.0, 1.0]).into(),
            scale: Vec3::from([1.0, 1.0, 1.0]).into(),
            children: Default::default(),
            update_fn: RefCell::new(Some(Box::new(move |_, delta| {
                update_fns.iter_mut().for_each(|f| f(delta));
            }))),
        }
    }
}
