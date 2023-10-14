use std::collections::BTreeMap;
use std::{collections::HashMap, fmt::Display};

use crate::ast::{self, BinOp, PatternTupleStruct, UnOp};
use crate::rhif::{
    AluBinary, AluUnary, AssignOp, BinaryOp, BlockId, CopyOp, ExecOp, FieldOp, FieldRefOp, IfOp,
    IndexRefOp, Member, OpCode, RefOp, RomArgument, RomOp, Slot, StructOp, TupleOp, UnaryOp,
};
use crate::Kind;
use anyhow::bail;
use anyhow::Result;

const ROOT_BLOCK: BlockId = BlockId(0);

impl From<ast::Member> for crate::rhif::Member {
    fn from(member: ast::Member) -> Self {
        match member {
            ast::Member::Named(name) => crate::rhif::Member::Named(name),
            ast::Member::Unnamed(index) => crate::rhif::Member::Unnamed(index),
        }
    }
}

pub struct Block {
    pub id: BlockId,
    pub names: HashMap<String, Slot>,
    pub ops: Vec<OpCode>,
    pub result: Slot,
    pub children: Vec<BlockId>,
    pub parent: BlockId,
}

pub struct Compiler {
    pub blocks: Vec<Block>,
    pub reg_count: usize,
    pub active_block: BlockId,
    pub types: BTreeMap<usize, Kind>,
}

impl Display for Compiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (ndx, kind) in &self.types {
            writeln!(f, "Type r{} = {:?}", ndx, kind)?;
        }
        for block in &self.blocks {
            writeln!(f, "Block {}", block.id.0)?;
            for op in &block.ops {
                writeln!(f, "  {}", op)?;
            }
        }
        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            blocks: vec![Block {
                id: ROOT_BLOCK,
                names: Default::default(),
                ops: vec![],
                result: Slot::Empty,
                children: vec![],
                parent: ROOT_BLOCK,
            }],
            reg_count: 0,
            active_block: ROOT_BLOCK,
            types: Default::default(),
        }
    }
}

impl Compiler {
    pub fn reg(&mut self) -> Slot {
        let reg = self.reg_count;
        self.reg_count += 1;
        Slot::Register(reg)
    }
    pub fn bind(&mut self, name: &str) -> Slot {
        let reg = self.reg();
        self.blocks[self.active_block.0]
            .names
            .insert(name.to_string(), reg.clone());
        reg
    }
    pub fn get_reference(&mut self, path: &str) -> Result<Slot> {
        let mut ip = self.active_block;
        loop {
            if let Some(slot) = self.blocks[ip.0].names.get(path) {
                return Ok(slot.clone());
            }
            if ip == ROOT_BLOCK {
                break;
            }
            ip = self.blocks[ip.0].parent;
        }
        bail!("Unknown path {}", path);
    }
    pub fn op(&mut self, op: OpCode) {
        self.blocks[self.active_block.0].ops.push(op);
    }
    pub fn new_block(&mut self, result: Slot) -> BlockId {
        let id = BlockId(self.blocks.len());
        self.blocks.push(Block {
            id,
            names: Default::default(),
            ops: vec![],
            result,
            children: vec![],
            parent: self.active_block,
        });
        self.blocks[self.active_block.0].children.push(id);
        self.active_block = id;
        id
    }
    fn current_block(&self) -> BlockId {
        self.active_block
    }
    fn set_block(&mut self, id: BlockId) {
        self.active_block = id;
    }
    fn block_result(&self, id: BlockId) -> Slot {
        self.blocks[id.0].result.clone()
    }

    pub fn compile(&mut self, ast: crate::ast::Block) -> Result<Slot> {
        let lhs = self.reg();
        let block_id = self.expr_block(ast, lhs.clone())?;
        self.op(OpCode::Call(block_id));
        Ok(lhs)
    }

    fn expr(&mut self, expr_: ast::Expr) -> Result<Slot> {
        match expr_ {
            ast::Expr::Lit(lit) => self.expr_literal(lit),
            ast::Expr::Unary(unop) => self.expr_unop(unop),
            ast::Expr::Binary(binop) => self.expr_binop(binop),
            ast::Expr::Block(block) => {
                let lhs = self.reg();
                let block = self.expr_block(block, lhs.clone())?;
                self.op(OpCode::Call(block));
                Ok(lhs)
            }
            ast::Expr::If(if_expr) => self.expr_if(if_expr),
            ast::Expr::Assign(assign) => self.expr_assign(assign),
            ast::Expr::Paren(paren) => self.expr(*paren),
            ast::Expr::Path(path) => Ok(self.get_reference(&path.path.join("::"))?),
            ast::Expr::Tuple(tuple) => self.tuple(&tuple),
            ast::Expr::Match(match_) => self.expr_match(match_),
            ast::Expr::Call(call) => self.expr_call(call),
            ast::Expr::Struct(structure) => self.expr_struct(structure),
            _ => todo!("expr {:?}", expr_),
        }
    }

    fn field_value(&mut self, element: ast::FieldValue) -> Result<crate::rhif::FieldValue> {
        let value = self.expr(*element.value)?;
        Ok(crate::rhif::FieldValue {
            member: element.member.into(),
            value,
        })
    }

    fn expr_struct(&mut self, structure: ast::ExprStruct) -> Result<Slot> {
        let lhs = self.reg();
        let fields = structure
            .fields
            .into_iter()
            .map(|x| self.field_value(x))
            .collect::<Result<Vec<_>>>()?;
        self.op(OpCode::Struct(StructOp {
            lhs: lhs.clone(),
            path: structure.path.path,
            fields,
            rest: None,
        }));
        Ok(lhs)
    }

    fn tuple(&mut self, tuple: &[ast::Expr]) -> Result<Slot> {
        let lhs = self.reg();
        let mut fields = vec![];
        for expr in tuple {
            fields.push(self.expr(expr.clone())?);
        }
        self.op(OpCode::Tuple(TupleOp {
            lhs: lhs.clone(),
            fields,
        }));
        Ok(lhs)
    }

    fn expr_call(&mut self, call: ast::ExprCall) -> Result<Slot> {
        let lhs = self.reg();
        let path = call.path.path;
        let args = call
            .args
            .into_iter()
            .map(|x| self.expr(x))
            .collect::<Result<_>>()?;
        self.op(OpCode::Exec(ExecOp {
            lhs: lhs.clone(),
            path,
            args,
        }));
        Ok(lhs)
    }

    fn expr_match(&mut self, expr_match: ast::ExprMatch) -> Result<Slot> {
        // Only two supported cases of match arms
        // The first is all literals and possibly a wildcard
        // The second is all enums with no literals and possibly a wildcard
        for arm in &expr_match.arms {
            if let Some(guard) = &arm.guard {
                bail!(
                    "RHDL does not currently support match guards in hardware {:?}",
                    guard
                );
            }
        }
        let all_literals_or_wild = expr_match
            .arms
            .iter()
            .all(|arm| matches!(arm.pattern, ast::Pattern::Lit(_) | ast::Pattern::Wild));
        let all_enum_or_wild = expr_match.arms.iter().all(|arm| {
            matches!(
                arm.pattern,
                ast::Pattern::Path(_)
                    | ast::Pattern::Struct(_)
                    | ast::Pattern::TupleStruct(_)
                    | ast::Pattern::Wild
            )
        });
        if !all_literals_or_wild && !all_enum_or_wild {
            bail!("RHDL currently supports only match arms with all literals or all enums (and a wildcard '_' is allowed)");
        }
        self.expr_rom(expr_match)
    }

    fn expr_rom(&mut self, expr_match: ast::ExprMatch) -> Result<Slot> {
        let lhs = self.reg();
        let target = self.expr(*expr_match.expr)?;
        let table = expr_match
            .arms
            .into_iter()
            .map(|arm| self.expr_arm(target.clone(), lhs.clone(), arm))
            .collect::<Result<_>>()?;
        self.op(OpCode::Rom(RomOp {
            lhs: lhs.clone(),
            expr: target,
            table,
        }));
        Ok(lhs)
    }

    fn expr_arm_struct(
        &mut self,
        target: Slot,
        lhs: Slot,
        structure: ast::PatternStruct,
        body: ast::Expr,
    ) -> Result<(RomArgument, BlockId)> {
        // Collect the elements of the struct that are identifiers (and not wildcards)
        // For each element of the pattern, collect the name (this is the binding) and the
        // position within the tuple.
        let bindings: Vec<(Member, String)> = structure
            .fields
            .into_iter()
            .map(|x| match *x.pat {
                ast::Pattern::Ident(ident) => Ok(Some((x.member.into(), ident.name))),
                ast::Pattern::Wild => Ok(None),
                _ => bail!("Unsupported match pattern {:?} in hardware", x),
            })
            .filter_map(|x| x.transpose())
            .collect::<Result<Vec<_>>>()?;
        // Create a new block for the struct match
        let current_id = self.current_block();
        let id = self.new_block(lhs.clone());
        // For each binding, create a new register and bind it to the name
        // Then insert an opcode into the block to extract the field from the struct
        // that is the target of the match.
        bindings.into_iter().for_each(|(member, ident)| {
            let reg = self.bind(&ident);
            self.op(OpCode::Field(FieldOp {
                lhs: reg,
                arg: target.clone(),
                member,
            }));
        });
        // Add the arm body to the block
        let expr_output = self.expr(body)?;
        // Copy the result of the arm body to the lhs
        self.op(OpCode::Copy(CopyOp {
            lhs,
            rhs: expr_output,
        }));
        self.set_block(current_id);
        Ok((RomArgument::Path(structure.path.path), id))
    }

    fn expr_arm_tuple_struct(
        &mut self,
        target: Slot,
        lhs: Slot,
        tuple: ast::PatternTupleStruct,
        body: ast::Expr,
    ) -> Result<(RomArgument, BlockId)> {
        // Collect the elements of the tuple struct that are identifiers (and not wildcards)
        // For each element of the pattern, collect the name (this is the binding) and the
        // position within the tuple.
        let bindings = tuple
            .elems
            .into_iter()
            .enumerate()
            .map(|(ndx, x)| match x {
                ast::Pattern::Ident(ident) => Ok(Some((ident.name, ndx))),
                ast::Pattern::Wild => Ok(None),
                _ => bail!("Unsupported match pattern {:?} in hardware", x),
            })
            .filter_map(|x| x.transpose())
            .collect::<Result<Vec<_>>>()?;
        // Create a new block for the tuple struct match
        let current_id = self.current_block();
        let id = self.new_block(lhs.clone());
        // For each binding, create a new register and bind it to the name
        // Then insert an opcode into the block to extract the field from the tuple
        // that is the target of the match.
        bindings.into_iter().for_each(|(ident, index)| {
            let reg = self.bind(&ident);
            self.op(OpCode::Field(FieldOp {
                lhs: reg,
                arg: target.clone(),
                member: Member::Unnamed(index as u32),
            }));
        });
        // Add the arm body to the block
        let expr_output = self.expr(body)?;
        // Copy the result of the arm body to the lhs
        self.op(OpCode::Copy(CopyOp {
            lhs,
            rhs: expr_output,
        }));
        self.set_block(current_id);
        Ok((RomArgument::Path(tuple.path.path), id))
    }

    fn expr_arm(
        &mut self,
        target: Slot,
        lhs: Slot,
        arm: ast::Arm,
    ) -> Result<(RomArgument, BlockId)> {
        match arm.pattern {
            ast::Pattern::Wild => Ok((
                RomArgument::Wild,
                self.wrap_expr_in_block(Some(arm.body), lhs)?,
            )),
            ast::Pattern::Lit(lit) => Ok((
                RomArgument::Literal(Slot::Literal(lit)),
                self.wrap_expr_in_block(Some(arm.body), lhs)?,
            )),
            ast::Pattern::Path(pat) => Ok((
                RomArgument::Path(pat.path),
                self.wrap_expr_in_block(Some(arm.body), lhs)?,
            )),
            ast::Pattern::TupleStruct(tuple) => {
                self.expr_arm_tuple_struct(target, lhs, tuple, *arm.body)
            }
            ast::Pattern::Struct(structure) => {
                self.expr_arm_struct(target, lhs, structure, *arm.body)
            }
            _ => bail!("Unsupported match pattern {:?} in hardware", arm.pattern),
        }
    }

    pub fn expr_lhs(&mut self, expr_: ast::Expr) -> Result<Slot> {
        Ok(match expr_ {
            ast::Expr::Path(path) => {
                let arg = self.get_reference(&path.path.join("::"))?;
                let lhs = self.reg();
                self.op(OpCode::Ref(RefOp {
                    lhs: lhs.clone(),
                    arg,
                }));
                lhs
            }
            ast::Expr::Field(field) => {
                let lhs = self.reg();
                let arg = self.expr_lhs(*field.expr)?;
                let member = match field.member {
                    ast::Member::Named(name) => Member::Named(name),
                    ast::Member::Unnamed(index) => Member::Unnamed(index),
                };
                self.op(OpCode::FieldRef(FieldRefOp {
                    lhs: lhs.clone(),
                    arg,
                    member,
                }));
                lhs
            }
            ast::Expr::Index(index) => {
                let lhs = self.reg();
                let arg = self.expr_lhs(*index.expr)?;
                let index = self.expr(*index.index)?;
                self.op(OpCode::IndexRef(IndexRefOp {
                    lhs: lhs.clone(),
                    arg,
                    index,
                }));
                lhs
            }
            _ => todo!("expr_lhs {:?}", expr_),
        })
    }

    pub fn stmt(&mut self, statement: ast::Stmt) -> Result<Slot> {
        match statement {
            ast::Stmt::Local(local) => {
                self.local(local)?;
                Ok(Slot::Empty)
            }
            ast::Stmt::Expr(expr_) => self.expr(expr_.expr),
            ast::Stmt::Semi(expr_) => {
                self.expr(expr_.expr)?;
                Ok(Slot::Empty)
            }
        }
    }

    fn local(&mut self, local: ast::Local) -> Result<()> {
        let rhs = local.value.map(|x| self.expr(*x)).transpose()?;
        self.let_pattern(local.pattern, rhs)?;
        Ok(())
    }

    // Some observations.
    // A type designation must appear outermost.  So if we have
    // something like:
    //  let (a, b, c) : ty = foo
    // this is legal, but
    //  let (a: ty, b: ty, c: ty) = foo
    // is not legal.
    //
    // In some ways, (a, b, c) is sort of like a shadow type declaration.
    // We could just as well devise an anonymous Tuple Struct named "Foo",
    // and then write:
    //   let Foo(a, b, c) = foo

    fn let_pattern(&mut self, pattern: ast::Pattern, rhs: Option<Slot>) -> Result<()> {
        if let ast::Pattern::Type(ty) = pattern {
            self.let_pattern_inner(*ty.pattern, Some(ty.kind), rhs)
        } else {
            self.let_pattern_inner(pattern, None, rhs)
        }
    }

    fn let_pattern_inner(
        &mut self,
        pattern: ast::Pattern,
        ty: Option<Kind>,
        rhs: Option<Slot>,
    ) -> Result<()> {
        match pattern {
            ast::Pattern::Ident(ident) => {
                let lhs = self.bind(&ident.name);
                if let Some(ty) = ty {
                    self.types.insert(lhs.reg()?, ty);
                }
                if let Some(rhs) = rhs {
                    self.op(OpCode::Copy(CopyOp {
                        lhs: lhs.clone(),
                        rhs,
                    }));
                }
                Ok(())
            }
            ast::Pattern::Tuple(tuple) => {
                for (ndx, pat) in tuple.into_iter().enumerate() {
                    let element_lhs = self.reg();
                    if let Some(rhs) = rhs.clone() {
                        self.op(OpCode::Field(FieldOp {
                            lhs: element_lhs.clone(),
                            arg: rhs.clone(),
                            member: Member::Unnamed(ndx as u32),
                        }));
                    }
                    let element_ty = if let Some(ty) = ty.as_ref() {
                        let sub_ty = ty.get_tuple_kind(ndx)?;
                        self.types.insert(element_lhs.reg()?, sub_ty.clone());
                        Some(sub_ty)
                    } else {
                        None
                    };
                    if rhs.is_some() {
                        self.let_pattern_inner(pat, element_ty, Some(element_lhs))?;
                    } else {
                        self.let_pattern_inner(pat, element_ty, None)?;
                    }
                }
                Ok(())
            }
            _ => todo!("Unsupported let pattern {:?}", pattern),
        }
    }

    pub fn expr_if(&mut self, if_expr: crate::ast::ExprIf) -> Result<Slot> {
        let lhs = self.reg();
        let cond = self.expr(*if_expr.cond)?;
        let then_branch = self.expr_block(if_expr.then_branch, lhs.clone())?;
        // Create a block containing the else part of the if expression
        let else_branch = self.wrap_expr_in_block(if_expr.else_branch, lhs.clone())?;
        self.op(OpCode::If(IfOp {
            lhs: lhs.clone(),
            cond,
            then_branch,
            else_branch,
        }));
        Ok(lhs)
    }

    // Start simple.
    // If an expression is <ExprLit> then stuff it into a Slot
    pub fn expr_literal(&mut self, lit: crate::ast::ExprLit) -> Result<Slot> {
        Ok(Slot::Literal(lit))
    }

    pub fn expr_assign(&mut self, assign: ast::ExprAssign) -> Result<Slot> {
        let lhs = self.expr_lhs(*assign.lhs)?;
        let rhs = self.expr(*assign.rhs)?;
        self.op(OpCode::Assign(AssignOp { lhs, rhs }));
        Ok(Slot::Empty)
    }

    pub fn expr_unop(&mut self, unop: crate::ast::ExprUnary) -> Result<Slot> {
        let arg = self.expr(*unop.expr)?;
        let dest = self.reg();
        let alu = match unop.op {
            UnOp::Neg => AluUnary::Neg,
            UnOp::Not => AluUnary::Not,
        };
        self.op(OpCode::Unary(UnaryOp {
            op: alu,
            lhs: dest.clone(),
            arg1: arg,
        }));
        Ok(dest)
    }

    pub fn expr_binop(&mut self, binop: crate::ast::ExprBinary) -> Result<Slot> {
        // Allocate a register for the output
        let lhs = self.expr(*binop.lhs)?;
        let rhs = self.expr(*binop.rhs)?;
        let self_assign = matches!(
            binop.op,
            BinOp::AddAssign
                | BinOp::SubAssign
                | BinOp::MulAssign
                | BinOp::BitXorAssign
                | BinOp::BitAndAssign
                | BinOp::ShlAssign
                | BinOp::BitOrAssign
                | BinOp::ShrAssign
        );
        let alu = match binop.op {
            BinOp::Add | BinOp::AddAssign => AluBinary::Add,
            BinOp::Sub | BinOp::SubAssign => AluBinary::Sub,
            BinOp::Mul | BinOp::MulAssign => AluBinary::Mul,
            BinOp::And => AluBinary::And,
            BinOp::Or => AluBinary::Or,
            BinOp::BitXor | BinOp::BitXorAssign => AluBinary::BitXor,
            BinOp::BitAnd | BinOp::BitAndAssign => AluBinary::BitAnd,
            BinOp::BitOr | BinOp::BitOrAssign => AluBinary::BitOr,
            BinOp::Shl | BinOp::ShlAssign => AluBinary::Shl,
            BinOp::Shr | BinOp::ShrAssign => AluBinary::Shr,
            BinOp::Eq => AluBinary::Eq,
            BinOp::Lt => AluBinary::Lt,
            BinOp::Le => AluBinary::Le,
            BinOp::Ne => AluBinary::Ne,
            BinOp::Ge => AluBinary::Ge,
            BinOp::Gt => AluBinary::Gt,
        };
        let dest = if self_assign { lhs.clone() } else { self.reg() };
        self.op(OpCode::Binary(BinaryOp {
            op: alu,
            lhs: dest.clone(),
            arg1: lhs,
            arg2: rhs,
        }));
        Ok(dest)
    }

    // Add a set of statements to the current block with capturing of lhs for the last
    // statement in the list.
    fn expr_block_inner(&mut self, statements: &[ast::Stmt], lhs: Slot) -> Result<()> {
        let statement_count = statements.len();
        for (ndx, stmt) in statements.iter().enumerate() {
            if ndx == statement_count - 1 {
                let rhs = self.stmt(stmt.clone())?;
                self.op(OpCode::Copy(CopyOp {
                    lhs: lhs.clone(),
                    rhs,
                }));
            } else {
                self.stmt(stmt.clone())?;
            }
        }
        Ok(())
    }

    pub fn expr_block(&mut self, block: crate::ast::Block, lhs: Slot) -> Result<BlockId> {
        let statement_count = block.0.len();
        let current_block = self.current_block();
        let id = self.new_block(lhs.clone());
        self.expr_block_inner(&block.0, lhs.clone())?;
        self.set_block(current_block);
        Ok(id)
    }

    // There are places where Rust allows either an expression or a block.  For example in the
    // else branch of an if, or in each arm of a match.  Because these have different behaviors
    // in RHIF (a block is executed when jumped to, while an expression is immediate), we need
    // to be able to "wrap" an expression into a block so that it can be invoked conditionally.
    // As a special case, if the expression is empty (None), we create an empty block.
    pub fn wrap_expr_in_block(
        &mut self,
        expr: Option<Box<crate::ast::Expr>>,
        lhs: Slot,
    ) -> Result<BlockId> {
        let block = if let Some(expr) = expr {
            ast::Block(vec![ast::Stmt::Expr(ast::ExprStatement {
                expr: *expr,
                text: None,
            })])
        } else {
            ast::Block(vec![])
        };
        self.expr_block(block, lhs)
    }
}
