/*!
# Erlang Pattern Matching Structures

Patterns are used in Erlang for matching data structures. They appear in function clauses,
case expressions, receive expressions, and other pattern matching contexts.
*/

use super::*;

/// All pattern types in Erlang
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// Variable binding
    Variable(Variable),

    /// Literal value
    Literal(Literal),

    /// Wildcard pattern (_)
    Wildcard,

    /// Tuple pattern
    Tuple(TuplePattern),

    /// List pattern
    List(ListPattern),

    /// Binary pattern
    Binary(BinaryPattern),

    /// Map pattern
    Map(MapPattern),

    /// Record pattern
    Record(RecordPattern),

    /// Unary operator in pattern
    UnaryOp(UnaryOpPattern),

    /// Binary operator in pattern
    BinaryOp(BinaryOpPattern),
}

/// Tuple pattern for destructuring
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuplePattern {
    pub elements: Vec<Pattern>,
}

impl TuplePattern {
    pub fn new(elements: Vec<Pattern>) -> Self {
        Self { elements }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

/// List pattern for destructuring
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPattern {
    pub elements: Vec<Pattern>,
    pub tail: Option<Box<Pattern>>, // For improper lists
}

impl ListPattern {
    pub fn proper(elements: Vec<Pattern>) -> Self {
        Self {
            elements,
            tail: None,
        }
    }

    pub fn improper(elements: Vec<Pattern>, tail: Pattern) -> Self {
        Self {
            elements,
            tail: Some(Box::new(tail)),
        }
    }
}

/// Binary pattern for bit-level matching
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryPattern {
    pub segments: Vec<BinarySegmentPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinarySegmentPattern {
    pub pattern: Pattern,
    pub size: Option<Pattern>,
    pub unit: Option<Pattern>,
}

/// Map pattern for matching map contents
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapPattern {
    pub entries: Vec<MapEntryPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapEntryPattern {
    pub key: Pattern,
    pub value: Pattern,
}

/// Record pattern for matching record contents
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordPattern {
    pub name: Atom,
    pub fields: Vec<RecordFieldPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordFieldPattern {
    pub name: Atom,
    pub pattern: Pattern,
}

/// Unary operator in patterns (rare but possible)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryOpPattern {
    pub operator: UnaryOperator,
    pub operand: Box<Pattern>,
}

/// Binary operator in patterns (rare but possible)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryOpPattern {
    pub operator: BinaryOperator,
    pub left: Box<Pattern>,
    pub right: Box<Pattern>,
}

impl From<Variable> for Pattern {
    fn from(var: Variable) -> Self {
        Self::Variable(var)
    }
}

impl From<Literal> for Pattern {
    fn from(lit: Literal) -> Self {
        Self::Literal(lit)
    }
}

impl From<&str> for Pattern {
    fn from(s: &str) -> Self {
        Self::Literal(Literal::from(s))
    }
}

impl From<Atom> for Pattern {
    fn from(atom: Atom) -> Self {
        Self::Literal(Literal::Atom(atom))
    }
}

impl From<i64> for Pattern {
    fn from(value: i64) -> Self {
        Self::Literal(value.into())
    }
}

/// Pattern matching result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchResult {
    /// Pattern matched successfully with bindings
    Success(Bindings),
    /// Pattern did not match
    Failure,
}

/// Variable bindings from pattern matching
pub type Bindings = std::collections::HashMap<Variable, Literal>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_pattern() {
        let pattern = Pattern::from(Variable::new("X"));
        assert!(matches!(pattern, Pattern::Variable(_)));
    }

    #[test]
    fn test_literal_pattern() {
        let pattern = Pattern::from(42);
        assert!(matches!(pattern, Pattern::Literal(Literal::Integer(_))));
    }

    #[test]
    fn test_atom_pattern() {
        let pattern = Pattern::from(Atom::new("ok"));
        if let Pattern::Literal(Literal::Atom(atom)) = pattern {
            assert_eq!(atom.as_str(), "ok");
        } else {
            panic!("Expected atom literal");
        }
    }

    #[test]
    fn test_tuple_pattern() {
        let elements = vec![
            Pattern::from(Variable::new("A")),
            Pattern::from(42),
            Pattern::Wildcard,
        ];

        let tuple_pattern = TuplePattern::new(elements);
        assert_eq!(tuple_pattern.len(), 3);

        let pattern = Pattern::Tuple(tuple_pattern);
        assert!(matches!(pattern, Pattern::Tuple(_)));
    }

    #[test]
    fn test_list_patterns() {
        let elements = vec![
            Pattern::from(1),
            Pattern::from(2),
        ];

        let proper = ListPattern::proper(elements.clone());
        assert!(proper.tail.is_none());

        let improper = ListPattern::improper(elements, Pattern::from(Variable::new("Tail")));
        assert!(improper.tail.is_some());
    }

    #[test]
    fn test_map_pattern() {
        let entries = vec![
            MapEntryPattern {
                key: Pattern::from("key1"),
                value: Pattern::from(Variable::new("Value1")),
            },
            MapEntryPattern {
                key: Pattern::from("key2"),
                value: Pattern::from(42),
            },
        ];

        let map_pattern = MapPattern { entries };
        assert_eq!(map_pattern.entries.len(), 2);

        let pattern = Pattern::Map(map_pattern);
        assert!(matches!(pattern, Pattern::Map(_)));
    }

    #[test]
    fn test_record_pattern() {
        let fields = vec![
            RecordFieldPattern {
                name: Atom::new("name"),
                pattern: Pattern::from(Variable::new("Name")),
            },
            RecordFieldPattern {
                name: Atom::new("age"),
                pattern: Pattern::from(Variable::new("Age")),
            },
        ];

        let record_pattern = RecordPattern {
            name: Atom::new("person"),
            fields,
        };

        assert_eq!(record_pattern.name.as_str(), "person");
        assert_eq!(record_pattern.fields.len(), 2);
    }

    #[test]
    fn test_wildcard_pattern() {
        let pattern = Pattern::Wildcard;
        assert!(matches!(pattern, Pattern::Wildcard));
    }
}
