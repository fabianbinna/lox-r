use std::{cell::RefCell, collections::HashMap, iter::zip, rc::Rc};

use crate::{class::Class, environment::Environment, expr::{Expr, ExprType}, function::Function, instance::Instance, native::NativeFunction, object::Object, stmt::Stmt, token::TokenType};


pub trait Visitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_stmt(&mut self, stmt: &Stmt);
}

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<usize, usize>,
    returning: bool,
}

impl Interpreter {

    pub fn new(locals: HashMap<usize, usize>) -> Self {
        let globals = 
            Rc::new(RefCell::new(Environment::new(None)));
        let environment = globals.clone();

        globals.borrow_mut().define(NativeFunction::Clock.to_string(), Object::NativeFunction(NativeFunction::Clock));
        globals.borrow_mut().define(NativeFunction::Input.to_string(), Object::NativeFunction(NativeFunction::Input));
        globals.borrow_mut().define(NativeFunction::ReadFile.to_string(), Object::NativeFunction(NativeFunction::ReadFile));

        Interpreter {
            globals,
            environment,
            locals,
            returning: false,
        }
    }
    
    pub fn interpret(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.visit_stmt(statement);
        }
    }

    pub fn extend_locals(&mut self, locals: HashMap<usize, usize>) {
        self.locals.extend(locals);
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>, environment: Rc<RefCell<Environment>>) {
        let tmp = self.environment.clone();
        self.environment = environment;
        for statement in statements {
            self.visit_stmt(statement);
        }
        self.environment = tmp;
    }

    fn call_function(&mut self, function: Rc<RefCell<Function>>, arguments: Vec<Object>) -> Object {

        if function.borrow().name == "clock" {
            return Object::Number(16.0);
        }
       
        let arity = function.borrow().arity();
        if arguments.len() != arity {
            panic!("{}, Expected {} arguments but got {}.", 
                function.borrow().name, arity, arguments.len());
        }

        let environment = Rc::new(RefCell::new(Environment::new(Some(function.borrow().closure.clone()))));
        for (param, arg) in zip(&function.borrow().params, arguments) {
            environment.borrow_mut().define(param.clone(), arg);
        }

        self.execute_block(&function.borrow().body, environment);
        self.returning = false;
        
        // handle return nil
        if function.borrow().is_initializer {
            return function.borrow().closure.borrow().get_at(0, &String::from("this"));
        } else {
            return self.globals.borrow_mut().get_return_value();
        }
    }

    fn lookup_variable(&mut self, name: String, id: usize) -> Object {
        match self.locals.get(&id) {
            Some(distance) => self.environment.borrow().get_at(*distance, &name),
            None => self.globals.borrow().get(&name)
        }
    }
 
}

impl Visitor<Object> for Interpreter {

    fn visit_stmt(&mut self, stmt: &Stmt) {
        if self.returning {
            return;
        }

        match stmt {
            Stmt::Expression(expr) => {self.visit_expr(expr);()},
            Stmt::Print(expr) => println!("{}", self.visit_expr(expr)),
            Stmt::Var(name, initializer) => {
                let value = self.visit_expr(initializer);
                self.environment.borrow_mut().define(name.lexeme.clone(), value);
            },
            Stmt::Block(statements) => {
                let e = Rc::new(RefCell::new(Environment::new(Some(self.environment.clone()))));
                self.execute_block(statements, e);
            },
            Stmt::If(condition, then_branch, else_branch) => {
                if self.visit_expr(condition).is_truthy() {
                    self.visit_stmt(then_branch);
                } else {
                    if let Some(statement) = else_branch.as_ref() {
                        self.visit_stmt(statement)
                    }
                }
            },
            Stmt::While(condition, body) => {
                while self.visit_expr(condition).is_truthy() {
                    self.visit_stmt(body);
                }
            }
            Stmt::Function(name, parameters, body) => {
                let params = parameters.iter().map(|token| token.lexeme.clone()).collect();
                let function = Function::new(name.lexeme.clone(), params, body.clone(), self.environment.clone(), false);
                self.environment.borrow_mut().define(name.lexeme.clone(), Object::Function(Rc::new(RefCell::new(function))));
            }
            Stmt::Return(value) => {
                
                match value.as_ref() {
                    Some(value) => {
                        let return_value = self.visit_expr(value);
                        self.returning = true;
                        self.globals.borrow_mut().define("return_value".to_string(), return_value);
                    },
                    None => {
                        self.returning = true;
                        self.globals.borrow_mut().define("return_value".to_string(), Object::Nil);
                    }
                }
            },
            Stmt::Class(name, superclass, methods) => {
                let superclass = if let Some(superclass) = superclass.as_ref() {
                    if let Object::Class(class) = self.visit_expr(superclass) {
                        Some(class)
                    } else {
                        panic!("Superclass must be a class.");
                    }
                } else {
                    None
                };

                self.environment.borrow_mut().define(name.lexeme.clone(), Object::Nil);

                if let Some(sc) = superclass.as_ref() {
                    self.environment = Rc::new(RefCell::new(Environment::new(Some(self.environment.clone()))));
                    self.environment.borrow_mut().define(String::from("super"), Object::Class(sc.clone()));
                }

                let mut methods2 = HashMap::new();
                for method in methods {
                    if let Stmt::Function(name, parameters, body) = method {
                        let function = Function::new(name.lexeme.clone(), parameters.iter().map(|p|p.lexeme.clone()).collect(), body.clone(), self.environment.clone(),
                            name.lexeme == "init");
                        methods2.insert(name.lexeme.clone(), Object::Function(Rc::new(RefCell::new(function))));
                    } else {
                        panic!();
                    }
                }

                
                let class = Rc::new(RefCell::new(Class::new(name.lexeme.clone(), superclass.clone(), methods2)));

                if superclass.is_some() {
                    let parent = self.environment.borrow().parent.clone().unwrap();
                    self.environment = parent;
                }

                self.environment.borrow_mut().assign(name.lexeme.clone(), Object::Class(class));
            },
        };
    }

    fn visit_expr(&mut self, expr: &Expr) -> Object {
        match &expr.expr_type {
            ExprType::Binary(left, op, right) => {
                let left = self.visit_expr(&left);
                let right = self.visit_expr(&right);

                match op.token_type {
                    TokenType::Minus => match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(l-r),
                        (_, _) => panic!()
                    },
                    TokenType::Plus => match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(l+r),
                        (Object::String(l), Object::String(r)) => Object::String(format!("{}{}",l,r)),
                        (Object::String(l), Object::Number(r)) => Object::String(format!("{}{}",l,r)),
                        (Object::Number(l), Object::String(r)) => Object::String(format!("{}{}",l,r)),
                        (Object::String(l), Object::Nil) => Object::String(format!("{}nil",l)),
                        (Object::Nil, Object::String(r)) => Object::String(format!("nil{}",r)),
                        (a, b) => panic!("{} {}", a, b)
                    },
                    TokenType::Slash => match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(l/r),
                        (_, _) => panic!()
                    },
                    TokenType::Star => match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(l*r),
                        (_, _) => panic!()
                    },
                    TokenType::Greater => match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l>r),
                        (_, _) => panic!()
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l>=r),
                        (_, _) => panic!()
                    },
                    TokenType::Less => match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l<r),
                        (_, _) => panic!()
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(l<=r),
                        (_, _) => panic!()
                    },
                    TokenType::BangEqual => Object::Boolean(!(left == right)),
                    TokenType::EqualEqual => Object::Boolean(left == right),
                    _ => panic!()
                }
            },
            ExprType::Unary(op, expr) => {
                let right = self.visit_expr(&expr);

                match op.token_type {
                    TokenType::Minus => match right {
                            Object::Number(n) => Object::Number(-n),
                            _ => panic!("{} Operator must be a number.", op.lexeme)
                        },
                    TokenType::Bang => Object::Boolean(!right.is_truthy()),
                    _ => panic!()
                }
            },
            ExprType::Grouping(expr) => self.visit_expr(&expr),
            ExprType::Literal(value) => value.clone(),
            ExprType::Variable(name) => {
                self.lookup_variable(name.lexeme.clone(), expr.id)
            },
            ExprType::Assign(name, value) => {
                let value = self.visit_expr(&value);
                let distance = self.locals.get(&expr.id);

                match distance {
                    Some(distance) => self.environment.borrow_mut().assign_at(*distance, name.lexeme.clone(), value.clone()),
                    None => self.globals.borrow_mut().assign(name.lexeme.clone(), value.clone())
                }

                value
            },
            ExprType::Logical(left, op, right) => {
                let left = self.visit_expr(&left);

                if op.token_type == TokenType::Or {
                    if left.is_truthy() {
                        return left;
                    }
                } else {
                    if !left.is_truthy() {
                        return left;
                    }
                }

                self.visit_expr(&right)
            },
            ExprType::Call(callee, paren, arguments) => {
                let callee = self.visit_expr(&callee);

                let mut args = Vec::new();
                for argument in arguments {
                    args.push(self.visit_expr(&argument));
                }

                match callee {
                    Object::Function(function) => self.call_function(function, args),
                    Object::NativeFunction(native_function) => native_function.call(args),
                    Object::Class(class) => {
                        let instance = Object::Instance(Rc::new(RefCell::new(Instance::new(class.clone()))));
                        if let Some(initializer) = class.borrow().find_method("init") {
                            if let Object::Function(function) = initializer {
                                self.call_function(Rc::new(RefCell::new(function.borrow_mut().bind(instance.clone()))), args);
                            }
                        }
                        instance
                    }
                    _ => panic!("{} Can only call functions and classes.", paren.lexeme)
                }
            },
            ExprType::Get(expr, name) => {
                let object = self.visit_expr(&expr);
                if let Object::Instance(ref instance) = object {
                    return instance.borrow().get(&name.lexeme, &object);
                } else {
                    panic!("Only instances have properties.");
                }
            },
            ExprType::Set(object, name, value) => {
                if let Object::Instance(instance) = self.visit_expr(object) {
                    let value = self.visit_expr(value);
                    instance.borrow_mut().set(name.lexeme.clone(), value.clone());
                    return value;
                } else {
                    panic!("Only instances have fields.");
                }
            },
            ExprType::This(keyword) => {
                return self.lookup_variable(keyword.lexeme.clone(), expr.id);
            },
            ExprType::Super(_keyword, name) => {
                let distance = *self.locals.get(&expr.id).unwrap();

                let superclass = if let Object::Class(superclass) = self.environment.borrow().get_at(distance, &String::from("super")) {
                    superclass
                } else {
                    panic!();
                };

                let object = self.environment.borrow().get_at(distance - 1, &String::from("this"));
                
                let method = if let Some(method) = superclass.borrow().find_method(&name.lexeme) {
                    method
                } else {
                    panic!();
                };

                let function = if let Object::Function(function) = method {
                    function
                } else {
                    panic!();
                };
                
                return Object::Function(Rc::new(RefCell::new(function.borrow().bind(object))));
            },
        }
    }
    
}
