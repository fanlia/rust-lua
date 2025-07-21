use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::Vm;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Table(Rc<std::cell::RefCell<HashMap<Value, Value>>>),
    Function(Function),
}

#[derive(Debug, Clone)]
pub enum Function {
    Native(fn(&mut Vm, Vec<Value>) -> Value),
    UserDefined {
        parameters: Vec<String>,
        body: Vec<crate::parser::Stmt>,
        closure: Rc<std::cell::RefCell<HashMap<String, Value>>>,
    },
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Native(f1), Function::Native(f2)) => std::ptr::eq(f1, f2),
            (Function::UserDefined { .. }, Function::UserDefined { .. }) => false, // User-defined functions are never equal
            _ => false,
        }
    }
}

impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => 0.hash(state),
            Value::Boolean(b) => {
                1.hash(state);
                b.hash(state);
            }
            Value::Number(n) => {
                2.hash(state);
                n.to_bits().hash(state);
            }
            Value::String(s) => {
                3.hash(state);
                s.hash(state);
            }
            Value::Table(_) => 4.hash(state),
            Value::Function(_) => 5.hash(state),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Table(_) => write!(f, "table"),
            Value::Function(_) => write!(f, "function"),
        }
    }
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Boolean(false) => false,
            _ => true,
        }
    }

    pub fn to_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::String(s) => s.parse().ok(),
            Value::Boolean(true) => Some(1.0),
            Value::Boolean(false) => Some(0.0),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Table(_) => "table".to_string(),
            Value::Function(_) => "function".to_string(),
        }
    }

    pub fn new_table() -> Self {
        Value::Table(Rc::new(std::cell::RefCell::new(HashMap::new())))
    }

    pub fn add(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Number(a + b),
            _ => Value::Nil,
        }
    }

    pub fn subtract(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Number(a - b),
            _ => Value::Nil,
        }
    }

    pub fn multiply(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Number(a * b),
            _ => Value::Nil,
        }
    }

    pub fn divide(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Number(a / b),
            _ => Value::Nil,
        }
    }

    pub fn power(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Number(a.powf(b)),
            _ => Value::Nil,
        }
    }

    pub fn modulo(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Number(a % b),
            _ => Value::Nil,
        }
    }

    pub fn equal(&self, other: &Value) -> Value {
        Value::Boolean(self == other)
    }

    pub fn not_equal(&self, other: &Value) -> Value {
        Value::Boolean(self != other)
    }

    pub fn less_than(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Boolean(a < b),
            _ => Value::Boolean(false),
        }
    }

    pub fn less_equal(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Boolean(a <= b),
            _ => Value::Boolean(false),
        }
    }

    pub fn greater_than(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Boolean(a > b),
            _ => Value::Boolean(false),
        }
    }

    pub fn greater_equal(&self, other: &Value) -> Value {
        match (self.to_number(), other.to_number()) {
            (Some(a), Some(b)) => Value::Boolean(a >= b),
            _ => Value::Boolean(false),
        }
    }

    pub fn concat(&self, other: &Value) -> Value {
        Value::String(format!("{}{}", self.to_string(), other.to_string()))
    }

    pub fn length(&self) -> Value {
        match self {
            Value::String(s) => Value::Number(s.len() as f64),
            Value::Table(t) => Value::Number(t.borrow().len() as f64),
            _ => Value::Nil,
        }
    }

    pub fn negate(&self) -> Value {
        match self.to_number() {
            Some(n) => Value::Number(-n),
            _ => Value::Nil,
        }
    }

    pub fn not(&self) -> Value {
        Value::Boolean(!self.is_truthy())
    }
}
