//! Database Query component schema
//!
//! The Database Query component executes SQL queries against a database.
//! Supports Supabase/PostgreSQL with parameterized queries.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Database operation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum QueryOperation {
    /// SELECT query
    #[default]
    Select,
    /// INSERT query
    Insert,
    /// UPDATE query
    Update,
    /// DELETE query
    Delete,
    /// Raw SQL (use with caution)
    Raw,
    /// Stored procedure/function call
    Function,
}

impl QueryOperation {
    /// Convert to TypeScript representation
    pub fn to_typescript(&self) -> &'static str {
        match self {
            QueryOperation::Select => "'select'",
            QueryOperation::Insert => "'insert'",
            QueryOperation::Update => "'update'",
            QueryOperation::Delete => "'delete'",
            QueryOperation::Raw => "'raw'",
            QueryOperation::Function => "'function'",
        }
    }

    /// Check if this operation modifies data
    pub fn is_mutation(&self) -> bool {
        matches!(
            self,
            QueryOperation::Insert
                | QueryOperation::Update
                | QueryOperation::Delete
                | QueryOperation::Raw
        )
    }
}

/// Query result format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ResultFormat {
    /// Return all rows as array
    #[default]
    Rows,
    /// Return first row only
    Single,
    /// Return count of affected rows
    Count,
    /// Return nothing (for mutations)
    None,
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    /// Connection name/alias
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_name: Option<String>,

    /// Database URL (from environment variable name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_env_var: Option<String>,

    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Query timeout in milliseconds
    #[serde(default = "default_query_timeout")]
    pub timeout_ms: u64,
}

fn default_pool_size() -> u32 {
    5
}

fn default_query_timeout() -> u64 {
    30000
}

impl ConnectionConfig {
    /// Create config with connection name
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            connection_name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Create config with environment variable
    pub fn from_env(env_var: impl Into<String>) -> Self {
        Self {
            url_env_var: Some(env_var.into()),
            ..Default::default()
        }
    }

    /// Set pool size
    pub fn with_pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }
}

/// Database Query component input
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseQueryInput {
    /// Operation type
    #[serde(default)]
    pub operation: QueryOperation,

    /// Table name (for non-raw queries)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,

    /// SQL query (for raw queries)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,

    /// Query parameters (keyed by name)
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,

    /// Columns to select (empty = all)
    #[serde(default)]
    pub columns: Vec<String>,

    /// WHERE conditions
    #[serde(default)]
    pub where_conditions: Vec<WhereCondition>,

    /// ORDER BY clauses
    #[serde(default)]
    pub order_by: Vec<OrderByClause>,

    /// LIMIT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// OFFSET
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,

    /// Data to insert/update
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Result format
    #[serde(default)]
    pub result_format: ResultFormat,

    /// Connection configuration
    #[serde(default)]
    pub connection: ConnectionConfig,

    /// Whether to use a transaction
    #[serde(default)]
    pub use_transaction: bool,
}

impl DatabaseQueryInput {
    /// Create a SELECT query
    pub fn select(table: impl Into<String>) -> Self {
        Self {
            operation: QueryOperation::Select,
            table: Some(table.into()),
            query: None,
            params: HashMap::new(),
            columns: Vec::new(),
            where_conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            data: None,
            result_format: ResultFormat::Rows,
            connection: ConnectionConfig::default(),
            use_transaction: false,
        }
    }

    /// Create an INSERT query
    pub fn insert(table: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            operation: QueryOperation::Insert,
            table: Some(table.into()),
            query: None,
            params: HashMap::new(),
            columns: Vec::new(),
            where_conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            data: Some(data),
            result_format: ResultFormat::Single,
            connection: ConnectionConfig::default(),
            use_transaction: false,
        }
    }

    /// Create an UPDATE query
    pub fn update(table: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            operation: QueryOperation::Update,
            table: Some(table.into()),
            data: Some(data),
            ..Self::select("")
        }
    }

    /// Create a DELETE query
    pub fn delete(table: impl Into<String>) -> Self {
        Self {
            operation: QueryOperation::Delete,
            table: Some(table.into()),
            result_format: ResultFormat::Count,
            ..Self::select("")
        }
    }

    /// Create a raw SQL query
    pub fn raw(sql: impl Into<String>) -> Self {
        Self {
            operation: QueryOperation::Raw,
            query: Some(sql.into()),
            ..Self::select("")
        }
    }

    /// Select specific columns
    pub fn columns(mut self, columns: Vec<impl Into<String>>) -> Self {
        self.columns = columns.into_iter().map(|c| c.into()).collect();
        self
    }

    /// Add a WHERE condition
    pub fn where_eq(mut self, column: impl Into<String>, value: serde_json::Value) -> Self {
        self.where_conditions.push(WhereCondition::eq(column, value));
        self
    }

    /// Add WHERE IN condition
    pub fn where_in(mut self, column: impl Into<String>, values: Vec<serde_json::Value>) -> Self {
        self.where_conditions.push(WhereCondition::in_list(column, values));
        self
    }

    /// Add ORDER BY
    pub fn order_by(mut self, column: impl Into<String>, ascending: bool) -> Self {
        self.order_by.push(OrderByClause {
            column: column.into(),
            ascending,
        });
        self
    }

    /// Set LIMIT
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set OFFSET
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set result format
    pub fn format(mut self, format: ResultFormat) -> Self {
        self.result_format = format;
        self
    }

    /// Set connection config
    pub fn with_connection(mut self, connection: ConnectionConfig) -> Self {
        self.connection = connection;
        self
    }

    /// Enable transaction
    pub fn in_transaction(mut self) -> Self {
        self.use_transaction = true;
        self
    }

    /// Add a named parameter
    pub fn with_param(mut self, name: impl Into<String>, value: serde_json::Value) -> Self {
        self.params.insert(name.into(), value);
        self
    }
}

impl Default for DatabaseQueryInput {
    fn default() -> Self {
        Self::select("table")
    }
}

/// WHERE condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhereCondition {
    /// Column name
    pub column: String,

    /// Operator
    pub operator: WhereOperator,

    /// Value(s)
    pub value: serde_json::Value,
}

impl WhereCondition {
    /// Create an equals condition
    pub fn eq(column: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            column: column.into(),
            operator: WhereOperator::Eq,
            value,
        }
    }

    /// Create a not equals condition
    pub fn neq(column: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            column: column.into(),
            operator: WhereOperator::Neq,
            value,
        }
    }

    /// Create an IN condition
    pub fn in_list(column: impl Into<String>, values: Vec<serde_json::Value>) -> Self {
        Self {
            column: column.into(),
            operator: WhereOperator::In,
            value: serde_json::Value::Array(values),
        }
    }

    /// Create a LIKE condition
    pub fn like(column: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            column: column.into(),
            operator: WhereOperator::Like,
            value: serde_json::Value::String(pattern.into()),
        }
    }
}

/// WHERE operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WhereOperator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    Ilike,
    In,
    IsNull,
    IsNotNull,
}

impl WhereOperator {
    /// Convert to SQL operator
    pub fn to_sql(&self) -> &'static str {
        match self {
            WhereOperator::Eq => "=",
            WhereOperator::Neq => "!=",
            WhereOperator::Gt => ">",
            WhereOperator::Gte => ">=",
            WhereOperator::Lt => "<",
            WhereOperator::Lte => "<=",
            WhereOperator::Like => "LIKE",
            WhereOperator::Ilike => "ILIKE",
            WhereOperator::In => "IN",
            WhereOperator::IsNull => "IS NULL",
            WhereOperator::IsNotNull => "IS NOT NULL",
        }
    }
}

/// ORDER BY clause
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderByClause {
    /// Column name
    pub column: String,

    /// Ascending order
    #[serde(default = "default_true")]
    pub ascending: bool,
}

fn default_true() -> bool {
    true
}

/// Database Query component output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseQueryOutput {
    /// Whether the query succeeded
    pub success: bool,

    /// Query results (for SELECT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Number of rows affected
    #[serde(default)]
    pub rows_affected: u64,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl DatabaseQueryOutput {
    /// Create a successful query result
    pub fn success(data: serde_json::Value, rows_affected: u64, duration_ms: u64) -> Self {
        Self {
            success: true,
            data: Some(data),
            rows_affected,
            duration_ms,
            error: None,
        }
    }

    /// Create a failed query result
    pub fn failure(error: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            success: false,
            data: None,
            rows_affected: 0,
            duration_ms,
            error: Some(error.into()),
        }
    }
}

impl Default for DatabaseQueryOutput {
    fn default() -> Self {
        Self::success(serde_json::Value::Array(vec![]), 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_operation() {
        assert!(QueryOperation::Insert.is_mutation());
        assert!(QueryOperation::Update.is_mutation());
        assert!(!QueryOperation::Select.is_mutation());
    }

    #[test]
    fn test_select_query() {
        let query = DatabaseQueryInput::select("users")
            .columns(vec!["id", "name", "email"])
            .where_eq("active", serde_json::json!(true))
            .order_by("name", true)
            .limit(10);

        assert_eq!(query.operation, QueryOperation::Select);
        assert_eq!(query.table, Some("users".to_string()));
        assert_eq!(query.columns.len(), 3);
        assert_eq!(query.where_conditions.len(), 1);
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_insert_query() {
        let data = serde_json::json!({
            "name": "John",
            "email": "john@example.com"
        });
        let query = DatabaseQueryInput::insert("users", data);

        assert_eq!(query.operation, QueryOperation::Insert);
        assert!(query.data.is_some());
    }

    #[test]
    fn test_update_query() {
        let data = serde_json::json!({"active": false});
        let query = DatabaseQueryInput::update("users", data)
            .where_eq("id", serde_json::json!(123));

        assert_eq!(query.operation, QueryOperation::Update);
        assert_eq!(query.where_conditions.len(), 1);
    }

    #[test]
    fn test_delete_query() {
        let query = DatabaseQueryInput::delete("users")
            .where_eq("active", serde_json::json!(false));

        assert_eq!(query.operation, QueryOperation::Delete);
        assert_eq!(query.result_format, ResultFormat::Count);
    }

    #[test]
    fn test_raw_query() {
        let query = DatabaseQueryInput::raw("SELECT * FROM users WHERE age > $1")
            .with_param("age", serde_json::json!(18));

        assert_eq!(query.operation, QueryOperation::Raw);
        assert!(query.query.is_some());
        assert!(query.params.contains_key("age"));
    }

    #[test]
    fn test_where_conditions() {
        let eq = WhereCondition::eq("status", serde_json::json!("active"));
        assert_eq!(eq.operator, WhereOperator::Eq);

        let in_list = WhereCondition::in_list(
            "id",
            vec![serde_json::json!(1), serde_json::json!(2), serde_json::json!(3)],
        );
        assert_eq!(in_list.operator, WhereOperator::In);

        let like = WhereCondition::like("name", "%john%");
        assert_eq!(like.operator, WhereOperator::Like);
    }

    #[test]
    fn test_where_operator_sql() {
        assert_eq!(WhereOperator::Eq.to_sql(), "=");
        assert_eq!(WhereOperator::In.to_sql(), "IN");
        assert_eq!(WhereOperator::Like.to_sql(), "LIKE");
        assert_eq!(WhereOperator::IsNull.to_sql(), "IS NULL");
    }

    #[test]
    fn test_connection_config() {
        let config = ConnectionConfig::from_env("DATABASE_URL")
            .with_pool_size(10)
            .with_timeout(60000);

        assert_eq!(config.url_env_var, Some("DATABASE_URL".to_string()));
        assert_eq!(config.pool_size, 10);
        assert_eq!(config.timeout_ms, 60000);
    }

    #[test]
    fn test_query_output_success() {
        let output = DatabaseQueryOutput::success(
            serde_json::json!([{"id": 1, "name": "John"}]),
            1,
            50,
        );

        assert!(output.success);
        assert!(output.data.is_some());
        assert_eq!(output.rows_affected, 1);
    }

    #[test]
    fn test_query_output_failure() {
        let output = DatabaseQueryOutput::failure("Connection refused", 10);

        assert!(!output.success);
        assert!(output.error.is_some());
    }

    #[test]
    fn test_serialization() {
        let query = DatabaseQueryInput::select("products")
            .where_eq("category", serde_json::json!("electronics"))
            .limit(20);

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("operation"));
        assert!(json.contains("table"));
        assert!(json.contains("whereConditions"));
    }
}
