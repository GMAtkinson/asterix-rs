/// Several helpers for deku reading of certain types into certain types
use deku::prelude::*;

#[allow(dead_code)]
pub enum Op {
    Multiply,
    Divide,
    Add,
    Subtract,
}

pub mod read {
    use super::{BitSlice, DekuError, DekuRead, Msb0, Op};

    /// Read in big-endian bits to u32, multiply by f32, return f32
    pub fn bits_to_f32(
        rest: &BitSlice<Msb0, u8>,
        bits: usize,
        modifier: f32,
        modifier_op: Op,
    ) -> Result<(&BitSlice<Msb0, u8>, f32), DekuError> {
        let (rest, value) = u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(bits)))?;
        Ok(op(rest, value as f32, modifier, modifier_op))
    }

    /// Read in big-endian bits to i16, multiply by f32, return f32
    pub fn bits_i16_to_f32(
        rest: &BitSlice<Msb0, u8>,
        bits: usize,
        modifier: f32,
        modifier_op: Op,
    ) -> Result<(&BitSlice<Msb0, u8>, f32), DekuError> {
        let (rest, value) = i16::read(rest, (deku::ctx::Endian::Big, deku::ctx::Size::Bits(bits)))?;
        Ok(op(rest, f32::from(value), modifier, modifier_op))
    }

    fn op(
        rest: &BitSlice<Msb0, u8>,
        value: f32,
        modifier: f32,
        modifier_op: Op,
    ) -> (&BitSlice<Msb0, u8>, f32) {
        match modifier_op {
            Op::Multiply => (rest, value as f32 * modifier),
            Op::Divide => (rest, value as f32 / modifier),
            Op::Add => (rest, value as f32 + modifier),
            Op::Subtract => (rest, value as f32 - modifier),
        }
    }

    /// Read in big-endian bits, multiply by f32, return Some(f32)
    pub fn bits_to_optionf32(
        rest: &BitSlice<Msb0, u8>,
        bits: usize,
        modifier: f32,
        modifier_op: Op,
    ) -> Result<(&BitSlice<Msb0, u8>, Option<f32>), DekuError> {
        bits_to_f32(rest, bits, modifier, modifier_op).map(|(rest, f)| (rest, Some(f)))
    }
}

pub mod write {
    use super::{BitVec, DekuError, DekuWrite, Msb0, Op};

    pub fn f32_u32(
        value: &f32,
        bits: usize,
        modifier: f32,
        modifier_op: Op,
        output: &mut BitVec<Msb0, u8>,
    ) -> Result<(), DekuError> {
        // TODO this should be function for this and the other one
        let value = match modifier_op {
            Op::Multiply => *value * modifier,
            Op::Divide => *value / modifier,
            Op::Add => *value + modifier,
            Op::Subtract => *value - modifier,
        };
        (value as u32).write(
            output,
            (deku::ctx::Endian::Big, deku::ctx::Size::Bits(bits)),
        )
    }

    pub fn f32_optionu32(
        value: &Option<f32>,
        bits: usize,
        modifier: f32,
        modifier_op: Op,
        output: &mut BitVec<Msb0, u8>,
    ) -> Result<(), DekuError> {
        value.map_or(Ok(()), |value| {
            f32_u32(&value, bits, modifier, modifier_op, output)
        })
    }

    pub fn f32_i32(
        value: &f32,
        bits: usize,
        modifier: f32,
        modifier_op: Op,
        output: &mut BitVec<Msb0, u8>,
    ) -> Result<(), DekuError> {
        let value = match modifier_op {
            Op::Multiply => *value * modifier,
            Op::Divide => *value / modifier,
            Op::Add => *value + modifier,
            Op::Subtract => *value - modifier,
        };
        (value as i32).write(
            output,
            (deku::ctx::Endian::Big, deku::ctx::Size::Bits(bits)),
        )
    }
}
