use std::rc::Rc;

use crate::{token::Token, expr::Expr};

#[derive(Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Class(Box<Token>, Box<Option<Expr>>, Vec<Stmt>),
    Expression(Box<Expr>),
    Function(Box<Token>, Vec<Token>, Rc<Vec<Stmt>>),
    If(Box<Expr>, Box<Stmt>, Box<Option<Stmt>>),
    Print(Box<Expr>),
    Return(Box<Option<Expr>>),
    Var(Box<Token>, Box<Expr>),
    While(Box<Expr>, Box<Stmt>)
}