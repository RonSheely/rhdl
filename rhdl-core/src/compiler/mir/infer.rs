use std::collections::BTreeMap;

use crate::{
    ast::ast_impl::{ExprLit, NodeId},
    compiler::mir::{error::RHDLTypeCheckError, ty::SignFlag},
    error::RHDLError,
    path::{sub_kind, Path, PathElement},
    rhif::{
        spec::{AluBinary, AluUnary, CaseArgument, OpCode, Slot},
        Object,
    },
    Digital, Kind, TypedBits,
};

use super::{
    error::{RHDLClockCoherenceViolation, RHDLCompileError, RHDLTypeError, TypeCheck, ICE},
    mir_impl::{Mir, TypeEquivalence},
    ty::{TypeId, UnifyContext},
};
#[derive(Debug, Clone)]
pub struct TypeUnaryOp {
    op: AluUnary,
    lhs: TypeId,
    arg1: TypeId,
}

#[derive(Debug, Clone)]
pub struct TypeBinOp {
    op: AluBinary,
    lhs: TypeId,
    arg1: TypeId,
    arg2: TypeId,
}

#[derive(Debug, Clone)]
pub struct TypeIndex {
    lhs: TypeId,
    arg: TypeId,
    path: Path,
}

#[derive(Debug, Clone)]
pub struct TypeSelect {
    selector: TypeId,
    true_value: TypeId,
    false_value: TypeId,
}

#[derive(Debug, Clone)]
pub struct TypeOperation {
    id: NodeId,
    kind: TypeOperationKind,
}

#[derive(Debug, Clone)]
pub enum TypeOperationKind {
    UnaryOp(TypeUnaryOp),
    BinOp(TypeBinOp),
    Index(TypeIndex),
    Select(TypeSelect),
}

pub struct MirTypeInference<'a> {
    ctx: UnifyContext,
    slot_map: BTreeMap<Slot, TypeId>,
    mir: &'a Mir,
    type_ops: Vec<TypeOperation>,
}

type Result<T> = std::result::Result<T, RHDLError>;

impl<'a> MirTypeInference<'a> {
    fn new(mir: &'a Mir) -> Self {
        Self {
            mir,
            ctx: UnifyContext::default(),
            slot_map: BTreeMap::default(),
            type_ops: Vec::new(),
        }
    }
    fn raise_ice(&self, cause: ICE, id: NodeId) -> Box<RHDLCompileError> {
        let source_span = self.mir.symbols.source.span(id);
        Box::new(RHDLCompileError {
            cause,
            src: self.mir.symbols.source.source.clone(),
            err_span: source_span.into(),
        })
    }
    fn raise_type_error(&self, cause: TypeCheck, id: NodeId) -> Box<RHDLTypeError> {
        let source_span = self.mir.symbols.source.span(id);
        Box::new(RHDLTypeError {
            cause,
            src: self.mir.symbols.source.source.clone(),
            err_span: source_span.into(),
        })
    }
    fn cast_literal_to_inferred_type(&mut self, t: ExprLit, ty: TypeId) -> Result<TypedBits> {
        let kind = self.ctx.into_kind(ty)?;
        Ok(match t {
            ExprLit::TypedBits(tb) => {
                if tb.value.kind != kind {
                    return Err(self
                        .raise_type_error(
                            TypeCheck::InferredLiteralTypeMismatch {
                                typ: tb.value.kind.clone(),
                                kind: kind.clone(),
                            },
                            ty.id,
                        )
                        .into());
                }
                tb.value
            }
            ExprLit::Int(x) => {
                if kind.is_unsigned() {
                    let x_as_u128 = if let Some(x) = x.strip_prefix("0b") {
                        u128::from_str_radix(x, 2)?
                    } else if let Some(x) = x.strip_prefix("0o") {
                        u128::from_str_radix(x, 8)?
                    } else if let Some(x) = x.strip_prefix("0x") {
                        u128::from_str_radix(x, 16)?
                    } else {
                        x.parse::<u128>()?
                    };
                    x_as_u128.typed_bits().unsigned_cast(kind.bits())?
                } else {
                    let x_as_i128 = if let Some(x) = x.strip_prefix("0b") {
                        i128::from_str_radix(x, 2)?
                    } else if let Some(x) = x.strip_prefix("0o") {
                        i128::from_str_radix(x, 8)?
                    } else if let Some(x) = x.strip_prefix("0x") {
                        i128::from_str_radix(x, 16)?
                    } else {
                        x.parse::<i128>()?
                    };
                    x_as_i128.typed_bits().signed_cast(kind.bits())?
                }
            }
            ExprLit::Bool(b) => b.typed_bits(),
        })
    }
    fn unify(&mut self, id: NodeId, lhs: TypeId, rhs: TypeId) -> Result<()> {
        eprintln!("Unifying {} and {}", self.ctx.desc(lhs), self.ctx.desc(rhs));
        if self.ctx.unify(lhs, rhs).is_err() {
            let lhs_span = self.mir.symbols.source.span(lhs.id);
            let rhs_span = self.mir.symbols.source.span(rhs.id);
            let lhs_desc = self.ctx.desc(lhs);
            let rhs_desc = self.ctx.desc(rhs);
            let cause_span = self.mir.symbols.source.span(id);
            let cause_description = "Because of this expression".to_owned();
            return Err(Box::new(RHDLTypeCheckError {
                src: self.mir.symbols.source.source.clone(),
                lhs_type: lhs_desc,
                lhs_span: lhs_span.into(),
                rhs_type: rhs_desc,
                rhs_span: rhs_span.into(),
                cause_description,
                cause_span: cause_span.into(),
            })
            .into());
        }
        Ok(())
    }
    fn import_literals(&mut self) {
        for (slot, lit) in &self.mir.literals {
            let id = self.mir.symbols.slot_map[slot].node;
            let ty = match lit {
                ExprLit::TypedBits(tb) => self.ctx.from_kind(id, &tb.value.kind),
                ExprLit::Int(_) => self.ctx.ty_integer(id),
                ExprLit::Bool(_) => self.ctx.ty_bool(id),
            };
            self.slot_map.insert(*slot, ty);
        }
    }
    fn import_signature(&mut self) -> Result<()> {
        for slot in &self.mir.arguments {
            let id = self.mir.symbols.slot_map[slot].node;
            let kind = &self.mir.ty[slot];
            let ty = self.ctx.from_kind(id, kind);
            self.slot_map.insert(*slot, ty);
        }
        let id = self.mir.symbols.slot_map[&self.mir.return_slot].node;
        let return_kind = &self.mir.ty[&self.mir.return_slot];
        let return_ty = self.ctx.from_kind(id, return_kind);
        self.slot_map.insert(self.mir.return_slot, return_ty);
        Ok(())
    }
    fn import_type_declarations(&mut self) -> Result<()> {
        for (slot, ty) in &self.mir.ty {
            let id = self.mir.symbols.slot_map[slot].node;
            let ty = self.ctx.from_kind(id, ty);
            self.slot_map.insert(*slot, ty);
        }
        Ok(())
    }
    fn import_type_equality(&mut self) -> Result<()> {
        for TypeEquivalence { id, lhs, rhs } in &self.mir.ty_equate {
            let lhs_ty = self.slot_ty(*lhs);
            let rhs_ty = self.slot_ty(*rhs);
            self.unify(*id, lhs_ty, rhs_ty)?;
        }
        Ok(())
    }
    fn slot_ty(&mut self, slot: Slot) -> TypeId {
        let id = self.mir.symbols.slot_map[&slot].node;
        if matches!(slot, Slot::Empty) {
            return self.ctx.ty_empty(id);
        }
        if let Some(ty) = self.slot_map.get(&slot) {
            *ty
        } else {
            let var = self.ctx.ty_var(id);
            self.slot_map.insert(slot, var);
            var
        }
    }
    fn slot_tys(&mut self, slots: &[Slot]) -> Vec<TypeId> {
        slots.iter().map(|slot| self.slot_ty(*slot)).collect()
    }
    fn all_slots_resolved(&mut self) -> bool {
        self.unresolved_slot_typeid().is_none()
    }
    fn unresolved_slot_typeid(&mut self) -> Option<TypeId> {
        for ty in self.slot_map.values() {
            if self.ctx.into_kind(*ty).is_err() {
                return Some(*ty);
            }
        }
        None
    }
    fn try_unary(&mut self, id: NodeId, op: &TypeUnaryOp) -> Result<()> {
        let a1 = self.ctx.apply(op.arg1);
        match op.op {
            AluUnary::All | AluUnary::Any | AluUnary::Xor => {
                let bool_ty = self.ctx.ty_bool(id);
                if self.ctx.is_signal(a1) {
                    let clock_ty = self.ctx.ty_var(id);
                    let bool_sig = self.ctx.ty_signal(id, bool_ty, clock_ty);
                    self.unify(id, op.lhs, bool_sig)?;
                    if let Some(a1_clock) = self.ctx.project_signal_clock(a1) {
                        self.unify(id, clock_ty, a1_clock)?;
                    }
                } else {
                    self.unify(id, op.lhs, bool_ty)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn try_binop(&mut self, id: NodeId, op: &TypeBinOp) -> Result<()> {
        let a1 = self.ctx.apply(op.arg1);
        let a2 = self.ctx.apply(op.arg2);
        eprintln!(
            "Try to apply {:?} to {} and {}",
            op.op,
            self.ctx.desc(a1),
            self.ctx.desc(a2)
        );
        self.enforce_clocks(id, &[a1, a2, op.lhs])?;
        match &op.op {
            AluBinary::Add
            | AluBinary::Mul
            | AluBinary::BitAnd
            | AluBinary::BitOr
            | AluBinary::BitXor
            | AluBinary::Sub => {
                let a1_is_signal = self.ctx.is_signal(op.arg1);
                let a2_is_signal = self.ctx.is_signal(op.arg2);
                if a1_is_signal {
                    self.unify(id, op.lhs, op.arg1)?;
                }
                if a2_is_signal {
                    self.unify(id, op.lhs, op.arg2)?;
                }
                if !a1_is_signal && !a2_is_signal {
                    self.unify(id, op.lhs, op.arg1)?;
                    self.unify(id, op.lhs, op.arg2)?;
                }
                if let (Some(arg1_data), Some(arg2_data)) = (
                    self.ctx.project_signal_value(op.arg1),
                    self.ctx.project_signal_value(op.arg2),
                ) {
                    self.unify(id, arg1_data, arg2_data)?;
                }
                if let (Some(lhs_data), Some(arg1_data)) = (
                    self.ctx.project_signal_value(op.lhs),
                    self.ctx.project_signal_value(op.arg1),
                ) {
                    self.unify(id, lhs_data, arg1_data)?;
                }
                if let (Some(lhs_data), Some(arg2_data)) = (
                    self.ctx.project_signal_value(op.lhs),
                    self.ctx.project_signal_value(op.arg2),
                ) {
                    self.unify(id, lhs_data, arg2_data)?;
                }
            }
            AluBinary::Eq
            | AluBinary::Lt
            | AluBinary::Le
            | AluBinary::Ne
            | AluBinary::Ge
            | AluBinary::Gt => {
                if let Some(arg1_clock) = self.ctx.project_signal_clock(op.arg1) {
                    let lhs_var = self.ctx.ty_bool(id);
                    let lhs_sig = self.ctx.ty_signal(id, lhs_var, arg1_clock);
                    self.unify(id, op.lhs, lhs_sig)?;
                }
                if let Some(arg2_clock) = self.ctx.project_signal_clock(op.arg2) {
                    let lhs_var = self.ctx.ty_bool(id);
                    let lhs_sig = self.ctx.ty_signal(id, lhs_var, arg2_clock);
                    self.unify(id, op.lhs, lhs_sig)?;
                }
                if !self.ctx.is_signal(op.arg1) && !self.ctx.is_signal(op.arg2) {
                    let lhs_var = self.ctx.ty_bool(id);
                    self.unify(id, op.lhs, lhs_var)?;
                }
                if let (Some(arg1_data), Some(arg2_data)) = (
                    self.ctx.project_signal_value(op.arg1),
                    self.ctx.project_signal_value(op.arg2),
                ) {
                    self.unify(id, arg1_data, arg2_data)?;
                }
            }
            AluBinary::Shl | AluBinary::Shr => {
                if let Some(arg2) = self.ctx.project_signal_value(a2) {
                    eprintln!("Project signal value flag for {}", self.ctx.desc(a2));
                    if let Some(flag) = self.ctx.project_sign_flag(arg2) {
                        eprintln!("Project sign flag for {}", self.ctx.desc(a2));
                        let unsigned_flag = self.ctx.ty_sign_flag(id, SignFlag::Unsigned);
                        self.unify(id, flag, unsigned_flag)?;
                    }
                }
                if let (Some(lhs_data), Some(arg1_data)) = (
                    self.ctx.project_signal_value(op.lhs),
                    self.ctx.project_signal_value(op.arg1),
                ) {
                    self.unify(id, lhs_data, arg1_data)?;
                } else {
                    self.unify(id, op.lhs, op.arg1)?;
                }
            }
        }
        Ok(())
    }

    fn ty_path_project(&mut self, arg: TypeId, path: &Path, id: NodeId) -> Result<TypeId> {
        let mut arg = self.ctx.apply(arg);
        for element in path.elements.iter() {
            match element {
                PathElement::Index(ndx) => {
                    arg = self.ctx.ty_index(arg, *ndx)?;
                }
                PathElement::Field(member) => {
                    arg = self.ctx.ty_field(arg, member)?;
                }
                PathElement::EnumDiscriminant => {
                    arg = self.ctx.ty_enum_discriminant(arg)?;
                }
                PathElement::TupleIndex(ndx) => {
                    arg = self.ctx.ty_index(arg, *ndx)?;
                }
                PathElement::EnumPayload(member) => {
                    arg = self.ctx.ty_variant(arg, member)?;
                }
                PathElement::DynamicIndex(slot) => {
                    let index = self.slot_ty(*slot);
                    let usize_ty = self.ctx.ty_usize(id);
                    if slot.is_literal() {
                        self.unify(id, index, usize_ty)?;
                    } else {
                        let reg_ty = self.ctx.apply(index);
                        if self.ctx.is_generic_integer(reg_ty) {
                            // For more clearly defined types, it is someone else's problem
                            // to ensure that the index is properly typed.
                            self.unify(id, reg_ty, usize_ty)?;
                        }
                    }
                    arg = self.ctx.ty_index(arg, 0)?;
                }
                PathElement::EnumPayloadByValue(value) => {
                    arg = self.ctx.ty_variant_by_value(arg, *value)?;
                }
                PathElement::SignalValue => {
                    arg = self
                        .ctx
                        .project_signal_value(arg)
                        .ok_or(self.raise_type_error(TypeCheck::ExpectedSignalValue, id))?;
                }
            }
        }
        Ok(arg)
    }

    fn try_index(&mut self, id: NodeId, op: &TypeIndex) -> Result<()> {
        eprintln!(
            "Try to apply index to {} with path {:?}",
            self.ctx.desc(op.arg),
            op.path
        );
        let mut all_slots = vec![op.lhs, op.arg];
        all_slots.extend(op.path.dynamic_slots().map(|slot| self.slot_ty(*slot)));
        self.enforce_clocks(id, &all_slots)?;
        match self.ty_path_project(op.arg, &op.path, id) {
            Ok(ty) => self.unify(id, op.lhs, ty),
            Err(err) => {
                eprintln!("Error: {}", err);
                Ok(())
            }
        }
    }
    fn enforce_clocks(&mut self, id: NodeId, t: &[TypeId]) -> Result<()> {
        let clocks = t
            .iter()
            .filter_map(|ty| self.ctx.project_signal_clock(*ty))
            .collect::<Vec<_>>();
        for (first, second) in clocks.iter().zip(clocks.iter().skip(1)) {
            self.unify(id, *first, *second)?;
        }
        Ok(())
    }
    fn clock_domain_for_error(&mut self, ty: TypeId) -> String {
        let Some(ty) = self.ctx.project_signal_clock(ty) else {
            return "Const".to_string();
        };
        if let Ok(clock) = self.ctx.into_ty_clock(ty) {
            format!("{:?}", clock)
        } else {
            "Const".to_string()
        }
    }
    fn try_select(&mut self, id: NodeId, op: &TypeSelect) -> Result<()> {
        if self
            .enforce_clocks(id, &[op.selector, op.true_value, op.false_value])
            .is_err()
        {
            eprintln!("{:?}", op);
            eprintln!(
                "Selector span: {:?}",
                self.mir.symbols.source.span(op.selector.id)
            );
            eprintln!(
                "True value span: {:?}",
                self.mir.symbols.source.span(op.true_value.id)
            );
            eprintln!(
                "False value span: {:?}",
                self.mir.symbols.source.span(op.false_value.id)
            );
            eprintln!(
                "True value snippet: '{}'",
                self.mir.symbols.source.snippet(op.true_value.id)
            );
            return Err(Box::new(RHDLClockCoherenceViolation {
                src: self.mir.symbols.source.source.clone(),
                elements: vec![
                    (
                        format!(
                            "Selector belongs to {} clock domain",
                            self.clock_domain_for_error(op.selector)
                        ),
                        self.mir.symbols.source.span(op.selector.id).into(),
                    ),
                    (
                        format!(
                            "True value belongs to {} clock domain",
                            self.clock_domain_for_error(op.true_value)
                        ),
                        self.mir.symbols.source.span(op.true_value.id).into(),
                    ),
                    (
                        format!(
                            "False value belongs to {} clock domain",
                            self.clock_domain_for_error(op.false_value)
                        ),
                        self.mir.symbols.source.span(op.false_value.id).into(),
                    ),
                ],
                cause_description:
                    "Select operation requires all three belong to the same clock domain"
                        .to_string(),
                cause_span: self.mir.symbols.source.span(id).into(),
            })
            .into());
        }
        Ok(())
    }
    fn try_type_op(&mut self, op: &TypeOperation) -> Result<()> {
        let id = op.id;
        match &op.kind {
            TypeOperationKind::BinOp(binop) => self.try_binop(id, binop),
            TypeOperationKind::Index(index) => self.try_index(id, index),
            TypeOperationKind::UnaryOp(unary) => self.try_unary(id, unary),
            TypeOperationKind::Select(select) => self.try_select(id, select),
        }
    }
    fn try_type_ops(&mut self, iteration_count: usize, ops: &[TypeOperation]) -> Result<()> {
        for loop_count in 0..iteration_count {
            eprintln!("Iteration {}", loop_count);
            let mod_state = self.ctx.modification_state();
            for op in ops {
                self.try_type_op(op)?;
            }
            if self.ctx.modification_state() == mod_state {
                break;
            }
            if self.all_slots_resolved() {
                break;
            }
        }
        Ok(())
    }
    fn process_ops(&mut self) -> Result<()> {
        for op in &self.mir.ops {
            eprintln!("Processing op {:?}", op.op);
            let id = op.source;
            match &op.op {
                OpCode::Array(array) => {
                    let lhs = self.slot_ty(array.lhs);
                    let rhs = self.slot_tys(&array.elements);
                    let array_base = self.ctx.ty_var(id);
                    let array_len = self.ctx.ty_const_len(id, rhs.len());
                    let lhs_ty = self.ctx.ty_array(id, array_base, array_len);
                    self.unify(id, lhs, lhs_ty)?;
                    for element in rhs {
                        self.unify(id, element, array_base)?;
                    }
                }
                OpCode::Assign(assign) => {
                    let lhs = self.slot_ty(assign.lhs);
                    let rhs = self.slot_ty(assign.rhs);
                    self.unify(id, lhs, rhs)?;
                }
                OpCode::AsBits(as_bits) => {
                    let arg = self.slot_ty(as_bits.arg);
                    let lhs = self.slot_ty(as_bits.lhs);
                    let len = if let Some(len) = as_bits.len {
                        self.ctx.ty_const_len(id, len)
                    } else {
                        self.ctx.ty_var(id)
                    };
                    let lhs_ty = self.ctx.ty_bits(id, len);
                    self.unify(id, lhs, lhs_ty)?;
                    let len_128 = self.ctx.ty_const_len(id, 128);
                    let arg_ty = self.ctx.ty_bits(id, len_128);
                    self.unify(id, arg, arg_ty)?;
                }
                OpCode::AsSigned(as_signed) => {
                    let arg = self.slot_ty(as_signed.arg);
                    let lhs = self.slot_ty(as_signed.lhs);
                    let len = if let Some(len) = as_signed.len {
                        self.ctx.ty_const_len(id, len)
                    } else {
                        self.ctx.ty_var(id)
                    };
                    let lhs_ty = self.ctx.ty_signed(id, len);
                    self.unify(id, lhs, lhs_ty)?;
                    let len_128 = self.ctx.ty_const_len(id, 128);
                    let arg_ty = self.ctx.ty_signed(id, len_128);
                    self.unify(id, arg, arg_ty)?;
                }
                OpCode::Binary(binary) => {
                    let lhs = self.slot_ty(binary.lhs);
                    let arg1 = self.slot_ty(binary.arg1);
                    let arg2 = self.slot_ty(binary.arg2);
                    self.type_ops.push(TypeOperation {
                        id,
                        kind: TypeOperationKind::BinOp(TypeBinOp {
                            op: binary.op,
                            lhs,
                            arg1,
                            arg2,
                        }),
                    });
                }
                OpCode::Case(case) => {
                    let lhs = self.slot_ty(case.lhs);
                    let disc = self.slot_ty(case.discriminant);
                    for (test, value) in case.table.iter() {
                        match test {
                            CaseArgument::Slot(slot) => {
                                let ty = self.slot_ty(*slot);
                                let free_var = self.ctx.ty_var(id);
                                eprintln!(
                                    "Adding constraint {} = {} == {}",
                                    self.ctx.desc(free_var),
                                    self.ctx.desc(disc),
                                    self.ctx.desc(ty)
                                );
                                self.type_ops.push(TypeOperation {
                                    id: op.source,
                                    kind: TypeOperationKind::BinOp(TypeBinOp {
                                        op: AluBinary::Eq,
                                        lhs: free_var,
                                        arg1: disc,
                                        arg2: ty,
                                    }),
                                });
                            }
                            CaseArgument::Wild => {}
                        }
                        let val_ty = self.slot_ty(*value);
                        self.unify(id, lhs, val_ty)?;
                    }
                }
                OpCode::Enum(enumerate) => {
                    let lhs = self.slot_ty(enumerate.lhs);
                    let Kind::Enum(enum_k) = &enumerate.template.kind else {
                        return Err(self
                            .raise_ice(
                                ICE::ExpectedEnumTemplate {
                                    kind: enumerate.template.kind.clone(),
                                },
                                op.source,
                            )
                            .into());
                    };
                    let lhs_ty = self.ctx.ty_enum(id, enum_k);
                    self.unify(id, lhs, lhs_ty)?;
                    let discriminant = enumerate.template.discriminant()?.as_i64()?;
                    for field in &enumerate.fields {
                        let path = match &field.member {
                            crate::rhif::spec::Member::Named(name) => {
                                Path::default().payload_by_value(discriminant).field(name)
                            }
                            crate::rhif::spec::Member::Unnamed(ndx) => Path::default()
                                .payload_by_value(discriminant)
                                .tuple_index(*ndx as usize),
                        };
                        let field_kind = sub_kind(enumerate.template.kind.clone(), &path)?;
                        let field_ty = self.ctx.from_kind(id, &field_kind);
                        let field_slot = self.slot_ty(field.value);
                        self.unify(id, field_ty, field_slot)?;
                    }
                }
                OpCode::Exec(exec) => {
                    let external_fn = &self.mir.stash[exec.id.0];
                    let signature = &external_fn.signature;
                    for (arg_kind, arg_slot) in signature.arguments.iter().zip(exec.args.iter()) {
                        let arg_ty = self.slot_ty(*arg_slot);
                        let arg_kind = self.ctx.from_kind(id, arg_kind);
                        self.unify(id, arg_ty, arg_kind)?;
                    }
                    let ret_ty = self.slot_ty(exec.lhs);
                    let ret_kind = self.ctx.from_kind(id, &signature.ret);
                    self.unify(id, ret_ty, ret_kind)?;
                }
                OpCode::Index(index) => {
                    let arg = self.slot_ty(index.arg);
                    let lhs = self.slot_ty(index.lhs);
                    let path = index.path.clone();
                    self.type_ops.push(TypeOperation {
                        id,
                        kind: TypeOperationKind::Index(TypeIndex { lhs, arg, path }),
                    });
                }
                OpCode::Repeat(repeat) => {
                    let lhs = self.slot_ty(repeat.lhs);
                    let value = self.slot_ty(repeat.value);
                    let len = self.ctx.ty_const_len(id, repeat.len as usize);
                    let lhs_ty = self.ctx.ty_array(id, value, len);
                    self.unify(id, lhs, lhs_ty)?;
                }
                OpCode::Retime(retime) => {
                    let lhs = self.slot_ty(retime.lhs);
                    let arg = self.slot_ty(retime.arg);
                    let color = retime.color;
                    let sig_ty = self.ctx.ty_var(id);
                    let sig_clock_lhs = self.ctx.ty_var(id);
                    let sig = self.ctx.ty_signal(id, sig_ty, sig_clock_lhs);
                    self.unify(id, lhs, sig)?;
                    let sig_ty = self.ctx.ty_var(id);
                    let sig_clock_rhs = self.ctx.ty_var(id);
                    let sig = self.ctx.ty_signal(id, sig_ty, sig_clock_rhs);
                    self.unify(id, arg, sig)?;
                    self.unify(id, sig_clock_lhs, sig_clock_rhs)?;
                    if let Some(color) = color {
                        let clk = self.ctx.ty_clock(id, color);
                        self.unify(id, sig_clock_lhs, clk)?;
                    }
                }
                OpCode::Select(select) => {
                    let cond = self.mir.find_root_for_slot(id, select.cond);
                    let true_value = self.mir.find_root_for_slot(id, select.true_value);
                    let false_value = self.mir.find_root_for_slot(id, select.false_value);
                    let lhs = self.slot_ty(select.lhs);
                    let cond = self.slot_ty(cond);
                    let arg1 = self.slot_ty(true_value);
                    let arg2 = self.slot_ty(false_value);
                    self.unify(id, lhs, arg1)?;
                    self.unify(id, lhs, arg2)?;
                    self.type_ops.push(TypeOperation {
                        id: op.source,
                        kind: TypeOperationKind::Select(TypeSelect {
                            selector: cond,
                            true_value: arg1,
                            false_value: arg2,
                        }),
                    });
                }
                OpCode::Splice(splice) => {
                    let lhs = self.slot_ty(splice.lhs);
                    let orig = self.slot_ty(splice.orig);
                    let subst = self.slot_ty(splice.subst);
                    let path = &splice.path;
                    self.unify(id, lhs, orig)?;
                    // Reflect the constraint that
                    // ty(subst) = ty(lhs[path])
                    self.type_ops.push(TypeOperation {
                        id,
                        kind: TypeOperationKind::Index(TypeIndex {
                            lhs: subst,
                            arg: lhs,
                            path: path.clone(),
                        }),
                    });
                }
                OpCode::Struct(structure) => {
                    let lhs = self.slot_ty(structure.lhs);
                    let Kind::Struct(strukt) = &structure.template.kind else {
                        return Err(self
                            .raise_ice(
                                ICE::ExpectedStructTemplate {
                                    kind: structure.template.kind.clone(),
                                },
                                op.source,
                            )
                            .into());
                    };
                    let lhs_ty = self.ctx.ty_struct(id, strukt);
                    self.unify(id, lhs, lhs_ty)?;
                    for field in &structure.fields {
                        let field_kind = strukt.get_field_kind(&field.member)?;
                        let field_ty = self.ctx.from_kind(id, &field_kind);
                        let field_slot = self.slot_ty(field.value);
                        self.unify(id, field_ty, field_slot)?;
                    }
                    if let Some(rest) = structure.rest {
                        let rest_ty = self.slot_ty(rest);
                        self.unify(id, lhs_ty, rest_ty)?;
                    }
                    self.unify(id, lhs, lhs_ty)?;
                }
                OpCode::Tuple(tuple) => {
                    let lhs = self.slot_ty(tuple.lhs);
                    let tys = tuple
                        .fields
                        .iter()
                        .map(|slot| self.slot_ty(*slot))
                        .collect();
                    let lhs_ty = self.ctx.ty_tuple(id, tys);
                    self.unify(id, lhs, lhs_ty)?;
                }
                OpCode::Unary(unary) => {
                    let lhs = self.slot_ty(unary.lhs);
                    let arg1 = self.slot_ty(unary.arg1);
                    match unary.op {
                        AluUnary::Not => {
                            self.unify(id, lhs, arg1)?;
                        }
                        AluUnary::Neg => {
                            let len = self.ctx.ty_var(id);
                            let signed_ty = self.ctx.ty_signed(id, len);
                            self.unify(id, lhs, signed_ty)?;
                            self.unify(id, arg1, signed_ty)?;
                        }
                        AluUnary::All | AluUnary::Any | AluUnary::Xor => {
                            self.type_ops.push(TypeOperation {
                                id: op.source,
                                kind: TypeOperationKind::UnaryOp(TypeUnaryOp {
                                    op: unary.op,
                                    lhs,
                                    arg1,
                                }),
                            });
                        }
                        AluUnary::Unsigned => {
                            let len = self.ctx.ty_var(id);
                            let signed_ty = self.ctx.ty_signed(id, len);
                            let unsigned_ty = self.ctx.ty_bits(id, len);
                            self.unify(id, lhs, unsigned_ty)?;
                            self.unify(id, arg1, signed_ty)?;
                        }
                        AluUnary::Signed => {
                            let len = self.ctx.ty_var(id);
                            let signed_ty = self.ctx.ty_signed(id, len);
                            let unsigned_ty = self.ctx.ty_bits(id, len);
                            self.unify(id, lhs, signed_ty)?;
                            self.unify(id, arg1, unsigned_ty)?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

pub fn infer(mir: Mir) -> Result<Object> {
    let mut infer = MirTypeInference::new(&mir);
    infer.import_literals();
    infer.import_signature()?;
    infer.import_type_equality()?;
    infer.import_type_declarations()?;
    eprintln!("=================================");
    eprintln!("Before inference");
    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        eprintln!("Slot {:?} -> type {}", slot, ty);
    }
    for op in mir.ops.iter() {
        eprintln!("{:?}", op.op);
    }
    eprintln!("=================================");
    if let Err(e) = infer.process_ops() {
        eprintln!("Error: {}", e);
        for (slot, ty) in &infer.slot_map {
            let ty = infer.ctx.apply(*ty);
            let ty = infer.ctx.desc(ty);
            eprintln!("Slot {:?} -> type {}", slot, ty);
        }
        return Err(e);
    }
    infer.process_ops()?;
    let type_ops = infer.type_ops.clone();
    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        eprintln!("Slot {:?} -> type {}", slot, ty);
    }
    infer.try_type_ops(5, &type_ops)?;
    eprintln!("Try to replace generic literals with ?32");
    // Try to replace generic literals with (b/s)32
    if !infer.all_slots_resolved() {
        for lit in mir.literals.keys() {
            let ty = infer.slot_ty(*lit);
            if infer.ctx.is_unsized_integer(ty) {
                let i32_len = infer.ctx.ty_const_len(ty.id, 32);
                let m32_ty = infer.ctx.ty_maybe_signed(ty.id, i32_len);
                eprintln!(
                    "Literal {:?} -> {} U {}",
                    lit,
                    infer.ctx.desc(ty),
                    infer.ctx.desc(m32_ty)
                );
                infer.unify(ty.id, ty, m32_ty)?;
            }
        }
    }
    eprintln!("Recheck delayed inference rools");
    infer.try_type_ops(5, &type_ops)?;

    eprintln!("Try to replace generic literals with i32");
    // Try to replace any generic literals with i32s
    if !infer.all_slots_resolved() {
        for lit in mir.literals.keys() {
            let ty = infer.slot_ty(*lit);
            if let Some(ty_sign) = infer.ctx.project_sign_flag(ty) {
                if infer.ctx.is_unresolved(ty_sign) {
                    let sign_flag = infer.ctx.ty_sign_flag(ty.id, SignFlag::Signed);
                    infer.unify(ty.id, ty_sign, sign_flag)?;
                }
            }
        }
    }
    eprintln!("Recheck delayed inference rules");
    infer.try_type_ops(5, &type_ops)?;

    if let Some(ty) = infer.unresolved_slot_typeid() {
        eprintln!("=================================");
        eprintln!("Inference failed");
        for (slot, ty) in &infer.slot_map {
            let ty = infer.ctx.apply(*ty);
            let ty = infer.ctx.desc(ty);
            eprintln!("Slot {:?} -> type {}", slot, ty);
        }
        for op in mir.ops.iter() {
            eprintln!("{:?}", op.op);
        }

        eprintln!("=================================");

        for lit in mir.literals.keys() {
            let ty = infer.slot_ty(*lit);
            if infer.ctx.into_kind(ty).is_err() {
                eprintln!("Literal {:?} -> {}", lit, infer.ctx.desc(ty));
            }
        }
        return Err(infer
            .raise_type_error(TypeCheck::UnableToDetermineType, ty.id)
            .into());
    }

    for (slot, ty) in &infer.slot_map {
        let ty = infer.ctx.apply(*ty);
        let ty = infer.ctx.desc(ty);
        eprintln!("Slot {:?} -> type {}", slot, ty);
    }
    let final_type_map: BTreeMap<Slot, TypeId> = infer
        .slot_map
        .clone()
        .into_iter()
        .map(|(slot, ty)| {
            let ty = infer.ctx.apply(ty);
            (slot, ty)
        })
        .collect();
    let kind = final_type_map
        .iter()
        .map(|(slot, ty)| infer.ctx.into_kind(*ty).map(|val| (*slot, val)))
        .collect::<anyhow::Result<BTreeMap<_, _>>>()
        .unwrap();
    for op in mir.ops.iter() {
        eprintln!("{:?}", op.op);
    }
    let literals = mir
        .literals
        .clone()
        .into_iter()
        .map(|(slot, lit)| {
            infer
                .cast_literal_to_inferred_type(lit, final_type_map[&slot])
                .map(|value| (slot, value))
        })
        .collect::<Result<_>>()?;
    let ops = mir.ops.into_iter().map(|op| op.op).collect();
    Ok(Object {
        symbols: mir.symbols,
        ops,
        literals,
        kind,
        arguments: mir.arguments.clone(),
        return_slot: mir.return_slot,
        externals: mir.stash,
        name: mir.name,
        fn_id: mir.fn_id,
    })
}
