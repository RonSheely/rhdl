// A visitor pattern for the ast
// To use this, impl Visitor on a data structure, and then pass it to the appropriate top
// level walk_mut_ function.
// This version allows you to mutate the ast as you traverse it.

use crate::ast::*;
use anyhow::Result;

pub trait VisitorMut {
    fn visit_block(&mut self, _block: &mut Block) -> Result<()> {
        Ok(())
    }
    fn visit_stmt(&mut self, _stmt: &mut Stmt) -> Result<()> {
        Ok(())
    }
    fn visit_local(&mut self, _local: &mut Local) -> Result<()> {
        Ok(())
    }
    fn visit_pat(&mut self, _pat: &mut Pat) -> Result<()> {
        Ok(())
    }
    fn visit_path_segment(&mut self, _path_segment: &mut PathSegment) -> Result<()> {
        Ok(())
    }
    fn visit_path(&mut self, _path: &mut Path) -> Result<()> {
        Ok(())
    }
    fn visit_pat_ident(&mut self, _pat_ident: &mut PatIdent) -> Result<()> {
        Ok(())
    }
    fn visit_pat_tuple(&mut self, _pat_tuple: &mut PatTuple) -> Result<()> {
        Ok(())
    }
    fn visit_pat_tuple_struct(&mut self, _pat_tuple_struct: &mut PatTupleStruct) -> Result<()> {
        Ok(())
    }
    fn visit_pat_lit(&mut self, _pat_lit: &mut PatLit) -> Result<()> {
        Ok(())
    }
    fn visit_pat_or(&mut self, _pat_or: &mut PatOr) -> Result<()> {
        Ok(())
    }
    fn visit_pat_paren(&mut self, _pat_paren: &mut PatParen) -> Result<()> {
        Ok(())
    }
    fn visit_pat_path(&mut self, _pat_path: &mut PatPath) -> Result<()> {
        Ok(())
    }
    fn visit_pat_struct(&mut self, _pat_struct: &mut PatStruct) -> Result<()> {
        Ok(())
    }
    fn visit_pat_type(&mut self, _pat_type: &mut PatType) -> Result<()> {
        Ok(())
    }
    fn visit_pat_wild(&mut self) -> Result<()> {
        Ok(())
    }
    fn visit_expr(&mut self, _expr: &mut Expr) -> Result<()> {
        Ok(())
    }
    fn visit_expr_binary(&mut self, _expr_binary: &mut ExprBinary) -> Result<()> {
        Ok(())
    }
    fn visit_expr_unary(&mut self, _expr_unary: &mut ExprUnary) -> Result<()> {
        Ok(())
    }
    fn visit_expr_match(&mut self, _expr_match: &mut ExprMatch) -> Result<()> {
        Ok(())
    }
    fn visit_expr_ret(&mut self, _expr_return: &mut ExprRet) -> Result<()> {
        Ok(())
    }
    fn visit_expr_if(&mut self, _expr_if: &mut ExprIf) -> Result<()> {
        Ok(())
    }
    fn visit_expr_index(&mut self, _expr_index: &mut ExprIndex) -> Result<()> {
        Ok(())
    }
    fn visit_expr_paren(&mut self, _expr_paren: &mut ExprParen) -> Result<()> {
        Ok(())
    }
    fn visit_expr_tuple(&mut self, _expr_tuple: &mut ExprTuple) -> Result<()> {
        Ok(())
    }
    fn visit_expr_for_loop(&mut self, _expr_for_loop: &mut ExprForLoop) -> Result<()> {
        Ok(())
    }
    fn visit_expr_assign(&mut self, _expr_assign: &mut ExprAssign) -> Result<()> {
        Ok(())
    }
    fn visit_expr_group(&mut self, _expr_group: &mut ExprGroup) -> Result<()> {
        Ok(())
    }
    fn visit_expr_field(&mut self, _expr_field: &mut ExprField) -> Result<()> {
        Ok(())
    }
    fn visit_expr_block(&mut self, _expr_block: &mut ExprBlock) -> Result<()> {
        Ok(())
    }
    fn visit_expr_array(&mut self, _expr_array: &mut ExprArray) -> Result<()> {
        Ok(())
    }
    fn visit_expr_range(&mut self, _expr_range: &mut ExprRange) -> Result<()> {
        Ok(())
    }
    fn visit_expr_path(&mut self, _expr_path: &mut ExprPath) -> Result<()> {
        Ok(())
    }
    fn visit_expr_let(&mut self, _expr_let: &mut ExprLet) -> Result<()> {
        Ok(())
    }
    fn visit_expr_repeat(&mut self, _expr_repeat: &mut ExprRepeat) -> Result<()> {
        Ok(())
    }
    fn visit_expr_struct(&mut self, _expr_struct: &mut ExprStruct) -> Result<()> {
        Ok(())
    }
    fn visit_expr_call(&mut self, _expr_call: &mut ExprCall) -> Result<()> {
        Ok(())
    }
    fn visit_expr_method_call(&mut self, _expr_method_call: &mut ExprMethodCall) -> Result<()> {
        Ok(())
    }
    fn visit_match_arm(&mut self, _arm: &mut Arm) -> Result<()> {
        Ok(())
    }
    fn visit_expr_lit(&mut self, _lit: &mut ExprLit) -> Result<()> {
        Ok(())
    }
    fn visit_field_value(&mut self, _field_value: &mut FieldValue) -> Result<()> {
        Ok(())
    }
    fn visit_field_pat(&mut self, _field_pat: &mut FieldPat) -> Result<()> {
        Ok(())
    }
}

pub fn walk_mut_block(visitor: &mut dyn VisitorMut, block: &mut Block) -> Result<()> {
    visitor.visit_block(block)?;
    for stmt in &mut block.stmts {
        walk_mut_stmt(visitor, stmt)?;
    }
    Ok(())
}

pub fn walk_mut_stmt(visitor: &mut dyn VisitorMut, stmt: &mut Stmt) -> Result<()> {
    visitor.visit_stmt(stmt)?;
    match &mut stmt.kind {
        StmtKind::Local(local) => {
            walk_mut_local(visitor, local)?;
        }
        StmtKind::Expr(expr) => {
            walk_mut_expr(visitor, expr)?;
        }
        StmtKind::Semi(expr) => {
            walk_mut_expr(visitor, expr)?;
        }
    }
    Ok(())
}

pub fn walk_mut_local(visitor: &mut dyn VisitorMut, local: &mut Local) -> Result<()> {
    visitor.visit_local(local)?;
    walk_mut_pat(visitor, &mut local.pat)?;
    if let Some(init) = &mut local.init {
        walk_mut_expr(visitor, init)?;
    }
    Ok(())
}

pub fn walk_mut_pat_ident(visitor: &mut dyn VisitorMut, pat_ident: &mut PatIdent) -> Result<()> {
    visitor.visit_pat_ident(pat_ident)?;
    Ok(())
}

pub fn walk_mut_pat_tuple(visitor: &mut dyn VisitorMut, pat_tuple: &mut PatTuple) -> Result<()> {
    visitor.visit_pat_tuple(pat_tuple)?;
    for pat in &mut pat_tuple.elements {
        walk_mut_pat(visitor, pat)?;
    }
    Ok(())
}

pub fn walk_mut_pat_tuple_struct(
    visitor: &mut dyn VisitorMut,
    pat_tuple_struct: &mut PatTupleStruct,
) -> Result<()> {
    visitor.visit_pat_tuple_struct(pat_tuple_struct)?;
    walk_mut_path(visitor, &mut pat_tuple_struct.path)?;
    for pat in &mut pat_tuple_struct.elems {
        walk_mut_pat(visitor, pat)?;
    }
    Ok(())
}

pub fn walk_mut_pat_lit(visitor: &mut dyn VisitorMut, pat_lit: &mut PatLit) -> Result<()> {
    visitor.visit_pat_lit(pat_lit)?;
    Ok(())
}

pub fn walk_mut_pat_or(visitor: &mut dyn VisitorMut, pat_or: &mut PatOr) -> Result<()> {
    visitor.visit_pat_or(pat_or)?;
    for pat in &mut pat_or.segments {
        walk_mut_pat(visitor, pat)?;
    }
    Ok(())
}

pub fn walk_mut_pat_paren(visitor: &mut dyn VisitorMut, pat_paren: &mut PatParen) -> Result<()> {
    visitor.visit_pat_paren(pat_paren)?;
    walk_mut_pat(visitor, &mut pat_paren.pat)?;
    Ok(())
}

pub fn walk_mut_pat_path(visitor: &mut dyn VisitorMut, pat_path: &mut PatPath) -> Result<()> {
    visitor.visit_pat_path(pat_path)?;
    walk_mut_path(visitor, &mut pat_path.path)?;
    Ok(())
}

pub fn walk_mut_pat_struct(visitor: &mut dyn VisitorMut, pat_struct: &mut PatStruct) -> Result<()> {
    visitor.visit_pat_struct(pat_struct)?;
    walk_mut_path(visitor, &mut pat_struct.path)?;
    for field in &mut pat_struct.fields {
        walk_mut_field_pat(visitor, field)?;
    }
    Ok(())
}

pub fn walk_mut_field_pat(visitor: &mut dyn VisitorMut, field_pat: &mut FieldPat) -> Result<()> {
    visitor.visit_field_pat(field_pat)?;
    walk_mut_pat(visitor, &mut field_pat.pat)?;
    Ok(())
}

pub fn walk_mut_pat_type(visitor: &mut dyn VisitorMut, pat_type: &mut PatType) -> Result<()> {
    visitor.visit_pat_type(pat_type)?;
    walk_mut_pat(visitor, &mut pat_type.pat)?;
    Ok(())
}

pub fn walk_mut_pat_wild(visitor: &mut dyn VisitorMut) -> Result<()> {
    visitor.visit_pat_wild()?;
    Ok(())
}

pub fn walk_mut_pat(visitor: &mut dyn VisitorMut, pat: &mut Pat) -> Result<()> {
    visitor.visit_pat(pat)?;
    match &mut pat.kind {
        PatKind::Ident(pat) => {
            walk_mut_pat_ident(visitor, pat)?;
        }
        PatKind::Tuple(tuple) => {
            walk_mut_pat_tuple(visitor, tuple)?;
        }
        PatKind::TupleStruct(tuple_struct) => {
            walk_mut_pat_tuple_struct(visitor, tuple_struct)?;
        }
        PatKind::Lit(lit) => {
            walk_mut_pat_lit(visitor, lit)?;
        }
        PatKind::Or(pat_or) => {
            walk_mut_pat_or(visitor, pat_or)?;
        }
        PatKind::Paren(pat_paren) => {
            walk_mut_pat_paren(visitor, pat_paren)?;
        }
        PatKind::Path(path) => {
            walk_mut_pat_path(visitor, path)?;
        }
        PatKind::Struct(structure) => {
            walk_mut_pat_struct(visitor, structure)?;
        }
        PatKind::Type(pat_type) => {
            walk_mut_pat_type(visitor, pat_type)?;
        }
        PatKind::Wild => {
            walk_mut_pat_wild(visitor)?;
        }
    }
    Ok(())
}

pub fn walk_mut_expr_binary(
    visitor: &mut dyn VisitorMut,
    expr_binary: &mut ExprBinary,
) -> Result<()> {
    visitor.visit_expr_binary(expr_binary)?;
    walk_mut_expr(visitor, &mut expr_binary.lhs)?;
    walk_mut_expr(visitor, &mut expr_binary.rhs)?;
    Ok(())
}

pub fn walk_mut_expr_unary(visitor: &mut dyn VisitorMut, expr_unary: &mut ExprUnary) -> Result<()> {
    visitor.visit_expr_unary(expr_unary)?;
    walk_mut_expr(visitor, &mut expr_unary.expr)?;
    Ok(())
}

pub fn walk_mut_expr_match(visitor: &mut dyn VisitorMut, expr_match: &mut ExprMatch) -> Result<()> {
    visitor.visit_expr_match(expr_match)?;
    walk_mut_expr(visitor, &mut expr_match.expr)?;
    for arm in &mut expr_match.arms {
        walk_mut_match_arm(visitor, arm)?;
    }
    Ok(())
}

pub fn walk_mut_match_arm(visitor: &mut dyn VisitorMut, arm: &mut Arm) -> Result<()> {
    visitor.visit_match_arm(arm)?;
    walk_mut_pat(visitor, &mut arm.pattern)?;
    if let Some(guard) = &mut arm.guard {
        walk_mut_expr(visitor, guard)?;
    }
    walk_mut_expr(visitor, &mut arm.body)?;
    Ok(())
}

pub fn walk_mut_expr_ret(visitor: &mut dyn VisitorMut, expr_return: &mut ExprRet) -> Result<()> {
    visitor.visit_expr_ret(expr_return)?;
    if let Some(expr) = &mut expr_return.expr {
        walk_mut_expr(visitor, expr)?;
    }
    Ok(())
}

pub fn walk_mut_expr_if(visitor: &mut dyn VisitorMut, expr_if: &mut ExprIf) -> Result<()> {
    visitor.visit_expr_if(expr_if)?;
    walk_mut_expr(visitor, &mut expr_if.cond)?;
    walk_mut_block(visitor, &mut expr_if.then_branch)?;
    if let Some(else_branch) = &mut expr_if.else_branch {
        walk_mut_expr(visitor, else_branch)?;
    }
    Ok(())
}

pub fn walk_mut_expr_index(visitor: &mut dyn VisitorMut, expr_index: &mut ExprIndex) -> Result<()> {
    visitor.visit_expr_index(expr_index)?;
    walk_mut_expr(visitor, &mut expr_index.expr)?;
    walk_mut_expr(visitor, &mut expr_index.index)?;
    Ok(())
}

pub fn walk_mut_expr_lit(visitor: &mut dyn VisitorMut, lit: &mut ExprLit) -> Result<()> {
    visitor.visit_expr_lit(lit)?;
    Ok(())
}

pub fn walk_mut_expr_paren(visitor: &mut dyn VisitorMut, expr_paren: &mut ExprParen) -> Result<()> {
    visitor.visit_expr_paren(expr_paren)?;
    walk_mut_expr(visitor, &mut expr_paren.expr)?;
    Ok(())
}

pub fn walk_mut_expr_tuple(visitor: &mut dyn VisitorMut, expr_tuple: &mut ExprTuple) -> Result<()> {
    visitor.visit_expr_tuple(expr_tuple)?;
    for expr in &mut expr_tuple.elements {
        walk_mut_expr(visitor, expr)?;
    }
    Ok(())
}

pub fn walk_mut_expr_for_loop(
    visitor: &mut dyn VisitorMut,
    expr_for_loop: &mut ExprForLoop,
) -> Result<()> {
    visitor.visit_expr_for_loop(expr_for_loop)?;
    walk_mut_pat(visitor, &mut expr_for_loop.pat)?;
    walk_mut_expr(visitor, &mut expr_for_loop.expr)?;
    walk_mut_block(visitor, &mut expr_for_loop.body)?;
    Ok(())
}

pub fn walk_mut_expr_assign(
    visitor: &mut dyn VisitorMut,
    expr_assign: &mut ExprAssign,
) -> Result<()> {
    visitor.visit_expr_assign(expr_assign)?;
    walk_mut_expr(visitor, &mut expr_assign.lhs)?;
    walk_mut_expr(visitor, &mut expr_assign.rhs)?;
    Ok(())
}

pub fn walk_mut_expr_group(visitor: &mut dyn VisitorMut, expr_group: &mut ExprGroup) -> Result<()> {
    visitor.visit_expr_group(expr_group)?;
    walk_mut_expr(visitor, &mut expr_group.expr)?;
    Ok(())
}

pub fn walk_mut_expr_field(visitor: &mut dyn VisitorMut, expr_field: &mut ExprField) -> Result<()> {
    visitor.visit_expr_field(expr_field)?;
    walk_mut_expr(visitor, &mut expr_field.expr)?;
    Ok(())
}

pub fn walk_mut_expr_block(visitor: &mut dyn VisitorMut, expr_block: &mut ExprBlock) -> Result<()> {
    visitor.visit_expr_block(expr_block)?;
    walk_mut_block(visitor, &mut expr_block.block)?;
    Ok(())
}

pub fn walk_mut_expr_array(visitor: &mut dyn VisitorMut, expr_array: &mut ExprArray) -> Result<()> {
    visitor.visit_expr_array(expr_array)?;
    for expr in &mut expr_array.elems {
        walk_mut_expr(visitor, expr)?;
    }
    Ok(())
}

pub fn walk_mut_expr_range(visitor: &mut dyn VisitorMut, expr_range: &mut ExprRange) -> Result<()> {
    visitor.visit_expr_range(expr_range)?;
    if let Some(start) = &mut expr_range.start {
        walk_mut_expr(visitor, start)?;
    }
    if let Some(end) = &mut expr_range.end {
        walk_mut_expr(visitor, end)?;
    }
    Ok(())
}

pub fn walk_mut_expr_path(visitor: &mut dyn VisitorMut, expr_path: &mut ExprPath) -> Result<()> {
    visitor.visit_expr_path(expr_path)?;
    walk_mut_path(visitor, &mut expr_path.path)?;
    Ok(())
}

pub fn walk_mut_expr_let(visitor: &mut dyn VisitorMut, expr_let: &mut ExprLet) -> Result<()> {
    visitor.visit_expr_let(expr_let)?;
    walk_mut_pat(visitor, &mut expr_let.pattern)?;
    walk_mut_expr(visitor, &mut expr_let.value)?;
    walk_mut_expr(visitor, &mut expr_let.body)?;
    Ok(())
}

pub fn walk_mut_expr_repeat(
    visitor: &mut dyn VisitorMut,
    expr_repeat: &mut ExprRepeat,
) -> Result<()> {
    visitor.visit_expr_repeat(expr_repeat)?;
    walk_mut_expr(visitor, &mut expr_repeat.value)?;
    walk_mut_expr(visitor, &mut expr_repeat.len)?;
    Ok(())
}

pub fn walk_mut_expr_struct(
    visitor: &mut dyn VisitorMut,
    expr_struct: &mut ExprStruct,
) -> Result<()> {
    visitor.visit_expr_struct(expr_struct)?;
    walk_mut_path(visitor, &mut expr_struct.path)?;
    for field in &mut expr_struct.fields {
        walk_mut_field_value(visitor, field)?;
    }
    if let Some(rest) = &mut expr_struct.rest {
        walk_mut_expr(visitor, rest)?;
    }
    Ok(())
}

pub fn walk_mut_expr_call(visitor: &mut dyn VisitorMut, expr_call: &mut ExprCall) -> Result<()> {
    visitor.visit_expr_call(expr_call)?;
    walk_mut_path(visitor, &mut expr_call.path)?;
    for arg in &mut expr_call.args {
        walk_mut_expr(visitor, arg)?;
    }
    Ok(())
}

pub fn walk_mut_expr_method_call(
    visitor: &mut dyn VisitorMut,
    expr_method_call: &mut ExprMethodCall,
) -> Result<()> {
    visitor.visit_expr_method_call(expr_method_call)?;
    walk_mut_expr(visitor, &mut expr_method_call.receiver)?;
    for arg in &mut expr_method_call.args {
        walk_mut_expr(visitor, arg)?;
    }
    Ok(())
}

pub fn walk_mut_expr(visitor: &mut dyn VisitorMut, expr: &mut Expr) -> Result<()> {
    visitor.visit_expr(expr)?;
    match &mut expr.kind {
        ExprKind::Binary(expr) => walk_mut_expr_binary(visitor, expr),
        ExprKind::Unary(expr) => walk_mut_expr_unary(visitor, expr),
        ExprKind::Match(expr) => walk_mut_expr_match(visitor, expr),
        ExprKind::Ret(expr) => walk_mut_expr_ret(visitor, expr),
        ExprKind::If(expr) => walk_mut_expr_if(visitor, expr),
        ExprKind::Index(expr) => walk_mut_expr_index(visitor, expr),
        ExprKind::Lit(expr) => walk_mut_expr_lit(visitor, expr),
        ExprKind::Paren(expr) => walk_mut_expr_paren(visitor, expr),
        ExprKind::Tuple(expr) => walk_mut_expr_tuple(visitor, expr),
        ExprKind::ForLoop(expr) => walk_mut_expr_for_loop(visitor, expr),
        ExprKind::Assign(expr) => walk_mut_expr_assign(visitor, expr),
        ExprKind::Group(expr) => walk_mut_expr_group(visitor, expr),
        ExprKind::Field(expr) => walk_mut_expr_field(visitor, expr),
        ExprKind::Block(expr) => walk_mut_expr_block(visitor, expr),
        ExprKind::Array(expr) => walk_mut_expr_array(visitor, expr),
        ExprKind::Range(expr) => walk_mut_expr_range(visitor, expr),
        ExprKind::Path(expr) => walk_mut_expr_path(visitor, expr),
        ExprKind::Let(expr) => walk_mut_expr_let(visitor, expr),
        ExprKind::Repeat(expr) => walk_mut_expr_repeat(visitor, expr),
        ExprKind::Struct(expr) => walk_mut_expr_struct(visitor, expr),
        ExprKind::Call(expr) => walk_mut_expr_call(visitor, expr),
        ExprKind::MethodCall(expr) => walk_mut_expr_method_call(visitor, expr),
    }
}

pub fn walk_mut_path(visitor: &mut dyn VisitorMut, path: &mut Path) -> Result<()> {
    visitor.visit_path(path)?;
    for segment in &mut path.segments {
        walk_mut_path_segment(visitor, segment)?;
    }
    Ok(())
}

pub fn walk_mut_path_segment(
    visitor: &mut dyn VisitorMut,
    path_segment: &mut PathSegment,
) -> Result<()> {
    visitor.visit_path_segment(path_segment)?;
    Ok(())
}

pub fn walk_mut_field_value(
    visitor: &mut dyn VisitorMut,
    field_value: &mut FieldValue,
) -> Result<()> {
    visitor.visit_field_value(field_value)?;
    walk_mut_expr(visitor, &mut field_value.value)?;
    Ok(())
}