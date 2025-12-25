/*!
# Erlang Expression AST Nodes

Expressions are the computational building blocks of Erlang. This module defines
all expression types that can appear in Erlang source code.
*/

use super::*;

/// All expression types in Erlang
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    /// Literal values (atoms, numbers, strings, etc.)
    Literal(Literal),

    /// Variable reference
    Variable(Variable),

    /// Function call (local or external)
    FunctionCall(FunctionCall),

    /// Anonymous function (fun)
    Fun(Fun),

    /// Case expression
    Case(Case),

    /// If expression
    If(If),

    /// Receive expression
    Receive(Receive),

    /// Try-catch expression
    Try(Try),

    /// Binary construction
    Binary(BinaryExpr),

    /// List construction
    List(ListExpr),

    /// Tuple construction
    Tuple(TupleExpr),

    /// Map construction/update
    Map(MapExpr),

    /// Record construction/update
    Record(RecordExpr),

    /// List comprehension
    ListComprehension(ListComprehension),

    /// Binary comprehension
    BinaryComprehension(BinaryComprehension),

    /// Block expression (begin-end)
    Block(Block),

    /// Parenthesized expression
    Parenthesized(Box<Expression>),

    /// Unary operator application
    UnaryOp(UnaryOp),

    /// Binary operator application
    BinaryOp(BinaryOp),

    /// Record field access
    RecordAccess(RecordAccess),

    /// Map field access
    MapAccess(MapAccess),
}

/// Function call expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    pub module: Option<Atom>,
    pub function: Atom,
    pub args: Vec<Expression>,
}

impl FunctionCall {
    pub fn local(function: Atom, args: Vec<Expression>) -> Self {
        Self {
            module: None,
            function,
            args,
        }
    }

    pub fn external(module: Atom, function: Atom, args: Vec<Expression>) -> Self {
        Self {
            module: Some(module),
            function,
            args,
        }
    }

    pub fn arity(&self) -> usize {
        self.args.len()
    }
}

/// Anonymous function (fun)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fun {
    pub clauses: Vec<Clause>,
}

impl Fun {
    pub fn new(clauses: Vec<Clause>) -> Self {
        Self { clauses }
    }
}

/// Case expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Case {
    pub expression: Box<Expression>,
    pub clauses: Vec<Clause>,
}

impl Case {
    pub fn new(expression: Expression, clauses: Vec<Clause>) -> Self {
        Self {
            expression: Box::new(expression),
            clauses,
        }
    }
}

/// If expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct If {
    pub clauses: Vec<IfExprClause>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfExprClause {
    pub guard: Vec<super::Guard>,
    pub body: Vec<Expression>,
}

impl If {
    pub fn new(clauses: Vec<IfExprClause>) -> Self {
        Self { clauses }
    }
}

/// Receive expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receive {
    pub clauses: Option<Vec<Clause>>,
    pub timeout: Option<Box<Expression>>,
    pub after: Option<Vec<Expression>>,
}

impl Receive {
    pub fn with_clauses(clauses: Vec<Clause>) -> Self {
        Self {
            clauses: Some(clauses),
            timeout: None,
            after: None,
        }
    }

    pub fn with_timeout(clauses: Vec<Clause>, timeout: Expression, after: Vec<Expression>) -> Self {
        Self {
            clauses: Some(clauses),
            timeout: Some(Box::new(timeout)),
            after: Some(after),
        }
    }
}

/// Try-catch expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Try {
    pub body: Vec<Expression>,
    pub catch_clauses: Vec<CatchClause>,
    pub after: Option<Vec<Expression>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatchExprClause {
    pub class: Option<super::Atom>,
    pub reason: super::Pattern,
    pub stack: Option<super::Variable>,
    pub guard: Vec<super::Guard>,
    pub body: Vec<Expression>,
}

/// Binary construction expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryExpr {
    pub segments: Vec<BinarySegment>,
}

/// List construction expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListExpr {
    pub elements: Vec<Expression>,
    pub tail: Option<Box<Expression>>, // For improper lists
}

impl ListExpr {
    pub fn proper(elements: Vec<Expression>) -> Self {
        Self {
            elements,
            tail: None,
        }
    }

    pub fn improper(elements: Vec<Expression>, tail: Expression) -> Self {
        Self {
            elements,
            tail: Some(Box::new(tail)),
        }
    }
}

/// Tuple construction expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TupleExpr {
    pub elements: Vec<Expression>,
}

impl TupleExpr {
    pub fn new(elements: Vec<Expression>) -> Self {
        Self { elements }
    }
}

/// Map construction/update expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapExpr {
    pub base: Option<Box<Expression>>, // None for construction, Some for update
    pub entries: Vec<MapEntryExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapEntryExpr {
    pub key: Expression,
    pub value: Expression,
    pub is_assoc: bool, // true for =>, false for :=
}

impl MapExpr {
    pub fn construction(entries: Vec<MapEntryExpr>) -> Self {
        Self {
            base: None,
            entries,
        }
    }

    pub fn update(base: Expression, entries: Vec<MapEntryExpr>) -> Self {
        Self {
            base: Some(Box::new(base)),
            entries,
        }
    }
}

/// Record construction/update expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordExpr {
    pub name: Atom,
    pub base: Option<Box<Expression>>, // None for construction, Some for update
    pub fields: Vec<RecordFieldExpr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordFieldExpr {
    pub name: Atom,
    pub value: Expression,
}

impl RecordExpr {
    pub fn construction(name: Atom, fields: Vec<RecordFieldExpr>) -> Self {
        Self {
            name,
            base: None,
            fields,
        }
    }

    pub fn update(name: Atom, base: Expression, fields: Vec<RecordFieldExpr>) -> Self {
        Self {
            name,
            base: Some(Box::new(base)),
            fields,
        }
    }
}

/// List comprehension
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListComprehension {
    pub expression: Box<Expression>,
    pub qualifiers: Vec<ComprehensionQualifier>,
}

/// Binary comprehension
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryComprehension {
    pub expression: Box<Expression>,
    pub qualifiers: Vec<ComprehensionQualifier>,
}

/// Comprehension qualifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComprehensionQualifier {
    Generator(super::Pattern, Expression),
    Filter(Expression),
}

/// Block expression (begin-end)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub expressions: Vec<Expression>,
}

impl Block {
    pub fn new(expressions: Vec<Expression>) -> Self {
        Self { expressions }
    }
}

/// Unary operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOp {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOperator {
    Plus,   // +
    Minus,  // -
    Not,    // not
    Bnot,   // bnot
}

impl UnaryOp {
    pub fn new(operator: UnaryOperator, operand: Expression) -> Self {
        Self {
            operator,
            operand: Box::new(operand),
        }
    }
}

/// Binary operator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOp {
    pub operator: BinaryOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /
    Modulo,   // mod
    Div,      // div
    Rem,      // rem

    // Comparison
    Equal,        // =:=
    NotEqual,     // =/=
    Less,         // <
    LessEqual,    // =<
    Greater,      // >
    GreaterEqual, // >=
    ExactEqual,   // ==
    ExactNotEqual,// /=

    // Boolean
    And,     // and
    Or,      // or
    Xor,     // xor
    AndAlso, // andalso
    OrElse,  // orelse

    // Bitwise
    Band, // band
    Bor,  // bor
    Bxor, // bxor
    Bsl,  // bsl
    Bsr,  // bsr

    // List operations
    Append, // ++
    Subtract, // --

    // Send
    Send, // !
}

impl BinaryOp {
    pub fn new(operator: BinaryOperator, left: Expression, right: Expression) -> Self {
        Self {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}

/// Record field access
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordAccess {
    pub record: Box<Expression>,
    pub name: Atom,
    pub field: Atom,
}

impl RecordAccess {
    pub fn new(record: Expression, name: Atom, field: Atom) -> Self {
        Self {
            record: Box::new(record),
            name,
            field,
        }
    }
}

/// Map field access
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapAccess {
    pub map: Box<Expression>,
    pub key: Box<Expression>,
}

impl MapAccess {
    pub fn new(map: Expression, key: Expression) -> Self {
        Self {
            map: Box::new(map),
            key: Box::new(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_call() {
        let call = FunctionCall::local(
            Atom::new("add"),
            vec![Expression::Literal(1.into()), Expression::Literal(2.into())],
        );

        assert_eq!(call.function.as_str(), "add");
        assert_eq!(call.arity(), 2);
        assert!(call.module.is_none());
    }

    #[test]
    fn test_external_function_call() {
        let call = FunctionCall::external(
            Atom::new("math"),
            Atom::new("add"),
            vec![Expression::Literal(1.into()), Expression::Literal(2.into())],
        );

        assert_eq!(call.module.as_ref().unwrap().as_str(), "math");
        assert_eq!(call.function.as_str(), "add");
        assert_eq!(call.arity(), 2);
    }

    #[test]
    fn test_case_expression() {
        let case_expr = Case::new(
            Expression::Variable(Variable::new("X")),
            vec![], // Would have clauses in real code
        );

        assert!(matches!(case_expr.expression.as_ref(), Expression::Variable(_)));
    }

    #[test]
    fn test_list_construction() {
        let proper = ListExpr::proper(vec![
            Expression::Literal(1.into()),
            Expression::Literal(2.into()),
        ]);

        assert!(proper.tail.is_none());
        assert_eq!(proper.elements.len(), 2);

        let improper = ListExpr::improper(
            vec![Expression::Literal(1.into())],
            Expression::Literal(2.into()),
        );

        assert!(improper.tail.is_some());
        assert_eq!(improper.elements.len(), 1);
    }

    #[test]
    fn test_tuple_construction() {
        let tuple = TupleExpr::new(vec![
            Expression::Literal("hello".into()),
            Expression::Literal(42.into()),
        ]);

        assert_eq!(tuple.elements.len(), 2);
    }

    #[test]
    fn test_map_construction() {
        let entries = vec![
            MapEntryExpr {
                key: Expression::Literal("key".into()),
                value: Expression::Literal(1.into()),
                is_assoc: true,
            },
        ];

        let map = MapExpr::construction(entries);
        assert!(map.base.is_none());
        assert_eq!(map.entries.len(), 1);
    }

    #[test]
    fn test_map_update() {
        let base = Expression::Variable(Variable::new("Map"));
        let entries = vec![
            MapEntryExpr {
                key: Expression::Literal("key".into()),
                value: Expression::Literal(2.into()),
                is_assoc: false, // :=
            },
        ];

        let map = MapExpr::update(base, entries);
        assert!(map.base.is_some());
        assert_eq!(map.entries.len(), 1);
        assert!(!map.entries[0].is_assoc);
    }

    #[test]
    fn test_binary_operations() {
        let add = BinaryOp::new(
            BinaryOperator::Plus,
            Expression::Literal(1.into()),
            Expression::Literal(2.into()),
        );

        assert_eq!(add.operator, BinaryOperator::Plus);

        let equal = BinaryOp::new(
            BinaryOperator::ExactEqual,
            Expression::Variable(Variable::new("X")),
            Expression::Variable(Variable::new("Y")),
        );

        assert_eq!(equal.operator, BinaryOperator::ExactEqual);
    }

    #[test]
    fn test_unary_operations() {
        let negate = UnaryOp::new(
            UnaryOperator::Minus,
            Expression::Literal(42.into()),
        );

        assert_eq!(negate.operator, UnaryOperator::Minus);
    }

    #[test]
    fn test_record_access() {
        let access = RecordAccess::new(
            Expression::Variable(Variable::new("Person")),
            Atom::new("person"),
            Atom::new("name"),
        );

        assert_eq!(access.name.as_str(), "person");
        assert_eq!(access.field.as_str(), "name");
    }
}
