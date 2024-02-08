use crate::{token::Token, object::Object};


#[derive(Clone, PartialEq)]
pub struct Expr {
    pub id: usize,
    pub expr_type: ExprType
}

impl Expr {
    pub fn new(expr_type: ExprType) -> Self {
        let id = rand::random::<usize>();
        Expr {
            id,
            expr_type
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ExprType {
    Assign(Box<Token>, Box<Expr>),
    Binary(Box<Expr>, Box<Token>, Box<Expr>),
    Call(Box<Expr>, Box<Token>, Vec<Expr>),
    Get(Box<Expr>, Box<Token>),
    Grouping(Box<Expr>),
    Literal(Object),
    Logical(Box<Expr>, Box<Token>, Box<Expr>),
    Set(Box<Expr>, Box<Token>, Box<Expr>),
    Super(Box<Token>, Box<Token>),
    This(Box<Token>),
    Unary(Box<Token>, Box<Expr>),
    Variable(Box<Token>),
}
