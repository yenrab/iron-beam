/*!
# Erlang Module Definitions

Modules are the top-level organizational units in Erlang. They contain functions,
attributes, and type specifications that define the module's interface and implementation.
*/

use super::*;

/// Erlang module definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: Atom,
    pub attributes: Vec<Attribute>,
    pub type_specs: Vec<TypeSpec>,
    pub functions: Vec<Function>,
    pub eof_marker: Option<String>, // Optional end-of-file comment
}

impl Module {
    pub fn new(name: Atom) -> Self {
        Self {
            name,
            attributes: Vec::new(),
            type_specs: Vec::new(),
            functions: Vec::new(),
            eof_marker: None,
        }
    }

    pub fn with_attributes(mut self, attributes: Vec<Attribute>) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn with_functions(mut self, functions: Vec<Function>) -> Self {
        self.functions = functions;
        self
    }

    pub fn add_attribute(&mut self, attribute: Attribute) {
        self.attributes.push(attribute);
    }

    pub fn add_function(&mut self, function: Function) {
        self.functions.push(function);
    }

    pub fn add_type_spec(&mut self, type_spec: TypeSpec) {
        self.type_specs.push(type_spec);
    }

    /// Find a function by name and arity
    pub fn find_function(&self, name: &Atom, arity: usize) -> Option<&Function> {
        self.functions.iter().find(|f| f.name.atom == *name && f.name.arity == arity)
    }

    /// Get all exported functions
    pub fn exported_functions(&self) -> Vec<&Function> {
        let exported_names: std::collections::HashSet<_> = self.export_attributes()
            .filter_map(|attr| {
                if let AttributeValue::Export(functions) = &attr.value {
                    Some(functions.clone())
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        self.functions.iter()
            .filter(|f| exported_names.contains(&f.name))
            .collect()
    }

    /// Get all export attributes
    pub fn export_attributes(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes.iter().filter(|attr| matches!(attr.value, AttributeValue::Export(_)))
    }

    /// Get module attribute by name
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|attr| attr.name.as_str() == name)
    }
}

/// Module attributes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: Atom,
    pub value: AttributeValue,
}

impl Attribute {
    pub fn new(name: Atom, value: AttributeValue) -> Self {
        Self { name, value }
    }
}

/// Attribute values
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    /// -module(name).
    Module(Atom),

    /// -export([function/arity, ...]).
    Export(Vec<FunctionName>),

    /// -import(module, [function/arity, ...]).
    Import(Atom, Vec<FunctionName>),

    /// -compile(options).
    Compile(Vec<CompileOption>),

    /// -vsn(version).
    Version(Literal),

    /// -author(name).
    Author(StringLiteral),

    /// -behaviour(behavior).
    Behaviour(Atom),

    /// -callback(name, type_spec).
    Callback(Atom, FunctionType),

    /// -spec(name, type_spec).
    Spec(FunctionName, FunctionType),

    /// -type(name, type_definition).
    Type(Atom, Type),

    /// -opaque(name, type_definition).
    Opaque(Atom, Type),

    /// -record(name, fields).
    Record(Atom, Vec<RecordFieldDef>),

    /// -include("file.hrl").
    Include(StringLiteral),

    /// -include_lib("library/file.hrl").
    IncludeLib(StringLiteral),

    /// -define(name, value).
    Define(Atom, Expression),

    /// -ifdef(name).
    Ifdef(Atom),

    /// -ifndef(name).
    Ifndef(Atom),

    /// -else.
    Else,

    /// -endif.
    Endif,

    /// -on_load(function/arity).
    OnLoad(FunctionName),

    /// Custom attribute
    Custom(Expression),
}

/// Record field definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordFieldDef {
    pub name: Atom,
    pub type_: Option<Type>,
    pub default: Option<Expression>,
}

impl RecordFieldDef {
    pub fn new(name: Atom) -> Self {
        Self {
            name,
            type_: None,
            default: None,
        }
    }

    pub fn with_type(mut self, type_: Type) -> Self {
        self.type_ = Some(type_);
        self
    }

    pub fn with_default(mut self, default: Expression) -> Self {
        self.default = Some(default);
        self
    }
}

/// Compilation options
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileOption {
    ExportAll,
    NoAutoImport,
    Inline(Vec<FunctionName>),
    Optimize(usize), // 0-999
    Warnings(bool),
    Verbose(bool),
    DebugInfo(bool),
    Custom(Atom, Option<Expression>),
}

/// Function definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: FunctionName,
    pub clauses: Vec<Clause>,
}

impl Function {
    pub fn new(name: FunctionName, clauses: Vec<Clause>) -> Self {
        Self { name, clauses }
    }

    pub fn arity(&self) -> usize {
        self.name.arity
    }

    /// Check if function has a type specification
    pub fn has_type_spec(&self) -> bool {
        // In a real implementation, this would check against module's type specs
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let module = Module::new(Atom::new("test_module"));
        assert_eq!(module.name.as_str(), "test_module");
        assert!(module.attributes.is_empty());
        assert!(module.functions.is_empty());
    }

    #[test]
    fn test_module_with_functions() {
        let mut module = Module::new(Atom::new("math"));

        let add_function = Function::new(
            FunctionName::from_str("add", 2),
            vec![], // Would have clauses
        );

        module.add_function(add_function);

        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name.atom.as_str(), "add");
        assert_eq!(module.functions[0].arity(), 2);
    }

    #[test]
    fn test_export_attribute() {
        let export_attr = Attribute::new(
            Atom::new("export"),
            AttributeValue::Export(vec![
                FunctionName::from_str("add", 2),
                FunctionName::from_str("multiply", 2),
            ]),
        );

        assert_eq!(export_attr.name.as_str(), "export");
        if let AttributeValue::Export(functions) = &export_attr.value {
            assert_eq!(functions.len(), 2);
            assert_eq!(functions[0].atom.as_str(), "add");
            assert_eq!(functions[1].atom.as_str(), "multiply");
        }
    }

    #[test]
    fn test_module_attribute() {
        let module_attr = Attribute::new(
            Atom::new("module"),
            AttributeValue::Module(Atom::new("my_module")),
        );

        assert_eq!(module_attr.name.as_str(), "module");
        if let AttributeValue::Module(name) = &module_attr.value {
            assert_eq!(name.as_str(), "my_module");
        }
    }

    #[test]
    fn test_record_definition() {
        let fields = vec![
            RecordFieldDef::new(Atom::new("name"))
                .with_type(Type::Builtin(BuiltinType::String)),
            RecordFieldDef::new(Atom::new("age"))
                .with_type(Type::Builtin(BuiltinType::Integer))
                .with_default(Expression::Literal(0.into())),
        ];

        let record_attr = Attribute::new(
            Atom::new("record"),
            AttributeValue::Record(Atom::new("person"), fields),
        );

        if let AttributeValue::Record(name, fields) = &record_attr.value {
            assert_eq!(name.as_str(), "person");
            assert_eq!(fields.len(), 2);
            assert!(fields[0].default.is_none());
            assert!(fields[1].default.is_some());
        }
    }

    #[test]
    fn test_compile_options() {
        let options = vec![
            CompileOption::ExportAll,
            CompileOption::Warnings(true),
            CompileOption::Optimize(999),
        ];

        let compile_attr = Attribute::new(
            Atom::new("compile"),
            AttributeValue::Compile(options),
        );

        if let AttributeValue::Compile(opts) = &compile_attr.value {
            assert_eq!(opts.len(), 3);
        }
    }

    #[test]
    fn test_type_spec_attribute() {
        let func_type = FunctionType::new(
            vec![Type::Builtin(BuiltinType::Integer)],
            Type::Builtin(BuiltinType::Integer),
        );

        let spec_attr = Attribute::new(
            Atom::new("spec"),
            AttributeValue::Spec(
                FunctionName::from_str("double", 1),
                func_type,
            ),
        );

        if let AttributeValue::Spec(func_name, _) = &spec_attr.value {
            assert_eq!(func_name.atom.as_str(), "double");
            assert_eq!(func_name.arity, 1);
        }
    }

    #[test]
    fn test_module_find_function() {
        let mut module = Module::new(Atom::new("test"));

        let func1 = Function::new(FunctionName::from_str("func", 1), vec![]);
        let func2 = Function::new(FunctionName::from_str("func", 2), vec![]);

        module.add_function(func1);
        module.add_function(func2);

        let found = module.find_function(&Atom::new("func"), 1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().arity(), 1);

        let not_found = module.find_function(&Atom::new("missing"), 1);
        assert!(not_found.is_none());
    }
}
