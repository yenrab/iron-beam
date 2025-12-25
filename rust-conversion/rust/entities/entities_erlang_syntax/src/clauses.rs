/*!
# Erlang Clauses and Guards

Clauses are the fundamental building blocks of Erlang functions and control flow.
They consist of patterns, guards, and body expressions.
*/

use super::*;

/// Function clause (pattern -> guard -> body)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    pub patterns: Vec<Pattern>,
    pub guard: Vec<Guard>,
    pub body: Vec<Expression>,
}

impl Clause {
    pub fn new(patterns: Vec<Pattern>, guard: Vec<Guard>, body: Vec<Expression>) -> Self {
        Self {
            patterns,
            guard,
            body,
        }
    }

    pub fn simple(patterns: Vec<Pattern>, body: Vec<Expression>) -> Self {
        Self::new(patterns, vec![], body)
    }

    pub fn arity(&self) -> usize {
        self.patterns.len()
    }

    /// Check if clause has guards
    pub fn has_guards(&self) -> bool {
        !self.guard.is_empty()
    }
}

/// Guard expression for pattern matching
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Guard {
    /// Simple expression (atom, variable, literal, etc.)
    Expression(Expression),

    /// Function call in guard context
    Call(FunctionCall),

    /// Binary operation
    BinaryOp(BinaryOp),

    /// Unary operation
    UnaryOp(UnaryOp),

    /// Guard sequence (comma-separated)
    And(Box<Guard>, Box<Guard>),

    /// Guard alternative (semicolon-separated)
    Or(Box<Guard>, Box<Guard>),
}

impl From<Expression> for Guard {
    fn from(expr: Expression) -> Self {
        Self::Expression(expr)
    }
}

impl From<FunctionCall> for Guard {
    fn from(call: FunctionCall) -> Self {
        Self::Call(call)
    }
}

/// Guard sequence (multiple guards connected with commas)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardSequence {
    pub guards: Vec<Guard>,
}

impl GuardSequence {
    pub fn new(guards: Vec<Guard>) -> Self {
        Self { guards }
    }

    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }

    pub fn len(&self) -> usize {
        self.guards.len()
    }
}

/// Case clause for case expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseClause {
    pub pattern: Pattern,
    pub guard: Vec<Guard>,
    pub body: Vec<Expression>,
}

impl CaseClause {
    pub fn new(pattern: Pattern, guard: Vec<Guard>, body: Vec<Expression>) -> Self {
        Self {
            pattern,
            guard,
            body,
        }
    }

    pub fn simple(pattern: Pattern, body: Vec<Expression>) -> Self {
        Self::new(pattern, vec![], body)
    }
}

/// If clause for if expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfClause {
    pub guard: Vec<Guard>,
    pub body: Vec<Expression>,
}

impl IfClause {
    pub fn new(guard: Vec<Guard>, body: Vec<Expression>) -> Self {
        Self { guard, body }
    }

    pub fn has_guards(&self) -> bool {
        !self.guard.is_empty()
    }
}

/// Receive clause for receive expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiveClause {
    pub pattern: Pattern,
    pub guard: Vec<Guard>,
    pub body: Vec<Expression>,
}

impl ReceiveClause {
    pub fn new(pattern: Pattern, guard: Vec<Guard>, body: Vec<Expression>) -> Self {
        Self {
            pattern,
            guard,
            body,
        }
    }

    pub fn simple(pattern: Pattern, body: Vec<Expression>) -> Self {
        Self::new(pattern, vec![], body)
    }
}

/// Catch clause for try-catch expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatchClause {
    pub class: Option<Atom>,
    pub reason: Pattern,
    pub stack: Option<Variable>,
    pub guard: Vec<Guard>,
    pub body: Vec<Expression>,
}

impl CatchClause {
    pub fn new(
        class: Option<Atom>,
        reason: Pattern,
        stack: Option<Variable>,
        guard: Vec<Guard>,
        body: Vec<Expression>,
    ) -> Self {
        Self {
            class,
            reason,
            stack,
            guard,
            body,
        }
    }

    pub fn simple(reason: Pattern, body: Vec<Expression>) -> Self {
        Self::new(None, reason, None, vec![], body)
    }
}

/// List comprehension qualifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Qualifier {
    /// Generator: Pattern <- Expression
    Generator(Pattern, Expression),

    /// Filter: Expression
    Filter(Expression),
}

/// Built-in guard functions (BIFs allowed in guards)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardBif {
    IsAtom(Expression),
    IsBoolean(Expression),
    IsInteger(Expression),
    IsFloat(Expression),
    IsNumber(Expression),
    IsString(Expression),
    IsBinary(Expression),
    IsList(Expression),
    IsTuple(Expression),
    IsMap(Expression),
    IsPid(Expression),
    IsPort(Expression),
    IsReference(Expression),
    IsFunction(Expression, Option<usize>), // arity optional
    IsRecord(Expression, Atom),

    // Type checks
    IsAlive, // for processes

    // Comparison
    Equal(Expression, Expression),
    NotEqual(Expression, Expression),
    Less(Expression, Expression),
    LessEqual(Expression, Expression),
    Greater(Expression, Expression),
    GreaterEqual(Expression, Expression),

    // Arithmetic
    Plus(Expression, Expression),
    Minus(Expression, Expression),
    Multiply(Expression, Expression),
    Divide(Expression, Expression),

    // List operations
    Length(Expression),
    Hd(Expression),
    Tl(Expression),
    Member(Expression, Expression),

    // Other
    Node,
    Self_,
    Size(Expression),
}

impl GuardBif {
    pub fn name(&self) -> &str {
        match self {
            Self::IsAtom(_) => "is_atom",
            Self::IsBoolean(_) => "is_boolean",
            Self::IsInteger(_) => "is_integer",
            Self::IsFloat(_) => "is_float",
            Self::IsNumber(_) => "is_number",
            Self::IsString(_) => "is_string",
            Self::IsBinary(_) => "is_binary",
            Self::IsList(_) => "is_list",
            Self::IsTuple(_) => "is_tuple",
            Self::IsMap(_) => "is_map",
            Self::IsPid(_) => "is_pid",
            Self::IsPort(_) => "is_port",
            Self::IsReference(_) => "is_reference",
            Self::IsFunction(_, _) => "is_function",
            Self::IsRecord(_, _) => "is_record",
            Self::IsAlive => "is_alive",
            Self::Equal(_, _) => "=:=",
            Self::NotEqual(_, _) => "/=",
            Self::Less(_, _) => "<",
            Self::LessEqual(_, _) => "=<",
            Self::Greater(_, _) => ">",
            Self::GreaterEqual(_, _) => ">=",
            Self::Plus(_, _) => "+",
            Self::Minus(_, _) => "-",
            Self::Multiply(_, _) => "*",
            Self::Divide(_, _) => "/",
            Self::Length(_) => "length",
            Self::Hd(_) => "hd",
            Self::Tl(_) => "tl",
            Self::Member(_, _) => "member",
            Self::Node => "node",
            Self::Self_ => "self",
            Self::Size(_) => "size",
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Self::IsAtom(_) |
            Self::IsBoolean(_) |
            Self::IsInteger(_) |
            Self::IsFloat(_) |
            Self::IsNumber(_) |
            Self::IsString(_) |
            Self::IsBinary(_) |
            Self::IsList(_) |
            Self::IsTuple(_) |
            Self::IsMap(_) |
            Self::IsPid(_) |
            Self::IsPort(_) |
            Self::IsReference(_) |
            Self::Length(_) |
            Self::Hd(_) |
            Self::Tl(_) |
            Self::Size(_) |
            Self::Node |
            Self::Self_ |
            Self::IsAlive => 1,

            Self::IsFunction(_, Some(_)) => 2, // with arity
            Self::IsFunction(_, None) => 1,    // without arity
            Self::IsRecord(_, _) => 2,

            Self::Equal(_, _) |
            Self::NotEqual(_, _) |
            Self::Less(_, _) |
            Self::LessEqual(_, _) |
            Self::Greater(_, _) |
            Self::GreaterEqual(_, _) |
            Self::Plus(_, _) |
            Self::Minus(_, _) |
            Self::Multiply(_, _) |
            Self::Divide(_, _) |
            Self::Member(_, _) => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clause_creation() {
        let patterns = vec![
            Pattern::from(Variable::new("X")),
            Pattern::from(Variable::new("Y")),
        ];

        let guard = vec![
            Guard::from(Expression::Literal(Literal::from(true))),
        ];

        let body = vec![
            Expression::BinaryOp(BinaryOp::new(
                BinaryOperator::Plus,
                Expression::Variable(Variable::new("X")),
                Expression::Variable(Variable::new("Y")),
            )),
        ];

        let clause = Clause::new(patterns, guard, body);
        assert_eq!(clause.arity(), 2);
        assert!(clause.has_guards());
    }

    #[test]
    fn test_simple_clause() {
        let patterns = vec![Pattern::from(42)];
        let body = vec![Expression::Literal("ok".into())];

        let clause = Clause::simple(patterns, body);
        assert_eq!(clause.arity(), 1);
        assert!(!clause.has_guards());
    }

    #[test]
    fn test_guard_expressions() {
        let var_x = Expression::Variable(Variable::new("X"));
        let var_y = Expression::Variable(Variable::new("Y"));

        let guard = Guard::BinaryOp(BinaryOp::new(
            BinaryOperator::Greater,
            var_x,
            var_y,
        ));

        assert!(matches!(guard, Guard::BinaryOp(_)));
    }

    #[test]
    fn test_guard_sequences() {
        let guard1 = Guard::from(Expression::Variable(Variable::new("X")));
        let guard2 = Guard::from(Expression::Variable(Variable::new("Y")));

        let sequence = GuardSequence::new(vec![guard1, guard2]);
        assert_eq!(sequence.len(), 2);
        assert!(!sequence.is_empty());
    }

    #[test]
    fn test_case_clause() {
        let pattern = Pattern::from(Variable::new("Result"));
        let body = vec![Expression::Literal("matched".into())];

        let case_clause = CaseClause::simple(pattern, body);
        assert!(case_clause.guard.is_empty()); // Simple case clause has no guards
        assert_eq!(case_clause.body.len(), 1);
    }

    #[test]
    fn test_if_clause() {
        let guard = vec![
            Guard::from(Expression::Variable(Variable::new("X"))),
        ];
        let body = vec![Expression::Literal(1.into())];

        let if_clause = IfClause::new(guard, body);
        assert!(if_clause.has_guards());
    }

    #[test]
    fn test_catch_clause() {
        let reason = Pattern::from(Variable::new("Error"));
        let body = vec![Expression::Literal("caught".into())];

        let catch_clause = CatchClause::simple(reason, body);
        assert!(catch_clause.class.is_none());
        assert!(catch_clause.stack.is_none());
        assert!(catch_clause.guard.is_empty());
    }

    #[test]
    fn test_list_comprehension_qualifiers() {
        let generator = Qualifier::Generator(
            Pattern::from(Variable::new("X")),
            Expression::Variable(Variable::new("List")),
        );

        let filter = Qualifier::Filter(Expression::BinaryOp(BinaryOp::new(
            BinaryOperator::Greater,
            Expression::Variable(Variable::new("X")),
            Expression::Literal(0.into()),
        )));

        assert!(matches!(generator, Qualifier::Generator(_, _)));
        assert!(matches!(filter, Qualifier::Filter(_)));
    }

    #[test]
    fn test_guard_bifs() {
        let is_atom = GuardBif::IsAtom(Expression::Variable(Variable::new("X")));
        assert_eq!(is_atom.name(), "is_atom");
        assert_eq!(is_atom.arity(), 1);

        let equal = GuardBif::Equal(
            Expression::Variable(Variable::new("A")),
            Expression::Variable(Variable::new("B")),
        );
        assert_eq!(equal.name(), "=:=");
        assert_eq!(equal.arity(), 2);

        let is_function = GuardBif::IsFunction(
            Expression::Variable(Variable::new("F")),
            Some(2),
        );
        assert_eq!(is_function.name(), "is_function");
        assert_eq!(is_function.arity(), 2);

        let is_function_no_arity = GuardBif::IsFunction(
            Expression::Variable(Variable::new("F")),
            None,
        );
        assert_eq!(is_function_no_arity.arity(), 1);
    }
}
