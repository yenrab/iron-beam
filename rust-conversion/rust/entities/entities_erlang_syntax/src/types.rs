/*!
# Erlang Type Specifications

Type specifications in Erlang are used for documentation and static analysis.
They define the expected types of functions and their parameters.
*/

use super::*;

/// Type specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Built-in types
    Builtin(BuiltinType),

    /// Literal type (singleton types)
    Literal(Literal),

    /// Union of types
    Union(Vec<Type>),

    /// Tuple type
    Tuple(Vec<Type>),

    /// List type
    List(Box<Type>),

    /// Map type
    Map(Vec<MapTypeEntry>),

    /// Record type
    Record(RecordType),

    /// Function type
    Function(FunctionType),

    /// User-defined type
    UserDefined(UserDefinedType),

    /// Type variable (for polymorphic types)
    Variable(String),

    /// Annotated type (for type annotations)
    Annotated(Box<Type>, String),

    /// Any type (top type)
    Any,

    /// None type (bottom type)
    None,
}

/// Built-in Erlang types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinType {
    Atom,
    Integer,
    Float,
    String, // charlist
    Binary,
    Boolean, // true | false
    Byte,    // 0..255
    Char,    // Unicode character
    Number,  // integer | float
    Pid,
    Port,
    Reference,
    Term,    // any()
}

impl BuiltinType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "atom" => Some(Self::Atom),
            "integer" => Some(Self::Integer),
            "float" => Some(Self::Float),
            "string" => Some(Self::String),
            "binary" => Some(Self::Binary),
            "boolean" => Some(Self::Boolean),
            "byte" => Some(Self::Byte),
            "char" => Some(Self::Char),
            "number" => Some(Self::Number),
            "pid" => Some(Self::Pid),
            "port" => Some(Self::Port),
            "reference" => Some(Self::Reference),
            "term" => Some(Self::Term),
            _ => None,
        }
    }
}

/// Map type entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapTypeEntry {
    pub key: Type,
    pub value: Type,
}

/// Record type specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordType {
    pub name: Atom,
    pub fields: Vec<RecordTypeField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordTypeField {
    pub name: Atom,
    pub type_: Type,
}

/// Function type specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub parameters: Vec<Type>,
    pub return_type: Box<Type>,
}

impl FunctionType {
    pub fn new(parameters: Vec<Type>, return_type: Type) -> Self {
        Self {
            parameters,
            return_type: Box::new(return_type),
        }
    }
}

/// User-defined type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDefinedType {
    pub module: Option<Atom>,
    pub name: Atom,
    pub parameters: Vec<Type>,
}

impl UserDefinedType {
    pub fn local(name: Atom, parameters: Vec<Type>) -> Self {
        Self {
            module: None,
            name,
            parameters,
        }
    }

    pub fn external(module: Atom, name: Atom, parameters: Vec<Type>) -> Self {
        Self {
            module: Some(module),
            name,
            parameters,
        }
    }
}

/// Type specification with optional constraints
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSpec {
    pub name: FunctionName,
    pub type_: FunctionType,
    pub constraints: Vec<Constraint>,
}

/// Type constraint for polymorphic types
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    pub variable: String,
    pub type_: Type,
}

/// Type guard function for runtime type checks
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeGuard {
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
    IsFunction(Expression),
    IsRecord(Expression, Atom),
}

impl TypeGuard {
    pub fn expression(&self) -> &Expression {
        match self {
            Self::IsAtom(e) |
            Self::IsBoolean(e) |
            Self::IsInteger(e) |
            Self::IsFloat(e) |
            Self::IsNumber(e) |
            Self::IsString(e) |
            Self::IsBinary(e) |
            Self::IsList(e) |
            Self::IsTuple(e) |
            Self::IsMap(e) |
            Self::IsPid(e) |
            Self::IsPort(e) |
            Self::IsReference(e) |
            Self::IsFunction(e) => e,
            Self::IsRecord(e, _) => e,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_types() {
        assert_eq!(BuiltinType::from_str("atom"), Some(BuiltinType::Atom));
        assert_eq!(BuiltinType::from_str("integer"), Some(BuiltinType::Integer));
        assert_eq!(BuiltinType::from_str("unknown"), None);
    }

    #[test]
    fn test_function_type() {
        let func_type = FunctionType::new(
            vec![Type::Builtin(BuiltinType::Integer), Type::Builtin(BuiltinType::Integer)],
            Type::Builtin(BuiltinType::Integer),
        );

        assert_eq!(func_type.parameters.len(), 2);
    }

    #[test]
    fn test_user_defined_types() {
        let local = UserDefinedType::local(
            Atom::new("my_type"),
            vec![Type::Builtin(BuiltinType::Integer)],
        );
        assert!(local.module.is_none());
        assert_eq!(local.name.as_str(), "my_type");

        let external = UserDefinedType::external(
            Atom::new("mymodule"),
            Atom::new("other_type"),
            vec![],
        );
        assert_eq!(external.module.as_ref().unwrap().as_str(), "mymodule");
    }

    #[test]
    fn test_union_types() {
        let union = Type::Union(vec![
            Type::Builtin(BuiltinType::Atom),
            Type::Literal(Literal::Atom(Atom::new("ok"))),
            Type::Literal(Literal::Atom(Atom::new("error"))),
        ]);

        assert!(matches!(union, Type::Union(_)));
    }

    #[test]
    fn test_tuple_type() {
        let tuple = Type::Tuple(vec![
            Type::Builtin(BuiltinType::Atom),
            Type::Builtin(BuiltinType::Integer),
        ]);

        assert!(matches!(tuple, Type::Tuple(_)));
    }

    #[test]
    fn test_list_type() {
        let list = Type::List(Box::new(Type::Builtin(BuiltinType::Integer)));
        assert!(matches!(list, Type::List(_)));
    }

    #[test]
    fn test_map_type() {
        let map_entries = vec![
            MapTypeEntry {
                key: Type::Builtin(BuiltinType::Atom),
                value: Type::Builtin(BuiltinType::Integer),
            },
        ];

        let map_type = Type::Map(map_entries);
        assert!(matches!(map_type, Type::Map(_)));
    }

    #[test]
    fn test_record_type() {
        let fields = vec![
            RecordTypeField {
                name: Atom::new("name"),
                type_: Type::Builtin(BuiltinType::String),
            },
            RecordTypeField {
                name: Atom::new("age"),
                type_: Type::Builtin(BuiltinType::Integer),
            },
        ];

        let record_type = RecordType {
            name: Atom::new("person"),
            fields,
        };

        assert_eq!(record_type.name.as_str(), "person");
        assert_eq!(record_type.fields.len(), 2);
    }

    #[test]
    fn test_type_spec() {
        let func_type = FunctionType::new(
            vec![Type::Builtin(BuiltinType::Integer)],
            Type::Builtin(BuiltinType::Integer),
        );

        let type_spec = TypeSpec {
            name: FunctionName::from_str("double", 1),
            type_: func_type,
            constraints: vec![],
        };

        assert_eq!(type_spec.name.atom.as_str(), "double");
        assert_eq!(type_spec.name.arity, 1);
    }

    #[test]
    fn test_type_guards() {
        let var_expr = Expression::Variable(Variable::new("X"));

        let is_atom = TypeGuard::IsAtom(var_expr.clone());
        assert!(matches!(is_atom.expression(), Expression::Variable(_)));

        let is_record = TypeGuard::IsRecord(var_expr, Atom::new("person"));
        assert!(matches!(is_record, TypeGuard::IsRecord(_, _)));
    }
}
