use anyhow::Result;

use crate::rhif::{
    spec::{Cast, OpCode},
    Object,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct LowerInferredCastsPass {}

impl Pass for LowerInferredCastsPass {
    fn name(&self) -> &'static str {
        "lower_inferred_casts"
    }
    fn description(&self) -> &'static str {
        "Lower inferred casts to concrete casts"
    }
    fn run(mut input: Object) -> Result<Object> {
        for op in input.ops.iter_mut() {
            match op.clone() {
                OpCode::AsBits(cast) => {
                    if cast.len.is_none() {
                        let dest_width = input.kind[&cast.lhs].bits();
                        *op = OpCode::AsBits(Cast {
                            lhs: cast.lhs,
                            arg: cast.arg,
                            len: Some(dest_width),
                        })
                    }
                }
                OpCode::AsSigned(cast) => {
                    if cast.len.is_none() {
                        let dest_width = input.kind[&cast.lhs].bits();
                        *op = OpCode::AsSigned(Cast {
                            lhs: cast.lhs,
                            arg: cast.arg,
                            len: Some(dest_width),
                        })
                    }
                }
                _ => {}
            }
        }
        Ok(input)
    }
}
