use super::json;
use crate::geometry::{Matrix, Vec2, Vec3, Vec4};

pub trait InnerAccessor: Sized {
    const COMPONENT_TYPE: json::ComponentType;
    unsafe fn read(buf: &[u8]) -> Self;
}

pub trait Accessor: Sized {
    const COMPONENT_TYPE: json::ComponentType;
    const ACCESSOR_TYPE: json::AccessorType;
    unsafe fn read(buf: &[u8]) -> Self;
}

impl InnerAccessor for u8 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::U8;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe { *buf.as_ptr().cast() }
    }
}

impl InnerAccessor for i8 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::I8;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe { *buf.as_ptr().cast() }
    }
}

impl InnerAccessor for u16 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::U16;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe { *buf.as_ptr().cast() }
    }
}

impl InnerAccessor for i16 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::I16;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe { *buf.as_ptr().cast() }
    }
}

impl InnerAccessor for u32 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::U32;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe { *buf.as_ptr().cast() }
    }
}

impl InnerAccessor for f32 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::F32;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe { *buf.as_ptr().cast() }
    }
}

impl InnerAccessor for f64 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::F32;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe { <f32 as InnerAccessor>::read(buf).into() }
    }
}

impl<T: InnerAccessor> Accessor for T {
    const COMPONENT_TYPE: json::ComponentType = T::COMPONENT_TYPE;
    const ACCESSOR_TYPE: json::AccessorType = json::AccessorType::SCALAR;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe { T::read(buf) }
    }
}

impl<T: InnerAccessor> Accessor for [T; 2] {
    const COMPONENT_TYPE: json::ComponentType = T::COMPONENT_TYPE;
    const ACCESSOR_TYPE: json::AccessorType = json::AccessorType::VEC2;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe {
            let t0 = T::read(buf);
            let t1 = T::read(&buf[T::COMPONENT_TYPE.len()..]);
            [t0, t1]
        }
    }
}

impl<T: InnerAccessor> Accessor for [T; 3] {
    const COMPONENT_TYPE: json::ComponentType = T::COMPONENT_TYPE;
    const ACCESSOR_TYPE: json::AccessorType = json::AccessorType::VEC3;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe {
            let t0 = T::read(buf);
            let t1 = T::read(&buf[T::COMPONENT_TYPE.len()..]);
            let t2 = T::read(&buf[2 * T::COMPONENT_TYPE.len()..]);
            [t0, t1, t2]
        }
    }
}

impl<T: InnerAccessor> Accessor for [T; 4] {
    const COMPONENT_TYPE: json::ComponentType = T::COMPONENT_TYPE;
    const ACCESSOR_TYPE: json::AccessorType = json::AccessorType::VEC4;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe {
            let t0 = T::read(buf);
            let t1 = T::read(&buf[T::COMPONENT_TYPE.len()..]);
            let t2 = T::read(&buf[2 * T::COMPONENT_TYPE.len()..]);
            let t3 = T::read(&buf[3 * T::COMPONENT_TYPE.len()..]);
            [t0, t1, t2, t3]
        }
    }
}

impl Accessor for Vec2 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::F32;
    const ACCESSOR_TYPE: json::AccessorType = json::AccessorType::VEC2;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe {
            let t0 = <f64 as InnerAccessor>::read(buf);
            let t1 =
                <f64 as InnerAccessor>::read(&buf[<f64 as InnerAccessor>::COMPONENT_TYPE.len()..]);
            [t0, t1].into()
        }
    }
}

impl Accessor for Vec3 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::F32;
    const ACCESSOR_TYPE: json::AccessorType = json::AccessorType::VEC3;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe {
            let t0 = <f64 as InnerAccessor>::read(buf);
            let t1 =
                <f64 as InnerAccessor>::read(&buf[<f64 as InnerAccessor>::COMPONENT_TYPE.len()..]);
            let t2 = <f64 as InnerAccessor>::read(
                &buf[2 * <f64 as InnerAccessor>::COMPONENT_TYPE.len()..],
            );
            [t0, t1, t2].into()
        }
    }
}

impl Accessor for Vec4 {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::F32;
    const ACCESSOR_TYPE: json::AccessorType = json::AccessorType::VEC4;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe {
            let t0 = <f64 as InnerAccessor>::read(buf);
            let t1 =
                <f64 as InnerAccessor>::read(&buf[<f64 as InnerAccessor>::COMPONENT_TYPE.len()..]);
            let t2 = <f64 as InnerAccessor>::read(
                &buf[2 * <f64 as InnerAccessor>::COMPONENT_TYPE.len()..],
            );
            let t3 = <f64 as InnerAccessor>::read(
                &buf[3 * <f64 as InnerAccessor>::COMPONENT_TYPE.len()..],
            );
            [t0, t1, t2, t3].into()
        }
    }
}

impl Accessor for Matrix {
    const COMPONENT_TYPE: json::ComponentType = json::ComponentType::F32;
    const ACCESSOR_TYPE: json::AccessorType = json::AccessorType::MAT4;
    unsafe fn read(buf: &[u8]) -> Self {
        unsafe {
            let matrix = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].map(|i| {
                <f64 as InnerAccessor>::read(
                    &buf[<f64 as InnerAccessor>::COMPONENT_TYPE.len() * i..],
                )
            });
            matrix.into()
        }
    }
}
