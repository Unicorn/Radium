//! TypeScript code generation for expressions
//!
//! Generates type-safe TypeScript code from parsed expressions.

use super::Expression;

/// TypeScript code generator for expressions
pub struct TypeScriptGenerator {
    /// State variable name (default: "state")
    state_var: String,
}

impl Default for TypeScriptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptGenerator {
    /// Create a new TypeScript generator
    pub fn new() -> Self {
        Self {
            state_var: "state".to_string(),
        }
    }

    /// Create a generator with a custom state variable name
    pub fn with_state_var(state_var: impl Into<String>) -> Self {
        Self {
            state_var: state_var.into(),
        }
    }

    /// Generate TypeScript code from an expression
    pub fn generate(&self, expr: &Expression) -> Result<String, TsGenError> {
        Ok(self.generate_expr(expr))
    }

    /// Generate TypeScript with null-safe access
    pub fn generate_safe(&self, expr: &Expression) -> Result<String, TsGenError> {
        Ok(self.generate_expr_safe(expr))
    }

    fn generate_expr(&self, expr: &Expression) -> String {
        match expr {
            // Literals
            Expression::String(s) => self.escape_string(s),
            Expression::Number(n) => format_number(*n),
            Expression::Integer(n) => n.to_string(),
            Expression::Boolean(b) => b.to_string(),
            Expression::Null => "null".to_string(),

            // Variable reference
            Expression::Variable(name) => {
                format!("{}.variables.{}", self.state_var, name)
            }

            // Arithmetic
            Expression::Add { left, right } => {
                format!(
                    "({} + {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::Subtract { left, right } => {
                format!(
                    "({} - {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::Multiply { left, right } => {
                format!(
                    "({} * {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::Divide { left, right } => {
                format!(
                    "({} / {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::Modulo { left, right } => {
                format!(
                    "({} % {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }

            // Comparison
            Expression::Equal { left, right } => {
                format!(
                    "({} === {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::NotEqual { left, right } => {
                format!(
                    "({} !== {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::LessThan { left, right } => {
                format!(
                    "({} < {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::GreaterThan { left, right } => {
                format!(
                    "({} > {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::LessOrEqual { left, right } => {
                format!(
                    "({} <= {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::GreaterOrEqual { left, right } => {
                format!(
                    "({} >= {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }

            // Logical
            Expression::And { left, right } => {
                format!(
                    "({} && {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::Or { left, right } => {
                format!(
                    "({} || {})",
                    self.generate_expr(left),
                    self.generate_expr(right)
                )
            }
            Expression::Not { operand } => {
                format!("(!{})", self.generate_expr(operand))
            }

            // Conditional
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                format!(
                    "({} ? {} : {})",
                    self.generate_expr(condition),
                    self.generate_expr(then_branch),
                    self.generate_expr(else_branch)
                )
            }

            // String concatenation
            Expression::Concat { parts } => {
                let ts_parts: Vec<String> =
                    parts.iter().map(|p| self.generate_expr(p)).collect();
                format!("[{}].join('')", ts_parts.join(", "))
            }

            // Array operations
            Expression::ArrayLength { array } => {
                format!("{}.length", self.generate_expr(array))
            }
            Expression::ArrayIncludes { array, item } => {
                format!(
                    "{}.includes({})",
                    self.generate_expr(array),
                    self.generate_expr(item)
                )
            }

            // Property access
            Expression::Property { object, property } => {
                format!("{}.{}", self.generate_expr(object), property)
            }
            Expression::Index { array, index } => {
                format!(
                    "{}[{}]",
                    self.generate_expr(array),
                    self.generate_expr(index)
                )
            }

            // Function calls
            Expression::FunctionCall { name, args } => {
                let ts_args: Vec<String> = args.iter().map(|a| self.generate_expr(a)).collect();
                self.generate_function_call(name, &ts_args)
            }
        }
    }

    fn generate_expr_safe(&self, expr: &Expression) -> String {
        match expr {
            // For property access, use optional chaining
            Expression::Property { object, property } => {
                format!("{}?.{}", self.generate_expr_safe(object), property)
            }
            Expression::Index { array, index } => {
                format!(
                    "{}?.[{}]",
                    self.generate_expr_safe(array),
                    self.generate_expr(index)
                )
            }
            Expression::Variable(name) => {
                format!("{}?.variables?.{}", self.state_var, name)
            }
            Expression::ArrayLength { array } => {
                format!("({}?.length ?? 0)", self.generate_expr_safe(array))
            }
            Expression::ArrayIncludes { array, item } => {
                format!(
                    "({}?.includes({}) ?? false)",
                    self.generate_expr_safe(array),
                    self.generate_expr_safe(item)
                )
            }
            // For other expressions, use regular generation
            _ => self.generate_expr(expr),
        }
    }

    fn generate_function_call(&self, name: &str, args: &[String]) -> String {
        match name {
            // Math functions - map to JavaScript Math
            "abs" => format!("Math.abs({})", args.first().unwrap_or(&"0".to_string())),
            "floor" => format!("Math.floor({})", args.first().unwrap_or(&"0".to_string())),
            "ceil" => format!("Math.ceil({})", args.first().unwrap_or(&"0".to_string())),
            "round" => format!("Math.round({})", args.first().unwrap_or(&"0".to_string())),
            "min" => format!("Math.min({})", args.join(", ")),
            "max" => format!("Math.max({})", args.join(", ")),
            "sqrt" => format!("Math.sqrt({})", args.first().unwrap_or(&"0".to_string())),
            "pow" => format!(
                "Math.pow({}, {})",
                args.first().unwrap_or(&"0".to_string()),
                args.get(1).unwrap_or(&"1".to_string())
            ),

            // String functions
            "uppercase" | "toUpperCase" => {
                format!("{}.toUpperCase()", args.first().unwrap_or(&"''".to_string()))
            }
            "lowercase" | "toLowerCase" => {
                format!("{}.toLowerCase()", args.first().unwrap_or(&"''".to_string()))
            }
            "trim" => format!("{}.trim()", args.first().unwrap_or(&"''".to_string())),
            "startsWith" => {
                format!(
                    "{}.startsWith({})",
                    args.first().unwrap_or(&"''".to_string()),
                    args.get(1).unwrap_or(&"''".to_string())
                )
            }
            "endsWith" => {
                format!(
                    "{}.endsWith({})",
                    args.first().unwrap_or(&"''".to_string()),
                    args.get(1).unwrap_or(&"''".to_string())
                )
            }
            "substring" => {
                if args.len() >= 3 {
                    format!(
                        "{}.substring({}, {})",
                        args[0], args[1], args[2]
                    )
                } else if args.len() >= 2 {
                    format!("{}.substring({})", args[0], args[1])
                } else {
                    format!("{}", args.first().unwrap_or(&"''".to_string()))
                }
            }
            "concat" => format!("[{}].join('')", args.join(", ")),
            "split" => {
                format!(
                    "{}.split({})",
                    args.first().unwrap_or(&"''".to_string()),
                    args.get(1).unwrap_or(&"''".to_string())
                )
            }
            "replace" => {
                format!(
                    "{}.replace({}, {})",
                    args.first().unwrap_or(&"''".to_string()),
                    args.get(1).unwrap_or(&"''".to_string()),
                    args.get(2).unwrap_or(&"''".to_string())
                )
            }

            // Type conversion
            "toString" | "String" => {
                format!("String({})", args.first().unwrap_or(&"null".to_string()))
            }
            "toNumber" | "Number" => {
                format!("Number({})", args.first().unwrap_or(&"0".to_string()))
            }
            "toInteger" | "parseInt" => {
                format!("parseInt({}, 10)", args.first().unwrap_or(&"'0'".to_string()))
            }
            "toBoolean" | "Boolean" => {
                format!("Boolean({})", args.first().unwrap_or(&"false".to_string()))
            }

            // Array functions
            "Array" => format!("[{}]", args.join(", ")),
            "join" => {
                format!(
                    "{}.join({})",
                    args.first().unwrap_or(&"[]".to_string()),
                    args.get(1).unwrap_or(&"','".to_string())
                )
            }
            "map" => {
                format!(
                    "{}.map({})",
                    args.first().unwrap_or(&"[]".to_string()),
                    args.get(1).unwrap_or(&"(x) => x".to_string())
                )
            }
            "filter" => {
                format!(
                    "{}.filter({})",
                    args.first().unwrap_or(&"[]".to_string()),
                    args.get(1).unwrap_or(&"(x) => true".to_string())
                )
            }
            "find" => {
                format!(
                    "{}.find({})",
                    args.first().unwrap_or(&"[]".to_string()),
                    args.get(1).unwrap_or(&"(x) => true".to_string())
                )
            }
            "some" => {
                format!(
                    "{}.some({})",
                    args.first().unwrap_or(&"[]".to_string()),
                    args.get(1).unwrap_or(&"(x) => true".to_string())
                )
            }
            "every" => {
                format!(
                    "{}.every({})",
                    args.first().unwrap_or(&"[]".to_string()),
                    args.get(1).unwrap_or(&"(x) => true".to_string())
                )
            }
            "reduce" => {
                if args.len() >= 3 {
                    format!("{}.reduce({}, {})", args[0], args[1], args[2])
                } else {
                    format!(
                        "{}.reduce({})",
                        args.first().unwrap_or(&"[]".to_string()),
                        args.get(1).unwrap_or(&"(acc, x) => acc".to_string())
                    )
                }
            }

            // Object functions
            "keys" => format!("Object.keys({})", args.first().unwrap_or(&"{}".to_string())),
            "values" => format!("Object.values({})", args.first().unwrap_or(&"{}".to_string())),
            "entries" => format!("Object.entries({})", args.first().unwrap_or(&"{}".to_string())),

            // Date functions
            "now" => "Date.now()".to_string(),
            "Date" => {
                if args.is_empty() {
                    "new Date()".to_string()
                } else {
                    format!("new Date({})", args[0])
                }
            }

            // JSON functions
            "JSON.stringify" | "stringify" => {
                format!("JSON.stringify({})", args.first().unwrap_or(&"null".to_string()))
            }
            "JSON.parse" | "parse" => {
                format!("JSON.parse({})", args.first().unwrap_or(&"'{}'".to_string()))
            }

            // Default: pass through as function call
            _ => format!("{}({})", name, args.join(", ")),
        }
    }

    fn escape_string(&self, s: &str) -> String {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        format!("'{}'", escaped)
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{:.1}", n) // Keep one decimal to indicate it's a float
    } else {
        n.to_string()
    }
}

/// TypeScript generation error
#[derive(Debug, Clone, thiserror::Error)]
pub enum TsGenError {
    #[error("Unsupported expression type: {0}")]
    UnsupportedExpression(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expressions::ExpressionParser;

    fn gen(expr: &str) -> String {
        let parsed = ExpressionParser::parse(expr).expect("parse error");
        let generator = TypeScriptGenerator::new();
        generator.generate(&parsed).expect("gen error")
    }

    fn gen_safe(expr: &str) -> String {
        let parsed = ExpressionParser::parse(expr).expect("parse error");
        let generator = TypeScriptGenerator::new();
        generator.generate_safe(&parsed).expect("gen error")
    }

    #[test]
    fn test_gen_literals() {
        assert_eq!(gen("42"), "42");
        assert_eq!(gen("3.14"), "3.14");
        assert_eq!(gen("'hello'"), "'hello'");
        assert_eq!(gen("true"), "true");
        assert_eq!(gen("null"), "null");
    }

    #[test]
    fn test_gen_variables() {
        assert_eq!(gen("foo"), "state.variables.foo");
        assert_eq!(gen("my_var"), "state.variables.my_var");
    }

    #[test]
    fn test_gen_arithmetic() {
        assert_eq!(gen("1 + 2"), "(1 + 2)");
        assert_eq!(gen("3 - 1"), "(3 - 1)");
        assert_eq!(gen("2 * 3"), "(2 * 3)");
        assert_eq!(gen("6 / 2"), "(6 / 2)");
        assert_eq!(gen("7 % 3"), "(7 % 3)");
    }

    #[test]
    fn test_gen_comparison() {
        assert_eq!(gen("a == b"), "(state.variables.a === state.variables.b)");
        assert_eq!(gen("a != b"), "(state.variables.a !== state.variables.b)");
        assert_eq!(gen("a < b"), "(state.variables.a < state.variables.b)");
    }

    #[test]
    fn test_gen_logical() {
        assert_eq!(gen("a && b"), "(state.variables.a && state.variables.b)");
        assert_eq!(gen("a || b"), "(state.variables.a || state.variables.b)");
        assert_eq!(gen("!a"), "(!state.variables.a)");
    }

    #[test]
    fn test_gen_ternary() {
        assert_eq!(gen("a ? b : c"), "(state.variables.a ? state.variables.b : state.variables.c)");
    }

    #[test]
    fn test_gen_property_access() {
        assert_eq!(gen("obj.prop"), "state.variables.obj.prop");
        assert_eq!(gen("arr[0]"), "state.variables.arr[0]");
    }

    #[test]
    fn test_gen_array_operations() {
        assert_eq!(gen("arr.length()"), "state.variables.arr.length");
        assert_eq!(gen("arr.includes(x)"), "state.variables.arr.includes(state.variables.x)");
    }

    #[test]
    fn test_gen_function_calls() {
        assert_eq!(gen("abs(-5)"), "Math.abs(-5)");
        assert_eq!(gen("floor(3.7)"), "Math.floor(3.7)");
        assert_eq!(gen("uppercase('hello')"), "'hello'.toUpperCase()");
        assert_eq!(gen("trim('  hi  ')"), "'  hi  '.trim()");
    }

    #[test]
    fn test_gen_safe_variable() {
        assert_eq!(gen_safe("foo"), "state?.variables?.foo");
    }

    #[test]
    fn test_gen_safe_property() {
        assert_eq!(gen_safe("obj.prop"), "state?.variables?.obj?.prop");
    }

    #[test]
    fn test_gen_safe_array_length() {
        assert_eq!(gen_safe("arr.length()"), "(state?.variables?.arr?.length ?? 0)");
    }

    #[test]
    fn test_gen_complex_expression() {
        let result = gen("x > 0 && y < 10 ? x + y : x - y");
        assert!(result.contains("state.variables.x"));
        assert!(result.contains("state.variables.y"));
        assert!(result.contains("?"));
    }

    #[test]
    fn test_gen_string_escaping() {
        assert_eq!(gen("'hello\\nworld'"), "'hello\\nworld'");
        assert_eq!(gen("'it\\'s'"), "'it\\'s'");
    }

    #[test]
    fn test_gen_custom_state_var() {
        let parsed = ExpressionParser::parse("foo").expect("parse error");
        let generator = TypeScriptGenerator::with_state_var("ctx");
        let result = generator.generate(&parsed).expect("gen error");
        assert_eq!(result, "ctx.variables.foo");
    }
}
