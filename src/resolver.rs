use std::collections::HashMap;

use crate::{expr::{Expr, ExprType}, token::Token, stmt::Stmt};

pub fn resolve(statements: &Vec<Stmt>) -> HashMap<usize, usize> {
    let mut resolver = Resolver::new();
    resolver.resolve_statements(&statements);
    resolver.locals
}

#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    Function,
    Initializer,
    Method,
    None
}

#[derive(Clone, Copy, PartialEq)]
enum ClassType {
    None,
    Class,
    Subclass
}

struct Resolver {
    scope: Vec<HashMap<String, bool>>,
    locals: HashMap<usize, usize>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver {
    fn new() -> Self {
        Resolver {
            scope: vec![],
            locals: HashMap::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    fn resolve_statements(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.resolve_statement(statement);
        }
    }

    fn resolve_statement(&mut self, statement: &Stmt) {
        self.visit_stmt(statement);
    }

    fn resolve_expression(&mut self, expression: &Expr) {
        self.visit_expr(expression);
    }

    fn begin_scope(&mut self) {
        self.scope.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scope.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scope.is_empty() {
            return;
        }

        if self.scope.last().unwrap().contains_key(&name.lexeme) {
            panic!("{}, Already a variable with this name in this scope.", &name.lexeme);
        }

        self.scope.last_mut().unwrap().insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scope.is_empty() {
            return;
        }

        self.scope.last_mut().unwrap().insert(name.lexeme.clone(), true); // check if exists?
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scope.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.locals.insert(expr.id, self.scope.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_function(&mut self, parameters: &Vec<Token>, body: &Vec<Stmt>, function_type: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = function_type;
        self.begin_scope();
        for parameter in parameters {
            self.declare(parameter);
            self.define(parameter);
        }
        self.resolve_statements(body);
        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve_statements(statements);
                self.end_scope();
            },
            Stmt::Var(name, initializer) => {
                self.declare(name);
                let _ = self.visit_expr(initializer);
                self.define(name);
            },
            Stmt::Function(name, parameters, body) => {
                self.declare(name);
                self.define(name);
                self.resolve_function(parameters, body, FunctionType::Function);
            },
            Stmt::Expression(expr) => self.resolve_expression(expr),
            Stmt::If(condition, then_branch, else_branch) => {
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                match else_branch.as_ref() {
                    Some(stmt) => self.resolve_statement(stmt),
                    None => {},
                }
            },
            Stmt::Print(expr) => self.resolve_expression(expr),
            Stmt::Return(value) => {
                if self.current_function == FunctionType::None {
                    panic!("Can't return from top-level code.");
                }

                match value.as_ref() {
                    Some(value) => {
                        if self.current_function == FunctionType::Initializer {
                            panic!("Can't return a value from an initializer.");
                        }
                        self.resolve_expression(value);
                    },
                    None => {},
                }
            },
            Stmt::While(condition, body) => {
                self.resolve_expression(condition);
                self.resolve_statement(body);
            }
            Stmt::Class(name, superclass, methods) => {
                let enclosng_class = self.current_class;
                self.current_class = ClassType::Class;

                self.declare(name);
                self.define(name);

                if let Some(superclass) = superclass.as_ref() {
                    if let ExprType::Variable(variable) = &superclass.expr_type {
                        self.begin_scope();
                        self.scope.last_mut().unwrap().insert(String::from("super"), true);
                        if variable.lexeme == name.lexeme {
                            panic!("A class can't inherit from itself.");
                        }
                        self.current_class = ClassType::Subclass;
                        self.resolve_expression(superclass);
                    } else {
                        panic!("Superclass must be a class.");
                    }
                }

                self.begin_scope();
                self.scope.last_mut().unwrap().insert(String::from("this"), true);

                for method in methods {
                    let mut declaration = FunctionType::Method;
                    if let Stmt::Function(name, parameters, body) = method {
                        if name.lexeme == "init" {
                            declaration = FunctionType::Initializer;
                        }
                        self.resolve_function(parameters, body, declaration);
                    } else {
                        panic!();
                    }
                }

                self.end_scope();

                if let Some(superclass) = superclass.as_ref() {
                    if let ExprType::Variable(_) = &superclass.expr_type {
                        self.end_scope();
                    }
                }

                self.current_class = enclosng_class;
            },
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.expr_type {
            ExprType::Variable(name) => {
                if let Some(scope) = self.scope.last() {
                    if let Some(var) = scope.get(&name.lexeme) {
                        if var == &false {
                            panic!("{} Can't read local variable in its own initializer.", name.lexeme);
                        }
                    }
                }

                self.resolve_local(expr, name.as_ref())
            },
            ExprType::Assign(name, value) => {
                self.visit_expr(&value);
                self.resolve_local(expr, &name)
            },
            ExprType::Binary(left, _op, right) => {
                self.resolve_expression(&left);
                self.resolve_expression(&right);
            },
            ExprType::Call(callee, _paren, arguments) => {
                self.resolve_expression(&callee);
                for argument in arguments {
                    self.resolve_expression(&argument);
                }
            },
            ExprType::Grouping(expr) => self.resolve_expression(&expr),
            ExprType::Logical(left, _op, right) => {
                self.resolve_expression(&left);
                self.resolve_expression(&right);
            },
            ExprType::Unary(_op, expr) => self.resolve_expression(&expr),
            ExprType::Get(expr, _name) => {
                self.resolve_expression(expr);
            },
            ExprType::Set(object, _name, value) => {
                self.resolve_expression(value);
                self.resolve_expression(object);
            },
            ExprType::Super(keyword, _) => {
                if self.current_class == ClassType::None {
                    panic!("Can't use 'super' outside of a class.");
                } else if self.current_class != ClassType::Subclass {
                    panic!("Can't use 'super' in a class with no superclass.");
                }

                self.resolve_local(expr, keyword);
            }
            ExprType::This(keyword) => {
                if self.current_class == ClassType::None {
                    panic!("Can't use 'this' outside of a class.");
                }

                self.resolve_local(expr, keyword);
            },
            _ => {}
        }
    }

}