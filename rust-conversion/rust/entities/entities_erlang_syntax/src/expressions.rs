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

    // Additional comprehensive tests for all expression types

    #[test]
    fn test_expression_enum_variants() {
        // Test all Expression enum variants for basic construction
        let literal = Expression::Literal(Literal::Integer(Integer::from_i64(42)));
        assert!(matches!(literal, Expression::Literal(_)));

        let variable = Expression::Variable(Variable::new("X"));
        assert!(matches!(variable, Expression::Variable(_)));

        let func_call = Expression::FunctionCall(FunctionCall::local(
            Atom::new("test"),
            vec![Expression::Literal(1.into())],
        ));
        assert!(matches!(func_call, Expression::FunctionCall(_)));

        let fun = Expression::Fun(Fun::new(vec![]));
        assert!(matches!(fun, Expression::Fun(_)));

        let case_expr = Expression::Case(Case::new(
            Expression::Variable(Variable::new("X")),
            vec![],
        ));
        assert!(matches!(case_expr, Expression::Case(_)));

        let if_expr = Expression::If(If::new(vec![]));
        assert!(matches!(if_expr, Expression::If(_)));

        let receive = Expression::Receive(Receive::with_clauses(vec![]));
        assert!(matches!(receive, Expression::Receive(_)));

        let try_expr = Expression::Try(Try {
            body: vec![],
            catch_clauses: vec![],
            after: None,
        });
        assert!(matches!(try_expr, Expression::Try(_)));

        let binary = Expression::Binary(BinaryExpr { segments: vec![] });
        assert!(matches!(binary, Expression::Binary(_)));

        let list = Expression::List(ListExpr::proper(vec![]));
        assert!(matches!(list, Expression::List(_)));

        let tuple = Expression::Tuple(TupleExpr::new(vec![]));
        assert!(matches!(tuple, Expression::Tuple(_)));

        let map = Expression::Map(MapExpr::construction(vec![]));
        assert!(matches!(map, Expression::Map(_)));

        let record = Expression::Record(RecordExpr::construction(
            Atom::new("test"),
            vec![],
        ));
        assert!(matches!(record, Expression::Record(_)));

        let list_comp = Expression::ListComprehension(ListComprehension {
            expression: Box::new(Expression::Literal(1.into())),
            qualifiers: vec![],
        });
        assert!(matches!(list_comp, Expression::ListComprehension(_)));

        let binary_comp = Expression::BinaryComprehension(BinaryComprehension {
            expression: Box::new(Expression::Literal(1.into())),
            qualifiers: vec![],
        });
        assert!(matches!(binary_comp, Expression::BinaryComprehension(_)));

        let block = Expression::Block(Block::new(vec![]));
        assert!(matches!(block, Expression::Block(_)));

        let parenthesized = Expression::Parenthesized(Box::new(Expression::Literal(42.into())));
        assert!(matches!(parenthesized, Expression::Parenthesized(_)));

        let unary = Expression::UnaryOp(UnaryOp::new(
            UnaryOperator::Minus,
            Expression::Literal(42.into()),
        ));
        assert!(matches!(unary, Expression::UnaryOp(_)));

        let binary_op = Expression::BinaryOp(BinaryOp::new(
            BinaryOperator::Plus,
            Expression::Literal(1.into()),
            Expression::Literal(2.into()),
        ));
        assert!(matches!(binary_op, Expression::BinaryOp(_)));

        let record_access = Expression::RecordAccess(RecordAccess::new(
            Expression::Variable(Variable::new("Rec")),
            Atom::new("record"),
            Atom::new("field"),
        ));
        assert!(matches!(record_access, Expression::RecordAccess(_)));

        let map_access = Expression::MapAccess(MapAccess::new(
            Expression::Variable(Variable::new("Map")),
            Expression::Literal("key".into()),
        ));
        assert!(matches!(map_access, Expression::MapAccess(_)));
    }

    #[test]
    fn test_fun_expression() {
        let clause = Clause::new(
            vec![Pattern::Variable(Variable::new("X"))],
            vec![],
            vec![Expression::Variable(Variable::new("X"))],
        );

        let fun = Fun::new(vec![clause]);
        assert_eq!(fun.clauses.len(), 1);

        // Test empty fun
        let empty_fun = Fun::new(vec![]);
        assert!(empty_fun.clauses.is_empty());
    }

    #[test]
    fn test_case_expression_comprehensive() {
        let var = Expression::Variable(Variable::new("Value"));
        let clause = Clause::new(
            vec![Pattern::Literal(Literal::Integer(Integer::from_i64(1)))],
            vec![],
            vec![Expression::Literal("one".into())],
        );

        let case = Case::new(var, vec![clause]);
        assert_eq!(case.clauses.len(), 1);
        assert!(matches!(case.expression.as_ref(), Expression::Variable(_)));
    }

    #[test]
    fn test_if_expression() {
        let clause = IfExprClause {
            guard: vec![Guard::Expression(Expression::Literal(true.into()))],
            body: vec![Expression::Literal("true".into())],
        };

        let if_expr = If::new(vec![clause]);
        assert_eq!(if_expr.clauses.len(), 1);

        // Test empty if
        let empty_if = If::new(vec![]);
        assert!(empty_if.clauses.is_empty());
    }

    #[test]
    fn test_if_expr_clause() {
        let clause = IfExprClause {
            guard: vec![Guard::Expression(Expression::Literal(true.into()))],
            body: vec![Expression::Literal(42.into())],
        };

        assert_eq!(clause.guard.len(), 1);
        assert_eq!(clause.body.len(), 1);
    }

    #[test]
    fn test_receive_expression() {
        // Test receive with clauses only
        let clause = Clause::new(
            vec![Pattern::Variable(Variable::new("Msg"))],
            vec![],
            vec![Expression::Variable(Variable::new("Msg"))],
        );
        let receive_clauses = Receive::with_clauses(vec![clause.clone()]);
        assert!(receive_clauses.clauses.is_some());
        assert!(receive_clauses.timeout.is_none());
        assert!(receive_clauses.after.is_none());

        // Test receive with timeout
        let timeout_expr = Expression::Literal(Literal::Integer(Integer::from_i64(5000)));
        let after_body = vec![Expression::Literal(Literal::Atom(Atom::new("timeout")))];
        let receive_timeout = Receive::with_timeout(
            vec![clause],
            timeout_expr,
            after_body,
        );
        assert!(receive_timeout.clauses.is_some());
        assert!(receive_timeout.timeout.is_some());
        assert!(receive_timeout.after.is_some());
    }

    #[test]
    fn test_try_expression() {
        let body = vec![Expression::Literal(1.into())];
        let catch_clause = CatchClause::new(
            Some(Atom::new("error")),
            Pattern::Variable(Variable::new("Reason")),
            Some(Variable::new("Stack")),
            vec![],
            vec![Expression::Literal("caught".into())],
        );

        let try_expr = Try {
            body,
            catch_clauses: vec![catch_clause],
            after: Some(vec![Expression::Literal("cleanup".into())]),
        };

        assert_eq!(try_expr.body.len(), 1);
        assert_eq!(try_expr.catch_clauses.len(), 1);
        assert!(try_expr.after.is_some());
    }

    #[test]
    fn test_catch_expr_clause() {
        let clause = CatchExprClause {
            class: Some(Atom::new("error")),
            reason: Pattern::Variable(Variable::new("Reason")),
            stack: None,
            guard: vec![Guard::Expression(Expression::Literal(true.into()))],
            body: vec![Expression::Literal("handled".into())],
        };

        assert_eq!(clause.class.as_ref().unwrap().as_str(), "error");
        assert_eq!(clause.guard.len(), 1);
        assert_eq!(clause.body.len(), 1);
        assert!(clause.stack.is_none());
    }

    #[test]
    fn test_binary_expression() {
        let segments = vec![BinarySegment {
            value: b"test".to_vec(),
            size: Some(Integer::from_i64(4)),
            unit: Some(Integer::from_i64(8)),
        }];

        let binary = BinaryExpr { segments };
        assert_eq!(binary.segments.len(), 1);
    }

    #[test]
    fn test_record_expression() {
        // Test construction
        let fields = vec![RecordFieldExpr {
            name: Atom::new("name"),
            value: Expression::Literal("test".into()),
        }];
        let record = RecordExpr::construction(Atom::new("person"), fields);
        assert_eq!(record.name.as_str(), "person");
        assert!(record.base.is_none());
        assert_eq!(record.fields.len(), 1);

        // Test update
        let base = Expression::Variable(Variable::new("Person"));
        let update_fields = vec![RecordFieldExpr {
            name: Atom::new("age"),
            value: Expression::Literal(25.into()),
        }];
        let update_record = RecordExpr::update(Atom::new("person"), base, update_fields);
        assert_eq!(update_record.name.as_str(), "person");
        assert!(update_record.base.is_some());
        assert_eq!(update_record.fields.len(), 1);
    }

    #[test]
    fn test_record_field_expr() {
        let field = RecordFieldExpr {
            name: Atom::new("field"),
            value: Expression::Literal(42.into()),
        };

        assert_eq!(field.name.as_str(), "field");
    }

    #[test]
    fn test_comprehensions() {
        // Test list comprehension
        let generator = ComprehensionQualifier::Generator(
            Pattern::Variable(Variable::new("X")),
            Expression::Variable(Variable::new("List")),
        );
        let filter = ComprehensionQualifier::Filter(Expression::BinaryOp(BinaryOp::new(
            BinaryOperator::Greater,
            Expression::Variable(Variable::new("X")),
            Expression::Literal(0.into()),
        )));

        let list_comp = ListComprehension {
            expression: Box::new(Expression::Variable(Variable::new("X"))),
            qualifiers: vec![generator.clone(), filter],
        };
        assert_eq!(list_comp.qualifiers.len(), 2);

        // Test binary comprehension
        let binary_comp = BinaryComprehension {
            expression: Box::new(Expression::Variable(Variable::new("X"))),
            qualifiers: vec![generator],
        };
        assert_eq!(binary_comp.qualifiers.len(), 1);
    }

    #[test]
    fn test_comprehension_qualifier() {
        // Test generator
        let generator = ComprehensionQualifier::Generator(
            Pattern::Variable(Variable::new("X")),
            Expression::Variable(Variable::new("List")),
        );
        assert!(matches!(generator, ComprehensionQualifier::Generator(_, _)));

        // Test filter
        let filter = ComprehensionQualifier::Filter(Expression::Literal(true.into()));
        assert!(matches!(filter, ComprehensionQualifier::Filter(_)));
    }

    #[test]
    fn test_block_expression() {
        let expressions = vec![
            Expression::Variable(Variable::new("X")),
            Expression::Literal(42.into()),
        ];
        let block = Block::new(expressions);
        assert_eq!(block.expressions.len(), 2);
    }

    #[test]
    fn test_map_access() {
        let map_access = MapAccess::new(
            Expression::Variable(Variable::new("Map")),
            Expression::Literal("key".into()),
        );

        assert!(matches!(map_access.map.as_ref(), Expression::Variable(_)));
        assert!(matches!(map_access.key.as_ref(), Expression::Literal(_)));
    }

    #[test]
    fn test_unary_operators() {
        // Test all unary operators
        let operators = vec![
            UnaryOperator::Plus,
            UnaryOperator::Minus,
            UnaryOperator::Not,
            UnaryOperator::Bnot,
        ];

        for op in operators {
            let unary = UnaryOp::new(op.clone(), Expression::Literal(42.into()));
            assert_eq!(unary.operator, op);
        }
    }

    #[test]
    fn test_binary_operators() {
        // Test all binary operators
        let operators = vec![
            BinaryOperator::Plus,
            BinaryOperator::Minus,
            BinaryOperator::Multiply,
            BinaryOperator::Divide,
            BinaryOperator::Modulo,
            BinaryOperator::Div,
            BinaryOperator::Rem,
            BinaryOperator::Equal,
            BinaryOperator::NotEqual,
            BinaryOperator::Less,
            BinaryOperator::LessEqual,
            BinaryOperator::Greater,
            BinaryOperator::GreaterEqual,
            BinaryOperator::ExactEqual,
            BinaryOperator::ExactNotEqual,
            BinaryOperator::And,
            BinaryOperator::Or,
            BinaryOperator::Xor,
            BinaryOperator::AndAlso,
            BinaryOperator::OrElse,
            BinaryOperator::Band,
            BinaryOperator::Bor,
            BinaryOperator::Bxor,
            BinaryOperator::Bsl,
            BinaryOperator::Bsr,
            BinaryOperator::Append,
            BinaryOperator::Subtract,
            BinaryOperator::Send,
        ];

        for op in operators {
            let binary = BinaryOp::new(
                op.clone(),
                Expression::Literal(1.into()),
                Expression::Literal(2.into()),
            );
            assert_eq!(binary.operator, op);
        }
    }

    #[test]
    fn test_complex_expression_nesting() {
        // Test deeply nested expressions
        let nested = Expression::BinaryOp(BinaryOp::new(
            BinaryOperator::Plus,
            Expression::UnaryOp(UnaryOp::new(
                UnaryOperator::Minus,
                Expression::Parenthesized(Box::new(Expression::Literal(42.into()))),
            )),
            Expression::FunctionCall(FunctionCall::external(
                Atom::new("math"),
                Atom::new("sqrt"),
                vec![Expression::Literal(16.into())],
            )),
        ));

        // Verify the structure
        if let Expression::BinaryOp(binary) = nested {
            assert_eq!(binary.operator, BinaryOperator::Plus);
            assert!(matches!(binary.left.as_ref(), Expression::UnaryOp(_)));
            assert!(matches!(binary.right.as_ref(), Expression::FunctionCall(_)));
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn test_expression_equality() {
        let expr1 = Expression::Literal(Literal::Integer(Integer::from_i64(42)));
        let expr2 = Expression::Literal(Literal::Integer(Integer::from_i64(42)));
        let expr3 = Expression::Literal(Literal::Integer(Integer::from_i64(43)));

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);
    }

    #[test]
    fn test_edge_cases() {
        // Test empty collections
        let empty_list = ListExpr::proper(vec![]);
        assert!(empty_list.elements.is_empty());

        let empty_tuple = TupleExpr::new(vec![]);
        assert!(empty_tuple.elements.is_empty());

        let empty_map = MapExpr::construction(vec![]);
        assert!(empty_map.entries.is_empty());

        let empty_record = RecordExpr::construction(Atom::new("empty"), vec![]);
        assert!(empty_record.fields.is_empty());

        // Test single element collections
        let single_list = ListExpr::proper(vec![Expression::Literal(1.into())]);
        assert_eq!(single_list.elements.len(), 1);

        let single_tuple = TupleExpr::new(vec![Expression::Literal(1.into())]);
        assert_eq!(single_tuple.elements.len(), 1);
    }
}
