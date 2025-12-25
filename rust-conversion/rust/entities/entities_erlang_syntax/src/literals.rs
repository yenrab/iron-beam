/*!
# Erlang Literal Values

Literals are the atomic values in Erlang: integers, floats, strings, binaries, etc.
These represent the basic data types that can appear in Erlang source code.
*/

use std::fmt;
use num_traits::cast::ToPrimitive;
use super::atoms::Atom;

/// Integer literal (can be arbitrary precision in Erlang)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Integer {
    pub value: num_bigint::BigInt,
}

impl Integer {
    pub fn new<I: Into<num_bigint::BigInt>>(value: I) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn from_i64(value: i64) -> Self {
        Self::new(value)
    }

    pub fn from_u64(value: u64) -> Self {
        Self::new(value)
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.value.to_i64()
    }

    pub fn as_u64(&self) -> Option<u64> {
        self.value.to_u64()
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}


/// Float literal (IEEE 754 double precision)
/// Note: Uses f64 to match Erlang's float representation.
/// For rational arithmetic, see the Rational type below.
#[derive(Debug, Clone)]
pub struct Float {
    pub value: f64,
}

/// Rational number literal (exact rational arithmetic)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rational {
    pub value: num_rational::BigRational,
}

impl Rational {
    pub fn new(numerator: num_bigint::BigInt, denominator: num_bigint::BigInt) -> Self {
        Self {
            value: num_rational::BigRational::new(numerator, denominator),
        }
    }

    pub fn from_ints(numerator: i64, denominator: i64) -> Self {
        Self::new(numerator.into(), denominator.into())
    }
}


impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        // Use epsilon comparison for floating point
        (self.value - other.value).abs() < f64::EPSILON
    }
}

impl Eq for Float {}

impl Float {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}


/// String literal (list of integers in Erlang)
#[derive(Debug, Clone, PartialEq, Eq, Hash, )]
pub struct StringLiteral {
    pub value: String,
}

impl StringLiteral {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Convert to Erlang charlist representation
    pub fn to_charlist(&self) -> Vec<Integer> {
        self.value.chars()
            .map(|c| Integer::from_i64(c as i64))
            .collect()
    }
}

impl fmt::Display for StringLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value.escape_default())
    }
}

/// Binary literal (Erlang binaries)
#[derive(Debug, Clone, PartialEq, Eq, Hash, )]
pub struct Binary {
    pub segments: Vec<BinarySegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, )]
pub struct BinarySegment {
    pub value: Vec<u8>,
    pub size: Option<Integer>,
    pub unit: Option<Integer>,
}


impl Binary {
    pub fn new(segments: Vec<BinarySegment>) -> Self {
        Self { segments }
    }

    pub fn simple(bytes: Vec<u8>) -> Self {
        Self {
            segments: vec![BinarySegment {
                value: bytes,
                size: None,
                unit: None,
            }],
        }
    }
}

/// Character literal
#[derive(Debug, Clone, PartialEq, Eq, Hash, )]
pub struct Char {
    pub value: char,
}

impl Char {
    pub fn new(value: char) -> Self {
        Self { value }
    }

    pub fn as_codepoint(&self) -> u32 {
        self.value as u32
    }
}

impl fmt::Display for Char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${}", self.value.escape_default())
    }
}

/// List literal
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum List {
    Proper(Vec<Literal>),
    Improper(Vec<Literal>, Box<Literal>),
}

impl List {
    pub fn proper(elements: Vec<Literal>) -> Self {
        Self::Proper(elements)
    }

    pub fn improper(elements: Vec<Literal>, tail: Literal) -> Self {
        Self::Improper(elements, Box::new(tail))
    }

    pub fn is_proper(&self) -> bool {
        matches!(self, Self::Proper(_))
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Proper(elements) => elements.len(),
            Self::Improper(elements, _) => elements.len(),
        }
    }
}

/// Tuple literal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tuple {
    pub elements: Vec<Literal>,
}

impl Tuple {
    pub fn new(elements: Vec<Literal>) -> Self {
        Self { elements }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

/// Map literal (Erlang maps)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    pub entries: Vec<MapEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapEntry {
    pub key: Literal,
    pub value: Literal,
}

impl Map {
    pub fn new(entries: Vec<MapEntry>) -> Self {
        Self { entries }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Record literal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub name: super::Atom,
    pub fields: Vec<RecordField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordField {
    pub name: super::Atom,
    pub value: Literal,
}

impl Record {
    pub fn new(name: super::Atom, fields: Vec<RecordField>) -> Self {
        Self { name, fields }
    }
}

/// All literal types in Erlang
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Atom(super::Atom),
    Integer(Integer),
    Float(Float),
    Rational(Rational),
    String(StringLiteral),
    Binary(Binary),
    Char(Char),
    List(List),
    Tuple(Tuple),
    Map(Map),
    Record(Record),
}

impl From<super::Atom> for Literal {
    fn from(atom: super::Atom) -> Self {
        Self::Atom(atom)
    }
}

impl From<Integer> for Literal {
    fn from(int: Integer) -> Self {
        Self::Integer(int)
    }
}

impl From<Float> for Literal {
    fn from(float: Float) -> Self {
        Self::Float(float)
    }
}

impl From<StringLiteral> for Literal {
    fn from(string: StringLiteral) -> Self {
        Self::String(string)
    }
}

impl From<i64> for Literal {
    fn from(value: i64) -> Self {
        Self::Integer(Integer::from_i64(value))
    }
}

impl From<f64> for Literal {
    fn from(value: f64) -> Self {
        Self::Float(Float::new(value))
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Self::String(StringLiteral::new(value))
    }
}

impl From<&str> for Literal {
    fn from(value: &str) -> Self {
        Self::String(StringLiteral::new(value))
    }
}

impl From<Rational> for Literal {
    fn from(rational: Rational) -> Self {
        Self::Rational(rational)
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Self::Atom(Atom::from(if value { "true" } else { "false" }))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    #[test]
    fn test_integer_literal() {
        let int = Integer::from_i64(42);
        assert_eq!(int.as_i64(), Some(42));

        let big_int = Integer::new(BigInt::from(999999999999i64));
        assert_eq!(big_int.to_string(), "999999999999");
    }

    #[test]
    fn test_float_literal() {
        let float = Float::new(3.14159);
        assert_eq!(float.value, 3.14159);
        assert_eq!(float.to_string(), "3.14159");
    }

    #[test]
    fn test_string_literal() {
        let string = StringLiteral::new("hello world");
        assert_eq!(string.as_str(), "hello world");
        assert_eq!(string.to_string(), "\"hello world\"");

        let charlist = string.to_charlist();
        assert_eq!(charlist.len(), 11);
        assert_eq!(charlist[0].as_i64(), Some('h' as i64));
    }

    #[test]
    fn test_list_literals() {
        let elements = vec![
            Literal::from(1),
            Literal::from(2),
            Literal::from(3),
        ];

        let proper_list = List::proper(elements.clone());
        assert!(proper_list.is_proper());
        assert_eq!(proper_list.len(), 3);

        let improper_list = List::improper(elements, Literal::from(4));
        assert!(!improper_list.is_proper());
        assert_eq!(improper_list.len(), 3);
    }

    #[test]
    fn test_tuple_literal() {
        let elements = vec![
            Literal::from("hello"),
            Literal::from(42),
        ];

        let tuple = Tuple::new(elements);
        assert_eq!(tuple.len(), 2);
    }

    #[test]
    fn test_map_literal() {
        let entries = vec![
            MapEntry {
                key: Literal::from("key1"),
                value: Literal::from(1),
            },
            MapEntry {
                key: Literal::from("key2"),
                value: Literal::from(2),
            },
        ];

        let map = Map::new(entries);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_rational_literal() {
        let rational = Rational::from_ints(3, 4);
        assert_eq!(rational.to_string(), "3/4");

        let rational_lit: Literal = rational.into();
        assert!(matches!(rational_lit, Literal::Rational(_)));
    }

    #[test]
    fn test_literal_conversions() {
        let atom_lit: Literal = crate::Atom::new("test").into();
        assert!(matches!(atom_lit, Literal::Atom(_)));

        let int_lit: Literal = 42.into();
        assert!(matches!(int_lit, Literal::Integer(_)));

        let float_lit: Literal = 3.14.into();
        assert!(matches!(float_lit, Literal::Float(_)));

        let string_lit: Literal = "hello".into();
        assert!(matches!(string_lit, Literal::String(_)));
    }

    #[test]
    fn test_char_literal() {
        let char_lit = Char::new('A');
        assert_eq!(char_lit.as_codepoint(), 65);
        assert_eq!(char_lit.to_string(), "$A");
    }
}
