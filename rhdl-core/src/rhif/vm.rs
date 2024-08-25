use std::collections::BTreeMap;

use crate::rhif::object::Object;
use crate::rhif::spec::{
    AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec, Index,
    Member, OpCode, Repeat, Slot, Struct, Tuple, Unary,
};
use crate::types::path::Path;
use crate::{ast::ast_impl::FunctionId, TypedBits};
use crate::{Digital, Kind};

use anyhow::Result;

use anyhow::{anyhow, bail};

use super::object::LocatedOpCode;
use super::runtime_ops::{array, binary, tuple, unary};
use super::spec::{LiteralId, Retime, Select, Splice};

struct VMState<'a> {
    reg_stack: &'a mut [Option<TypedBits>],
    literals: &'a BTreeMap<LiteralId, TypedBits>,
    obj: &'a Object,
}

impl<'a> VMState<'a> {
    fn read(&self, slot: Slot) -> Result<TypedBits> {
        match slot {
            Slot::Literal(l) => Ok(self.literals[&l].clone()),
            Slot::Register(r) => self.reg_stack[r.0]
                .clone()
                .ok_or(anyhow!("ICE Register {r:?} is not initialized")),
            Slot::Empty => Ok(TypedBits::EMPTY),
        }
    }
    fn write(&mut self, slot: Slot, value: TypedBits) -> Result<()> {
        match slot {
            Slot::Literal(_) => bail!("ICE Cannot write to literal"),
            Slot::Register(r) => {
                self.reg_stack[r.0] = Some(value);
                Ok(())
            }
            Slot::Empty => {
                if value.kind.is_empty() {
                    Ok(())
                } else {
                    bail!("ICE Cannot write non-empty value to empty slot")
                }
            }
        }
    }
    fn resolve_dynamic_paths(&mut self, path: &Path) -> Result<Path> {
        let mut result = Path::default();
        for element in &path.elements {
            match element {
                crate::types::path::PathElement::DynamicIndex(slot) => {
                    let slot = self.read(*slot)?;
                    let ndx = slot.as_i64()?;
                    result = result.index(ndx as usize);
                }
                _ => result.elements.push(element.clone()),
            }
        }
        Ok(result)
    }
}

fn execute_block(ops: &[LocatedOpCode], state: &mut VMState) -> Result<()> {
    for lop in ops {
        let op = &lop.op;
        match op {
            OpCode::Noop => {}
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
                let arg1 = state.read(*arg1)?;
                let arg2 = state.read(*arg2)?;
                let result = binary(*op, arg1, arg2)?;
                state.write(*lhs, result)?;
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                let arg1 = state.read(*arg1)?;
                let result = unary(*op, arg1)?;
                state.write(*lhs, result)?;
            }
            OpCode::Comment(_) => {}
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                let cond = state.read(*cond)?;
                let true_value = state.read(*true_value)?;
                let false_value = state.read(*false_value)?;
                if cond.any().as_bool()? {
                    state.write(*lhs, true_value)?;
                } else {
                    state.write(*lhs, false_value)?;
                }
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                let arg = state.read(*arg)?;
                let path = state.resolve_dynamic_paths(path)?;
                let result = arg.path(&path)?;
                state.write(*lhs, result)?;
            }
            OpCode::Splice(Splice {
                lhs,
                orig: rhs,
                path,
                subst: arg,
            }) => {
                let rhs_val = state.read(*rhs)?;
                let path = state.resolve_dynamic_paths(path)?;
                let arg_val = state.read(*arg)?;
                let result = rhs_val.splice(&path, arg_val)?;
                state.write(*lhs, result)?;
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                state.write(*lhs, state.read(*rhs)?)?;
            }
            OpCode::Tuple(Tuple { lhs, fields }) => {
                let fields = fields
                    .iter()
                    .map(|x| state.read(*x))
                    .collect::<Result<Vec<_>>>()?;
                let result = tuple(&fields);
                state.write(*lhs, result)?;
            }
            OpCode::Array(Array { lhs, elements }) => {
                let elements = elements
                    .iter()
                    .map(|x| state.read(*x))
                    .collect::<Result<Vec<_>>>()?;
                let result = array(&elements);
                state.write(*lhs, result)?;
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template,
            }) => {
                let mut result = if let Some(rest) = rest {
                    state.read(*rest)?
                } else {
                    template.clone()
                };
                for field in fields {
                    let value = state.read(field.value)?;
                    let path = match &field.member {
                        Member::Unnamed(ndx) => Path::default().tuple_index(*ndx as usize),
                        Member::Named(name) => Path::default().field(name),
                    };
                    result = result.splice(&path, value)?;
                }
                state.write(*lhs, result)?;
            }
            OpCode::Enum(Enum {
                lhs,
                fields,
                template,
            }) => {
                let mut result = template.clone();
                for field in fields {
                    let base_path =
                        Path::default().payload_by_value(template.discriminant()?.as_i64()?);
                    let value = state.read(field.value)?;
                    let path = match &field.member {
                        Member::Unnamed(ndx) => base_path.tuple_index(*ndx as usize),
                        Member::Named(name) => base_path.field(name),
                    };
                    result = result.splice(&path, value)?;
                }
                state.write(*lhs, result)?;
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                let discriminant = state.read(*discriminant)?;
                let arm = table
                    .iter()
                    .find(|(disc, _)| match disc {
                        CaseArgument::Slot(disc) => discriminant == state.read(*disc).unwrap(),
                        CaseArgument::Wild => true,
                    })
                    .ok_or(anyhow!("ICE Case was not exhaustive"))?
                    .1;
                let arm = state.read(arm)?;
                state.write(*lhs, arm)?;
            }
            OpCode::AsBits(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg)?;
                let len = len.ok_or(anyhow!("ICE Cast length not provided"))?;
                let result = arg.unsigned_cast(len)?;
                state.write(*lhs, result)?;
            }
            OpCode::AsSigned(Cast { lhs, arg, len }) => {
                let arg = state.read(*arg)?;
                let len = len.ok_or(anyhow!("ICE Cast length not provided"))?;
                let result = arg.signed_cast(len)?;
                state.write(*lhs, result)?;
            }
            OpCode::Retime(Retime { lhs, arg, color }) => {
                let mut arg = state.read(*arg)?;
                if let Some(color) = color {
                    arg.kind = Kind::make_signal(arg.kind, *color);
                }
                state.write(*lhs, arg)?;
            }
            OpCode::Exec(Exec { lhs, id, args }) => {
                let args = args
                    .iter()
                    .map(|x| state.read(*x))
                    .collect::<Result<Vec<_>>>()?;
                let func = &state.obj.externals[id];
                let result = execute(&func, args)?;
                state.write(*lhs, result)?;
            }
            OpCode::Repeat(Repeat { lhs, value, len }) => {
                let value = state.read(*value)?;
                let len = *len as usize;
                let result = value.repeat(len);
                state.write(*lhs, result)?;
            }
        }
    }
    Ok(())
}

pub fn execute(obj: &Object, arguments: Vec<TypedBits>) -> Result<TypedBits> {
    // Load the object for this function
    if obj.arguments.len() != arguments.len() {
        bail!(
            "Function {fn_name} expected {expected} arguments, got {got}",
            fn_name = obj.name,
            expected = obj.arguments.len(),
            got = arguments.len()
        );
    }
    for (ndx, arg) in arguments.iter().enumerate() {
        let arg_kind = &arg.kind;
        let obj_kind = obj
            .kind
            .get(&obj.arguments[ndx])
            .ok_or(anyhow!("ICE argument {ndx} type not found in object"))?;
        if obj_kind != arg_kind {
            bail!(
                "Function {fn_name} argument {ndx} expected {expected:?}, got {got:?}",
                fn_name = obj.name,
                ndx = ndx,
                expected = obj.kind.get(&obj.arguments[ndx]).unwrap(),
                got = arg_kind
            );
        }
    }
    // Allocate registers for the function call.
    let max_reg = obj.reg_max_index().0 + 1;
    let mut reg_stack = vec![None; max_reg + 1];
    // Copy the arguments into the appropriate registers
    for (ndx, arg) in arguments.into_iter().enumerate() {
        let r = obj.arguments[ndx];
        reg_stack[r.0] = Some(arg);
    }
    let mut state = VMState {
        reg_stack: &mut reg_stack,
        literals: &obj.literals,
        obj,
    };
    execute_block(&obj.ops, &mut state)?;
    match obj.return_slot {
        Slot::Empty => Ok(TypedBits::EMPTY),
        Slot::Register(r) => reg_stack
            .get(r.0)
            .cloned()
            .ok_or(anyhow!("return slot not found"))?
            .ok_or(anyhow!("ICE return slot is not initialized")),
        Slot::Literal(ndx) => Ok(obj.literals[&ndx].clone()),
    }
}
