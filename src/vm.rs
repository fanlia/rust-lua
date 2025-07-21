use crate::parser::{BinaryOperator, Expr, Stmt, UnaryOperator};
use crate::value::{Function, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Vm {
    globals: Rc<RefCell<HashMap<String, Value>>>,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
}

#[derive(Debug)]
pub struct CallFrame {
    locals: Rc<RefCell<HashMap<String, Value>>>,
    return_value: Option<Value>,
}

impl Vm {
    pub fn new() -> Self {
        let mut vm = Vm {
            globals: Rc::new(RefCell::new(HashMap::new())),
            stack: Vec::new(),
            call_stack: Vec::new(),
        };
        vm.setup_builtins();
        vm
    }

    fn setup_builtins(&mut self) {
        self.globals.borrow_mut().insert(
            "print".to_string(),
            Value::Function(Function::Native(print)),
        );
        self.globals.borrow_mut().insert(
            "type".to_string(),
            Value::Function(Function::Native(type_of)),
        );
        self.globals.borrow_mut().insert(
            "tonumber".to_string(),
            Value::Function(Function::Native(to_number)),
        );
        self.globals.borrow_mut().insert(
            "tostring".to_string(),
            Value::Function(Function::Native(to_string)),
        );
    }

    pub fn execute(&mut self, stmts: Vec<Stmt>) -> Value {
        let mut result = Value::Nil;
        for stmt in stmts {
            result = self.execute_stmt(&stmt);
            if let Value::Function(Function::UserDefined { .. }) = &result {
                continue;
            }
        }
        result
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Value {
        match stmt {
            Stmt::Expr(expr) => self.evaluate_expr(expr),
            Stmt::Assignment { variables, values } => self.execute_assignment(variables, values),
            Stmt::LocalAssignment { variables, values } => {
                self.execute_local_assignment(variables, values)
            }
            Stmt::If {
                condition,
                then_block,
                else_if_blocks,
                else_block,
            } => self.execute_if(condition, then_block, else_if_blocks, else_block),
            Stmt::While { condition, body } => self.execute_while(condition, body),
            Stmt::Repeat { body, condition } => self.execute_repeat(body, condition),
            Stmt::For {
                variable,
                start,
                end,
                step,
                body,
            } => self.execute_for(variable, start, end, step, body),
            Stmt::Function {
                name,
                parameters,
                body,
            } => self.execute_function(name, parameters, body),
            Stmt::LocalFunction {
                name,
                parameters,
                body,
            } => self.execute_local_function(name, parameters, body),
            Stmt::Return(values) => self.execute_return(values),
            Stmt::Break => Value::Nil,
        }
    }

    fn execute_assignment(&mut self, variables: &Vec<String>, values: &Vec<Expr>) -> Value {
        let evaluated_values: Vec<Value> = values.iter().map(|v| self.evaluate_expr(v)).collect();

        for (i, var) in variables.iter().enumerate() {
            let value = evaluated_values.get(i).unwrap_or(&Value::Nil).clone();
            self.globals.borrow_mut().insert(var.clone(), value);
        }

        Value::Nil
    }

    fn execute_local_assignment(&mut self, variables: &Vec<String>, values: &Vec<Expr>) -> Value {
        let evaluated_values: Vec<Value> = values.iter().map(|v| self.evaluate_expr(v)).collect();

        let current_frame = self.call_stack.last_mut().unwrap_or_else(|| {
            panic!("No call frame available");
        });

        for (i, var) in variables.iter().enumerate() {
            let value = evaluated_values.get(i).unwrap_or(&Value::Nil).clone();
            current_frame.locals.borrow_mut().insert(var.clone(), value);
        }

        Value::Nil
    }

    fn execute_if(
        &mut self,
        condition: &Expr,
        then_block: &Vec<Stmt>,
        else_if_blocks: &Vec<(Expr, Vec<Stmt>)>,
        else_block: &Option<Vec<Stmt>>,
    ) -> Value {
        let cond_value = self.evaluate_expr(condition);
        if cond_value.is_truthy() {
            return self.execute_block(then_block);
        }

        for (else_if_cond, else_if_body) in else_if_blocks {
            let else_if_value = self.evaluate_expr(else_if_cond);
            if else_if_value.is_truthy() {
                return self.execute_block(else_if_body);
            }
        }

        if let Some(else_body) = else_block {
            return self.execute_block(else_body);
        }

        Value::Nil
    }

    fn execute_while(&mut self, condition: &Expr, body: &Vec<Stmt>) -> Value {
        loop {
            let cond_value = self.evaluate_expr(condition);
            if !cond_value.is_truthy() {
                break;
            }
            self.execute_block(body);
        }
        Value::Nil
    }

    fn execute_repeat(&mut self, body: &Vec<Stmt>, condition: &Expr) -> Value {
        loop {
            self.execute_block(body);
            let cond_value = self.evaluate_expr(condition);
            if cond_value.is_truthy() {
                break;
            }
        }
        Value::Nil
    }

    fn execute_for(
        &mut self,
        variable: &String,
        start: &Expr,
        end: &Expr,
        step: &Option<Expr>,
        body: &Vec<Stmt>,
    ) -> Value {
        let start_val = self.evaluate_expr(start).to_number().unwrap_or(0.0);
        let end_val = self.evaluate_expr(end).to_number().unwrap_or(0.0);
        let step_val = step
            .as_ref()
            .map(|s| self.evaluate_expr(s).to_number().unwrap_or(1.0))
            .unwrap_or(1.0);

        let mut current = start_val;
        while (step_val > 0.0 && current <= end_val) || (step_val < 0.0 && current >= end_val) {
            let current_frame = self.call_stack.last_mut().unwrap_or_else(|| {
                panic!("No call frame available");
            });
            current_frame
                .locals
                .borrow_mut()
                .insert(variable.clone(), Value::Number(current));

            self.execute_block(body);
            current += step_val;
        }
        Value::Nil
    }

    fn execute_function(
        &mut self,
        name: &String,
        parameters: &Vec<String>,
        body: &Vec<Stmt>,
    ) -> Value {
        let function = Value::Function(Function::UserDefined {
            parameters: parameters.clone(),
            body: body.clone(),
            closure: Rc::new(RefCell::new(HashMap::new())),
        });
        self.globals.borrow_mut().insert(name.clone(), function);
        Value::Nil
    }

    fn execute_local_function(
        &mut self,
        name: &String,
        parameters: &Vec<String>,
        body: &Vec<Stmt>,
    ) -> Value {
        let function = Value::Function(Function::UserDefined {
            parameters: parameters.clone(),
            body: body.clone(),
            closure: Rc::new(RefCell::new(HashMap::new())),
        });

        let current_frame = self.call_stack.last_mut().unwrap_or_else(|| {
            panic!("No call frame available");
        });
        current_frame
            .locals
            .borrow_mut()
            .insert(name.clone(), function);
        Value::Nil
    }

    fn execute_return(&mut self, values: &Option<Vec<Expr>>) -> Value {
        match values {
            Some(exprs) => {
                if exprs.len() == 1 {
                    self.evaluate_expr(&exprs[0])
                } else {
                    Value::Nil
                }
            }
            None => Value::Nil,
        }
    }

    fn execute_block(&mut self, stmts: &Vec<Stmt>) -> Value {
        let mut result = Value::Nil;
        for stmt in stmts {
            result = self.execute_stmt(stmt);
        }
        result
    }

    fn evaluate_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Number(n) => Value::Number(*n),
            Expr::String(s) => Value::String(s.clone()),
            Expr::Boolean(b) => Value::Boolean(*b),
            Expr::Nil => Value::Nil,
            Expr::Identifier(name) => self.get_variable(name),
            Expr::UnaryOp { operator, operand } => self.evaluate_unary_op(operator, operand),
            Expr::BinaryOp {
                left,
                operator,
                right,
            } => self.evaluate_binary_op(left, operator, right),
            Expr::FunctionCall { name, arguments } => self.evaluate_function_call(name, arguments),
            Expr::TableAccess { table, key } => self.evaluate_table_access(table, key),
            Expr::TableConstructor { fields } => self.evaluate_table_constructor(fields),
        }
    }

    fn get_variable(&mut self, name: &str) -> Value {
        for frame in self.call_stack.iter().rev() {
            if let Some(value) = frame.locals.borrow().get(name) {
                return value.clone();
            }
        }

        if let Some(value) = self.globals.borrow().get(name) {
            return value.clone();
        }

        Value::Nil
    }

    fn evaluate_unary_op(&mut self, operator: &UnaryOperator, operand: &Expr) -> Value {
        let value = self.evaluate_expr(operand);
        match operator {
            UnaryOperator::Not => value.not(),
            UnaryOperator::Minus => value.negate(),
            UnaryOperator::Length => value.length(),
        }
    }

    fn evaluate_binary_op(
        &mut self,
        left: &Expr,
        operator: &BinaryOperator,
        right: &Expr,
    ) -> Value {
        let left_val = self.evaluate_expr(left);
        let right_val = self.evaluate_expr(right);

        match operator {
            BinaryOperator::Add => left_val.add(&right_val),
            BinaryOperator::Subtract => left_val.subtract(&right_val),
            BinaryOperator::Multiply => left_val.multiply(&right_val),
            BinaryOperator::Divide => left_val.divide(&right_val),
            BinaryOperator::Modulo => left_val.modulo(&right_val),
            BinaryOperator::Power => left_val.power(&right_val),
            BinaryOperator::Concat => left_val.concat(&right_val),
            BinaryOperator::Equal => left_val.equal(&right_val),
            BinaryOperator::NotEqual => left_val.not_equal(&right_val),
            BinaryOperator::LessThan => left_val.less_than(&right_val),
            BinaryOperator::LessEqual => left_val.less_equal(&right_val),
            BinaryOperator::GreaterThan => left_val.greater_than(&right_val),
            BinaryOperator::GreaterEqual => left_val.greater_equal(&right_val),
            BinaryOperator::And => Value::Boolean(left_val.is_truthy() && right_val.is_truthy()),
            BinaryOperator::Or => Value::Boolean(left_val.is_truthy() || right_val.is_truthy()),
        }
    }

    fn evaluate_function_call(&mut self, name: &String, arguments: &Vec<Expr>) -> Value {
        let func = self.get_variable(name);
        let evaluated_args: Vec<Value> = arguments
            .iter()
            .map(|arg| self.evaluate_expr(arg))
            .collect();

        match func {
            Value::Function(Function::Native(native_func)) => native_func(self, evaluated_args),
            Value::Function(Function::UserDefined {
                parameters,
                body,
                closure,
            }) => self.execute_user_function(&parameters, &body, &closure, evaluated_args),
            _ => Value::Nil,
        }
    }

    fn execute_user_function(
        &mut self,
        parameters: &Vec<String>,
        body: &Vec<Stmt>,
        _closure: &Rc<RefCell<HashMap<String, Value>>>,
        args: Vec<Value>,
    ) -> Value {
        let locals = Rc::new(RefCell::new(HashMap::new()));

        for (i, param) in parameters.iter().enumerate() {
            let value = args.get(i).unwrap_or(&Value::Nil).clone();
            locals.borrow_mut().insert(param.clone(), value);
        }

        let frame = CallFrame {
            locals: locals.clone(),
            return_value: None,
        };

        self.call_stack.push(frame);

        let result = self.execute_block(body);

        self.call_stack.pop();

        result
    }

    fn evaluate_table_access(&mut self, table: &Expr, key: &Expr) -> Value {
        let table_val = self.evaluate_expr(table);
        let key_val = self.evaluate_expr(key);

        if let Value::Table(t) = table_val {
            t.borrow().get(&key_val).cloned().unwrap_or(Value::Nil)
        } else {
            Value::Nil
        }
    }

    fn evaluate_table_constructor(&mut self, fields: &Vec<crate::parser::TableField>) -> Value {
        let table = Value::new_table();

        if let Value::Table(t) = &table {
            for field in fields {
                match field {
                    crate::parser::TableField::Value(expr) => {
                        let value = self.evaluate_expr(expr);
                        let key = Value::Number(t.borrow().len() as f64 + 1.0);
                        t.borrow_mut().insert(key, value);
                    }
                    crate::parser::TableField::KeyValue(key, expr) => {
                        let value = self.evaluate_expr(expr);
                        t.borrow_mut().insert(Value::String(key.clone()), value);
                    }
                }
            }
        }

        table
    }
}

fn print(_vm: &mut Vm, args: Vec<Value>) -> Value {
    let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    println!("{}", output.join("\t"));
    Value::Nil
}

fn type_of(_vm: &mut Vm, args: Vec<Value>) -> Value {
    if args.len() != 1 {
        return Value::Nil;
    }

    let type_name = match &args[0] {
        Value::Nil => "nil",
        Value::Boolean(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Table(_) => "table",
        Value::Function(_) => "function",
    };

    Value::String(type_name.to_string())
}

fn to_number(_vm: &mut Vm, args: Vec<Value>) -> Value {
    if args.is_empty() {
        return Value::Nil;
    }

    args[0].to_number().map(Value::Number).unwrap_or(Value::Nil)
}

fn to_string(_vm: &mut Vm, args: Vec<Value>) -> Value {
    if args.is_empty() {
        return Value::String("".to_string());
    }

    Value::String(args[0].to_string())
}

