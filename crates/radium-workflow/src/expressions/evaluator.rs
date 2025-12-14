//! Expression evaluator
//!
//! Evaluates parsed expressions at runtime using a variable context.

use std::collections::HashMap;

use crate::schema::variables::VariableValue;

use super::Expression;

/// Expression evaluator
pub struct ExpressionEvaluator {
    /// Variable values for evaluation
    variables: HashMap<String, VariableValue>,
}

impl ExpressionEvaluator {
    /// Create a new evaluator with the given variable context
    pub fn new(variables: HashMap<String, VariableValue>) -> Self {
        Self { variables }
    }

    /// Evaluate an expression
    pub fn evaluate(&self, expr: &Expression) -> Result<VariableValue, EvaluationError> {
        match expr {
            // Literals
            Expression::String(s) => Ok(VariableValue::String(s.clone())),
            Expression::Number(n) => Ok(VariableValue::Number(*n)),
            Expression::Integer(n) => Ok(VariableValue::Integer(*n)),
            Expression::Boolean(b) => Ok(VariableValue::Boolean(*b)),
            Expression::Null => Ok(VariableValue::Null),

            // Variable reference
            Expression::Variable(name) => self.variables.get(name).cloned().ok_or_else(|| {
                EvaluationError::UndefinedVariable(name.clone())
            }),

            // Arithmetic
            Expression::Add { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.add_values(l, r)
            }
            Expression::Subtract { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.subtract_values(l, r)
            }
            Expression::Multiply { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.multiply_values(l, r)
            }
            Expression::Divide { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.divide_values(l, r)
            }
            Expression::Modulo { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.modulo_values(l, r)
            }

            // Comparison
            Expression::Equal { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                Ok(VariableValue::Boolean(self.values_equal(&l, &r)))
            }
            Expression::NotEqual { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                Ok(VariableValue::Boolean(!self.values_equal(&l, &r)))
            }
            Expression::LessThan { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.compare_values(&l, &r, |a, b| a < b)
            }
            Expression::GreaterThan { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.compare_values(&l, &r, |a, b| a > b)
            }
            Expression::LessOrEqual { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.compare_values(&l, &r, |a, b| a <= b)
            }
            Expression::GreaterOrEqual { left, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.compare_values(&l, &r, |a, b| a >= b)
            }

            // Logical
            Expression::And { left, right } => {
                let l = self.evaluate(left)?;
                if !self.is_truthy(&l) {
                    return Ok(VariableValue::Boolean(false));
                }
                let r = self.evaluate(right)?;
                Ok(VariableValue::Boolean(self.is_truthy(&r)))
            }
            Expression::Or { left, right } => {
                let l = self.evaluate(left)?;
                if self.is_truthy(&l) {
                    return Ok(VariableValue::Boolean(true));
                }
                let r = self.evaluate(right)?;
                Ok(VariableValue::Boolean(self.is_truthy(&r)))
            }
            Expression::Not { operand } => {
                let v = self.evaluate(operand)?;
                Ok(VariableValue::Boolean(!self.is_truthy(&v)))
            }

            // Conditional
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.evaluate(condition)?;
                if self.is_truthy(&cond) {
                    self.evaluate(then_branch)
                } else {
                    self.evaluate(else_branch)
                }
            }

            // String concatenation
            Expression::Concat { parts } => {
                let mut result = String::new();
                for p in parts {
                    let v = self.evaluate(p)?;
                    result.push_str(&self.to_string(&v));
                }
                Ok(VariableValue::String(result))
            }

            // Array operations
            Expression::ArrayLength { array } => {
                let arr = self.evaluate(array)?;
                match arr {
                    VariableValue::Array(items) => Ok(VariableValue::Integer(items.len() as i64)),
                    VariableValue::String(s) => Ok(VariableValue::Integer(s.len() as i64)),
                    _ => Err(EvaluationError::TypeError(
                        "length".to_string(),
                        "array or string".to_string(),
                    )),
                }
            }
            Expression::ArrayIncludes { array, item } => {
                let arr = self.evaluate(array)?;
                let needle = self.evaluate(item)?;
                match arr {
                    VariableValue::Array(items) => {
                        let needle_json = needle.to_json();
                        let found = items.iter().any(|i| *i == needle_json);
                        Ok(VariableValue::Boolean(found))
                    }
                    VariableValue::String(s) => {
                        if let VariableValue::String(needle_str) = needle {
                            Ok(VariableValue::Boolean(s.contains(&needle_str)))
                        } else {
                            Err(EvaluationError::TypeError(
                                "includes".to_string(),
                                "string".to_string(),
                            ))
                        }
                    }
                    _ => Err(EvaluationError::TypeError(
                        "includes".to_string(),
                        "array or string".to_string(),
                    )),
                }
            }

            // Property access
            Expression::Property { object, property } => {
                let obj = self.evaluate(object)?;
                match obj {
                    VariableValue::Object(map) => {
                        let value = map.get(property).ok_or_else(|| {
                            EvaluationError::PropertyNotFound(property.clone())
                        })?;
                        VariableValue::from_json(value).ok_or_else(|| {
                            EvaluationError::TypeError("property".to_string(), "convertible value".to_string())
                        })
                    }
                    _ => Err(EvaluationError::TypeError(
                        "property access".to_string(),
                        "object".to_string(),
                    )),
                }
            }
            Expression::Index { array, index } => {
                let arr = self.evaluate(array)?;
                let idx = self.evaluate(index)?;
                match (arr, idx) {
                    (VariableValue::Array(items), VariableValue::Integer(i)) => {
                        let i = i as usize;
                        let value = items
                            .get(i)
                            .ok_or_else(|| EvaluationError::IndexOutOfBounds(i, items.len()))?;
                        VariableValue::from_json(value).ok_or_else(|| {
                            EvaluationError::TypeError("index".to_string(), "convertible value".to_string())
                        })
                    }
                    (VariableValue::String(s), VariableValue::Integer(i)) => {
                        let i = i as usize;
                        s.chars()
                            .nth(i)
                            .map(|c| VariableValue::String(c.to_string()))
                            .ok_or_else(|| EvaluationError::IndexOutOfBounds(i, s.len()))
                    }
                    _ => Err(EvaluationError::TypeError(
                        "index".to_string(),
                        "array and integer".to_string(),
                    )),
                }
            }

            // Function calls
            Expression::FunctionCall { name, args } => {
                self.call_function(name, args)
            }
        }
    }

    fn add_values(
        &self,
        left: VariableValue,
        right: VariableValue,
    ) -> Result<VariableValue, EvaluationError> {
        match (left, right) {
            (VariableValue::Integer(a), VariableValue::Integer(b)) => {
                Ok(VariableValue::Integer(a + b))
            }
            (VariableValue::Number(a), VariableValue::Number(b)) => {
                Ok(VariableValue::Number(a + b))
            }
            (VariableValue::Integer(a), VariableValue::Number(b)) => {
                Ok(VariableValue::Number(a as f64 + b))
            }
            (VariableValue::Number(a), VariableValue::Integer(b)) => {
                Ok(VariableValue::Number(a + b as f64))
            }
            (VariableValue::String(a), VariableValue::String(b)) => {
                Ok(VariableValue::String(format!("{}{}", a, b)))
            }
            (VariableValue::String(a), b) => {
                Ok(VariableValue::String(format!("{}{}", a, self.to_string(&b))))
            }
            (a, VariableValue::String(b)) => {
                Ok(VariableValue::String(format!("{}{}", self.to_string(&a), b)))
            }
            _ => Err(EvaluationError::TypeError(
                "+".to_string(),
                "numbers or strings".to_string(),
            )),
        }
    }

    fn subtract_values(
        &self,
        left: VariableValue,
        right: VariableValue,
    ) -> Result<VariableValue, EvaluationError> {
        match (left, right) {
            (VariableValue::Integer(a), VariableValue::Integer(b)) => {
                Ok(VariableValue::Integer(a - b))
            }
            (VariableValue::Number(a), VariableValue::Number(b)) => {
                Ok(VariableValue::Number(a - b))
            }
            (VariableValue::Integer(a), VariableValue::Number(b)) => {
                Ok(VariableValue::Number(a as f64 - b))
            }
            (VariableValue::Number(a), VariableValue::Integer(b)) => {
                Ok(VariableValue::Number(a - b as f64))
            }
            _ => Err(EvaluationError::TypeError(
                "-".to_string(),
                "numbers".to_string(),
            )),
        }
    }

    fn multiply_values(
        &self,
        left: VariableValue,
        right: VariableValue,
    ) -> Result<VariableValue, EvaluationError> {
        match (left, right) {
            (VariableValue::Integer(a), VariableValue::Integer(b)) => {
                Ok(VariableValue::Integer(a * b))
            }
            (VariableValue::Number(a), VariableValue::Number(b)) => {
                Ok(VariableValue::Number(a * b))
            }
            (VariableValue::Integer(a), VariableValue::Number(b)) => {
                Ok(VariableValue::Number(a as f64 * b))
            }
            (VariableValue::Number(a), VariableValue::Integer(b)) => {
                Ok(VariableValue::Number(a * b as f64))
            }
            _ => Err(EvaluationError::TypeError(
                "*".to_string(),
                "numbers".to_string(),
            )),
        }
    }

    fn divide_values(
        &self,
        left: VariableValue,
        right: VariableValue,
    ) -> Result<VariableValue, EvaluationError> {
        match (left, right) {
            (VariableValue::Integer(a), VariableValue::Integer(b)) => {
                if b == 0 {
                    return Err(EvaluationError::DivisionByZero);
                }
                Ok(VariableValue::Integer(a / b))
            }
            (VariableValue::Number(a), VariableValue::Number(b)) => {
                if b == 0.0 {
                    return Err(EvaluationError::DivisionByZero);
                }
                Ok(VariableValue::Number(a / b))
            }
            (VariableValue::Integer(a), VariableValue::Number(b)) => {
                if b == 0.0 {
                    return Err(EvaluationError::DivisionByZero);
                }
                Ok(VariableValue::Number(a as f64 / b))
            }
            (VariableValue::Number(a), VariableValue::Integer(b)) => {
                if b == 0 {
                    return Err(EvaluationError::DivisionByZero);
                }
                Ok(VariableValue::Number(a / b as f64))
            }
            _ => Err(EvaluationError::TypeError(
                "/".to_string(),
                "numbers".to_string(),
            )),
        }
    }

    fn modulo_values(
        &self,
        left: VariableValue,
        right: VariableValue,
    ) -> Result<VariableValue, EvaluationError> {
        match (left, right) {
            (VariableValue::Integer(a), VariableValue::Integer(b)) => {
                if b == 0 {
                    return Err(EvaluationError::DivisionByZero);
                }
                Ok(VariableValue::Integer(a % b))
            }
            (VariableValue::Number(a), VariableValue::Number(b)) => {
                if b == 0.0 {
                    return Err(EvaluationError::DivisionByZero);
                }
                Ok(VariableValue::Number(a % b))
            }
            (VariableValue::Integer(a), VariableValue::Number(b)) => {
                if b == 0.0 {
                    return Err(EvaluationError::DivisionByZero);
                }
                Ok(VariableValue::Number((a as f64) % b))
            }
            (VariableValue::Number(a), VariableValue::Integer(b)) => {
                if b == 0 {
                    return Err(EvaluationError::DivisionByZero);
                }
                Ok(VariableValue::Number(a % (b as f64)))
            }
            _ => Err(EvaluationError::TypeError(
                "%".to_string(),
                "numbers".to_string(),
            )),
        }
    }

    fn values_equal(&self, left: &VariableValue, right: &VariableValue) -> bool {
        match (left, right) {
            (VariableValue::Null, VariableValue::Null) => true,
            (VariableValue::Boolean(a), VariableValue::Boolean(b)) => a == b,
            (VariableValue::Integer(a), VariableValue::Integer(b)) => a == b,
            (VariableValue::Number(a), VariableValue::Number(b)) => (a - b).abs() < f64::EPSILON,
            (VariableValue::Integer(a), VariableValue::Number(b)) => (*a as f64 - b).abs() < f64::EPSILON,
            (VariableValue::Number(a), VariableValue::Integer(b)) => (a - *b as f64).abs() < f64::EPSILON,
            (VariableValue::String(a), VariableValue::String(b)) => a == b,
            (VariableValue::Array(a), VariableValue::Array(b)) => a == b,
            (VariableValue::Object(a), VariableValue::Object(b)) => a == b,
            _ => false,
        }
    }

    fn compare_values<F>(
        &self,
        left: &VariableValue,
        right: &VariableValue,
        cmp: F,
    ) -> Result<VariableValue, EvaluationError>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (VariableValue::Integer(a), VariableValue::Integer(b)) => {
                Ok(VariableValue::Boolean(cmp(*a as f64, *b as f64)))
            }
            (VariableValue::Number(a), VariableValue::Number(b)) => {
                Ok(VariableValue::Boolean(cmp(*a, *b)))
            }
            (VariableValue::Integer(a), VariableValue::Number(b)) => {
                Ok(VariableValue::Boolean(cmp(*a as f64, *b)))
            }
            (VariableValue::Number(a), VariableValue::Integer(b)) => {
                Ok(VariableValue::Boolean(cmp(*a, *b as f64)))
            }
            (VariableValue::String(a), VariableValue::String(b)) => {
                // String comparison by lexicographic order
                let ord = a.cmp(b) as i32;
                Ok(VariableValue::Boolean(cmp(ord as f64, 0.0)))
            }
            _ => Err(EvaluationError::TypeError(
                "comparison".to_string(),
                "comparable types".to_string(),
            )),
        }
    }

    fn is_truthy(&self, value: &VariableValue) -> bool {
        match value {
            VariableValue::Null => false,
            VariableValue::Boolean(b) => *b,
            VariableValue::Integer(n) => *n != 0,
            VariableValue::Number(n) => *n != 0.0,
            VariableValue::String(s) => !s.is_empty(),
            VariableValue::Array(arr) => !arr.is_empty(),
            VariableValue::Object(_) => true,
            VariableValue::Datetime(_) => true,
            VariableValue::Duration(_) => true,
        }
    }

    fn to_string(&self, value: &VariableValue) -> String {
        match value {
            VariableValue::Null => "null".to_string(),
            VariableValue::Boolean(b) => b.to_string(),
            VariableValue::Integer(n) => n.to_string(),
            VariableValue::Number(n) => n.to_string(),
            VariableValue::String(s) => s.clone(),
            VariableValue::Array(arr) => serde_json::to_string(arr).unwrap_or_default(),
            VariableValue::Object(obj) => serde_json::to_string(obj).unwrap_or_default(),
            VariableValue::Datetime(dt) => dt.to_rfc3339(),
            VariableValue::Duration(ms) => format!("{}ms", ms),
        }
    }

    fn call_function(
        &self,
        name: &str,
        args: &[Expression],
    ) -> Result<VariableValue, EvaluationError> {
        let evaluated_args: Result<Vec<_>, _> = args.iter().map(|a| self.evaluate(a)).collect();
        let args = evaluated_args?;

        match name {
            // Math functions
            "abs" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                match &args[0] {
                    VariableValue::Integer(n) => Ok(VariableValue::Integer(n.abs())),
                    VariableValue::Number(n) => Ok(VariableValue::Number(n.abs())),
                    _ => Err(EvaluationError::TypeError("abs".to_string(), "number".to_string())),
                }
            }
            "floor" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                match &args[0] {
                    VariableValue::Integer(n) => Ok(VariableValue::Integer(*n)),
                    VariableValue::Number(n) => Ok(VariableValue::Integer(n.floor() as i64)),
                    _ => Err(EvaluationError::TypeError("floor".to_string(), "number".to_string())),
                }
            }
            "ceil" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                match &args[0] {
                    VariableValue::Integer(n) => Ok(VariableValue::Integer(*n)),
                    VariableValue::Number(n) => Ok(VariableValue::Integer(n.ceil() as i64)),
                    _ => Err(EvaluationError::TypeError("ceil".to_string(), "number".to_string())),
                }
            }
            "round" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                match &args[0] {
                    VariableValue::Integer(n) => Ok(VariableValue::Integer(*n)),
                    VariableValue::Number(n) => Ok(VariableValue::Integer(n.round() as i64)),
                    _ => Err(EvaluationError::TypeError("round".to_string(), "number".to_string())),
                }
            }
            "min" => {
                if args.len() < 2 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 2, args.len()));
                }
                let mut min_val = self.to_number(&args[0])?;
                for arg in &args[1..] {
                    let val = self.to_number(arg)?;
                    if val < min_val {
                        min_val = val;
                    }
                }
                Ok(VariableValue::Number(min_val))
            }
            "max" => {
                if args.len() < 2 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 2, args.len()));
                }
                let mut max_val = self.to_number(&args[0])?;
                for arg in &args[1..] {
                    let val = self.to_number(arg)?;
                    if val > max_val {
                        max_val = val;
                    }
                }
                Ok(VariableValue::Number(max_val))
            }

            // String functions
            "toUpperCase" | "uppercase" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                match &args[0] {
                    VariableValue::String(s) => Ok(VariableValue::String(s.to_uppercase())),
                    _ => Err(EvaluationError::TypeError(name.to_string(), "string".to_string())),
                }
            }
            "toLowerCase" | "lowercase" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                match &args[0] {
                    VariableValue::String(s) => Ok(VariableValue::String(s.to_lowercase())),
                    _ => Err(EvaluationError::TypeError(name.to_string(), "string".to_string())),
                }
            }
            "trim" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                match &args[0] {
                    VariableValue::String(s) => Ok(VariableValue::String(s.trim().to_string())),
                    _ => Err(EvaluationError::TypeError(name.to_string(), "string".to_string())),
                }
            }
            "startsWith" => {
                if args.len() != 2 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 2, args.len()));
                }
                match (&args[0], &args[1]) {
                    (VariableValue::String(s), VariableValue::String(prefix)) => {
                        Ok(VariableValue::Boolean(s.starts_with(prefix)))
                    }
                    _ => Err(EvaluationError::TypeError(name.to_string(), "strings".to_string())),
                }
            }
            "endsWith" => {
                if args.len() != 2 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 2, args.len()));
                }
                match (&args[0], &args[1]) {
                    (VariableValue::String(s), VariableValue::String(suffix)) => {
                        Ok(VariableValue::Boolean(s.ends_with(suffix)))
                    }
                    _ => Err(EvaluationError::TypeError(name.to_string(), "strings".to_string())),
                }
            }
            "substring" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 2, args.len()));
                }
                match (&args[0], &args[1]) {
                    (VariableValue::String(s), VariableValue::Integer(start)) => {
                        let start = *start as usize;
                        let end = if args.len() == 3 {
                            if let VariableValue::Integer(e) = &args[2] {
                                *e as usize
                            } else {
                                return Err(EvaluationError::TypeError(name.to_string(), "integer".to_string()));
                            }
                        } else {
                            s.len()
                        };
                        let chars: Vec<char> = s.chars().collect();
                        let result: String = chars.get(start..end.min(chars.len())).unwrap_or(&[]).iter().collect();
                        Ok(VariableValue::String(result))
                    }
                    _ => Err(EvaluationError::TypeError(name.to_string(), "string and integer".to_string())),
                }
            }
            "concat" => {
                let mut result = String::new();
                for arg in &args {
                    result.push_str(&self.to_string(arg));
                }
                Ok(VariableValue::String(result))
            }

            // Type conversion
            "toString" | "String" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                Ok(VariableValue::String(self.to_string(&args[0])))
            }
            "toNumber" | "Number" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                Ok(VariableValue::Number(self.to_number(&args[0])?))
            }
            "toInteger" | "parseInt" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                match &args[0] {
                    VariableValue::Integer(n) => Ok(VariableValue::Integer(*n)),
                    VariableValue::Number(n) => Ok(VariableValue::Integer(*n as i64)),
                    VariableValue::String(s) => {
                        s.parse::<i64>()
                            .map(VariableValue::Integer)
                            .map_err(|_| EvaluationError::TypeError(name.to_string(), "parseable string".to_string()))
                    }
                    _ => Err(EvaluationError::TypeError(name.to_string(), "number or string".to_string())),
                }
            }
            "toBoolean" | "Boolean" => {
                if args.len() != 1 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                Ok(VariableValue::Boolean(self.is_truthy(&args[0])))
            }

            // Array functions
            "Array" => {
                // Array constructor - convert args to array
                let json_values: Vec<serde_json::Value> = args.iter().map(|v| v.to_json()).collect();
                Ok(VariableValue::Array(json_values))
            }
            "join" => {
                if args.len() < 1 || args.len() > 2 {
                    return Err(EvaluationError::WrongArgumentCount(name.to_string(), 1, args.len()));
                }
                match &args[0] {
                    VariableValue::Array(arr) => {
                        let separator = if args.len() == 2 {
                            if let VariableValue::String(s) = &args[1] {
                                s.as_str()
                            } else {
                                ","
                            }
                        } else {
                            ","
                        };
                        let strings: Vec<String> = arr
                            .iter()
                            .map(|v| {
                                if let Some(s) = v.as_str() {
                                    s.to_string()
                                } else {
                                    v.to_string()
                                }
                            })
                            .collect();
                        Ok(VariableValue::String(strings.join(separator)))
                    }
                    _ => Err(EvaluationError::TypeError(name.to_string(), "array".to_string())),
                }
            }

            _ => Err(EvaluationError::UnknownFunction(name.to_string())),
        }
    }

    fn to_number(&self, value: &VariableValue) -> Result<f64, EvaluationError> {
        match value {
            VariableValue::Integer(n) => Ok(*n as f64),
            VariableValue::Number(n) => Ok(*n),
            VariableValue::String(s) => s
                .parse()
                .map_err(|_| EvaluationError::TypeError("number".to_string(), "parseable string".to_string())),
            _ => Err(EvaluationError::TypeError("number".to_string(), "number or string".to_string())),
        }
    }
}

/// Evaluation error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum EvaluationError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Type error: {0} expects {1}")]
    TypeError(String, String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Unknown function: {0}")]
    UnknownFunction(String),

    #[error("Wrong argument count for {0}: expected {1}, got {2}")]
    WrongArgumentCount(String, usize, usize),

    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    #[error("Index out of bounds: {0} >= {1}")]
    IndexOutOfBounds(usize, usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expressions::ExpressionParser;

    fn eval(expr: &str, vars: &[(&str, VariableValue)]) -> Result<VariableValue, EvaluationError> {
        let parsed = ExpressionParser::parse(expr).expect("parse error");
        let context: HashMap<String, VariableValue> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        let evaluator = ExpressionEvaluator::new(context);
        evaluator.evaluate(&parsed)
    }

    #[test]
    fn test_eval_literals() {
        assert_eq!(eval("42", &[]).unwrap(), VariableValue::Integer(42));
        assert_eq!(eval("3.14", &[]).unwrap(), VariableValue::Number(3.14));
        assert_eq!(
            eval("'hello'", &[]).unwrap(),
            VariableValue::String("hello".to_string())
        );
        assert_eq!(eval("true", &[]).unwrap(), VariableValue::Boolean(true));
        assert_eq!(eval("null", &[]).unwrap(), VariableValue::Null);
    }

    #[test]
    fn test_eval_variables() {
        assert_eq!(
            eval("x", &[("x", VariableValue::Integer(10))]).unwrap(),
            VariableValue::Integer(10)
        );

        assert!(eval("undefined_var", &[]).is_err());
    }

    #[test]
    fn test_eval_arithmetic() {
        assert_eq!(eval("1 + 2", &[]).unwrap(), VariableValue::Integer(3));
        assert_eq!(eval("5 - 3", &[]).unwrap(), VariableValue::Integer(2));
        assert_eq!(eval("2 * 3", &[]).unwrap(), VariableValue::Integer(6));
        assert_eq!(eval("6 / 2", &[]).unwrap(), VariableValue::Integer(3));
        assert_eq!(eval("7 % 3", &[]).unwrap(), VariableValue::Integer(1));
    }

    #[test]
    fn test_eval_mixed_arithmetic() {
        assert_eq!(eval("1 + 2.5", &[]).unwrap(), VariableValue::Number(3.5));
        assert_eq!(eval("2.5 * 2", &[]).unwrap(), VariableValue::Number(5.0));
    }

    #[test]
    fn test_eval_string_concat() {
        assert_eq!(
            eval("'hello' + ' ' + 'world'", &[]).unwrap(),
            VariableValue::String("hello world".to_string())
        );
        assert_eq!(
            eval("'count: ' + 42", &[]).unwrap(),
            VariableValue::String("count: 42".to_string())
        );
    }

    #[test]
    fn test_eval_comparison() {
        assert_eq!(eval("1 == 1", &[]).unwrap(), VariableValue::Boolean(true));
        assert_eq!(eval("1 != 2", &[]).unwrap(), VariableValue::Boolean(true));
        assert_eq!(eval("1 < 2", &[]).unwrap(), VariableValue::Boolean(true));
        assert_eq!(eval("2 > 1", &[]).unwrap(), VariableValue::Boolean(true));
        assert_eq!(eval("1 <= 1", &[]).unwrap(), VariableValue::Boolean(true));
        assert_eq!(eval("2 >= 2", &[]).unwrap(), VariableValue::Boolean(true));
    }

    #[test]
    fn test_eval_logical() {
        assert_eq!(
            eval("true && true", &[]).unwrap(),
            VariableValue::Boolean(true)
        );
        assert_eq!(
            eval("true && false", &[]).unwrap(),
            VariableValue::Boolean(false)
        );
        assert_eq!(
            eval("false || true", &[]).unwrap(),
            VariableValue::Boolean(true)
        );
        assert_eq!(eval("!false", &[]).unwrap(), VariableValue::Boolean(true));
    }

    #[test]
    fn test_eval_ternary() {
        assert_eq!(
            eval("true ? 1 : 2", &[]).unwrap(),
            VariableValue::Integer(1)
        );
        assert_eq!(
            eval("false ? 1 : 2", &[]).unwrap(),
            VariableValue::Integer(2)
        );
    }

    #[test]
    fn test_eval_division_by_zero() {
        assert!(eval("1 / 0", &[]).is_err());
        assert!(eval("1 % 0", &[]).is_err());
    }

    #[test]
    fn test_eval_functions() {
        assert_eq!(eval("abs(-5)", &[]).unwrap(), VariableValue::Integer(5));
        assert_eq!(eval("floor(3.7)", &[]).unwrap(), VariableValue::Integer(3));
        assert_eq!(eval("ceil(3.2)", &[]).unwrap(), VariableValue::Integer(4));
        assert_eq!(eval("round(3.5)", &[]).unwrap(), VariableValue::Integer(4));
        assert_eq!(eval("min(1, 2, 3)", &[]).unwrap(), VariableValue::Number(1.0));
        assert_eq!(eval("max(1, 2, 3)", &[]).unwrap(), VariableValue::Number(3.0));
    }

    #[test]
    fn test_eval_string_functions() {
        assert_eq!(
            eval("uppercase('hello')", &[]).unwrap(),
            VariableValue::String("HELLO".to_string())
        );
        assert_eq!(
            eval("lowercase('HELLO')", &[]).unwrap(),
            VariableValue::String("hello".to_string())
        );
        assert_eq!(
            eval("trim('  hi  ')", &[]).unwrap(),
            VariableValue::String("hi".to_string())
        );
        assert_eq!(
            eval("startsWith('hello', 'he')", &[]).unwrap(),
            VariableValue::Boolean(true)
        );
        assert_eq!(
            eval("endsWith('hello', 'lo')", &[]).unwrap(),
            VariableValue::Boolean(true)
        );
    }

    #[test]
    fn test_eval_complex_expression() {
        let result = eval(
            "x > 0 && y < 10 ? x + y : x - y",
            &[
                ("x", VariableValue::Integer(5)),
                ("y", VariableValue::Integer(3)),
            ],
        )
        .unwrap();
        assert_eq!(result, VariableValue::Integer(8));
    }
}
