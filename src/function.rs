use std::{cell::RefCell, rc::Rc, fmt::Display};

use crate::{environment::Environment, stmt::Stmt, object::Object};

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Rc<Vec<Stmt>>,
    pub closure: Rc<RefCell<Environment>>,
    pub is_initializer: bool,
}

impl Function {

    pub fn new(name: String, params: Vec<String>, body: Rc<Vec<Stmt>>, closure: Rc<RefCell<Environment>>, is_initializer: bool) -> Self {
        Function {
            name,
            params, 
            body,
            closure,
            is_initializer
        }
    }

    pub fn bind(&self, instance: Object) -> Function {
        let mut environment = Environment::new(Some(self.closure.clone()));
        environment.define(String::from("this"), instance);
        Function::new(
            self.name.clone(), 
            self.params.clone(), 
            self.body.clone(), 
            Rc::new(RefCell::new(environment)),
                self.is_initializer)
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }

}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} function", self.name)
    }
}