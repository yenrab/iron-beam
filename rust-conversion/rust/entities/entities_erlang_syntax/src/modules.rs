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

    // Additional comprehensive tests for all module functionality

    #[test]
    fn test_module_with_attributes() {
        let attributes = vec![
            Attribute::new(Atom::new("module"), AttributeValue::Module(Atom::new("test"))),
            Attribute::new(Atom::new("author"), AttributeValue::Author(StringLiteral::new("Test Author"))),
        ];

        let module = Module::new(Atom::new("test")).with_attributes(attributes);
        assert_eq!(module.attributes.len(), 2);
        assert_eq!(module.attributes[0].name.as_str(), "module");
        assert_eq!(module.attributes[1].name.as_str(), "author");
    }

    #[test]
    fn test_module_with_functions_builder() {
        let functions = vec![
            Function::new(FunctionName::from_str("func1", 1), vec![]),
            Function::new(FunctionName::from_str("func2", 2), vec![]),
        ];

        let module = Module::new(Atom::new("test")).with_functions(functions);
        assert_eq!(module.functions.len(), 2);
        assert_eq!(module.functions[0].arity(), 1);
        assert_eq!(module.functions[1].arity(), 2);
    }

    #[test]
    fn test_module_add_attribute() {
        let mut module = Module::new(Atom::new("test"));
        let attr = Attribute::new(Atom::new("version"), AttributeValue::Version(Literal::Integer(Integer::from_i64(1))));

        module.add_attribute(attr);
        assert_eq!(module.attributes.len(), 1);
        assert_eq!(module.attributes[0].name.as_str(), "version");
    }

    #[test]
    fn test_module_add_type_spec() {
        let mut module = Module::new(Atom::new("test"));
        let func_type = FunctionType::new(
            vec![Type::Builtin(BuiltinType::Integer)],
            Type::Builtin(BuiltinType::Integer),
        );
        let type_spec = TypeSpec {
            name: FunctionName::from_str("double", 1),
            type_: func_type,
            constraints: vec![],
        };

        module.add_type_spec(type_spec);
        assert_eq!(module.type_specs.len(), 1);
    }

    #[test]
    fn test_module_eof_marker() {
        let mut module = Module::new(Atom::new("test"));
        assert!(module.eof_marker.is_none());

        module.eof_marker = Some("%% End of file".to_string());
        assert_eq!(module.eof_marker, Some("%% End of file".to_string()));
    }

    #[test]
    fn test_module_exported_functions() {
        let mut module = Module::new(Atom::new("test"));

        // Add export attribute
        let export_attr = Attribute::new(
            Atom::new("export"),
            AttributeValue::Export(vec![
                FunctionName::from_str("public_func", 1),
                FunctionName::from_str("another_func", 2),
            ]),
        );
        module.add_attribute(export_attr);

        // Add functions
        let public_func = Function::new(FunctionName::from_str("public_func", 1), vec![]);
        let private_func = Function::new(FunctionName::from_str("private_func", 0), vec![]);
        let another_func = Function::new(FunctionName::from_str("another_func", 2), vec![]);

        module.add_function(public_func);
        module.add_function(private_func);
        module.add_function(another_func);

        let exported = module.exported_functions();
        assert_eq!(exported.len(), 2);
        assert!(exported.iter().any(|f| f.name.atom.as_str() == "public_func"));
        assert!(exported.iter().any(|f| f.name.atom.as_str() == "another_func"));
        assert!(!exported.iter().any(|f| f.name.atom.as_str() == "private_func"));
    }

    #[test]
    fn test_module_export_attributes() {
        let mut module = Module::new(Atom::new("test"));

        let export_attr = Attribute::new(
            Atom::new("export"),
            AttributeValue::Export(vec![FunctionName::from_str("func", 1)]),
        );
        let other_attr = Attribute::new(
            Atom::new("module"),
            AttributeValue::Module(Atom::new("test")),
        );

        module.add_attribute(export_attr);
        module.add_attribute(other_attr);

        let export_attrs: Vec<_> = module.export_attributes().collect();
        assert_eq!(export_attrs.len(), 1);
        assert_eq!(export_attrs[0].name.as_str(), "export");
    }

    #[test]
    fn test_module_get_attribute() {
        let mut module = Module::new(Atom::new("test"));

        let module_attr = Attribute::new(
            Atom::new("module"),
            AttributeValue::Module(Atom::new("test")),
        );
        let version_attr = Attribute::new(
            Atom::new("vsn"),
            AttributeValue::Version(Literal::Integer(Integer::from_i64(1))),
        );

        module.add_attribute(module_attr);
        module.add_attribute(version_attr);

        let found_module = module.get_attribute("module");
        assert!(found_module.is_some());
        assert_eq!(found_module.unwrap().name.as_str(), "module");

        let found_version = module.get_attribute("vsn");
        assert!(found_version.is_some());
        assert_eq!(found_version.unwrap().name.as_str(), "vsn");

        let not_found = module.get_attribute("missing");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_attribute_value_import() {
        let import_attr = Attribute::new(
            Atom::new("import"),
            AttributeValue::Import(
                Atom::new("lists"),
                vec![FunctionName::from_str("map", 2), FunctionName::from_str("foldl", 3)],
            ),
        );

        if let AttributeValue::Import(module, functions) = &import_attr.value {
            assert_eq!(module.as_str(), "lists");
            assert_eq!(functions.len(), 2);
            assert_eq!(functions[0].atom.as_str(), "map");
            assert_eq!(functions[1].atom.as_str(), "foldl");
        } else {
            panic!("Expected Import attribute");
        }
    }

    #[test]
    fn test_attribute_value_version() {
        let version_attr = Attribute::new(
            Atom::new("vsn"),
            AttributeValue::Version(Literal::Integer(Integer::from_i64(42))),
        );

        if let AttributeValue::Version(lit) = &version_attr.value {
            assert!(matches!(lit, Literal::Integer(_)));
        } else {
            panic!("Expected Version attribute");
        }
    }

    #[test]
    fn test_attribute_value_author() {
        let author_attr = Attribute::new(
            Atom::new("author"),
            AttributeValue::Author(StringLiteral::new("Jane Doe")),
        );

        if let AttributeValue::Author(name) = &author_attr.value {
            assert_eq!(name.value, "Jane Doe");
        } else {
            panic!("Expected Author attribute");
        }
    }

    #[test]
    fn test_attribute_value_behaviour() {
        let behaviour_attr = Attribute::new(
            Atom::new("behaviour"),
            AttributeValue::Behaviour(Atom::new("gen_server")),
        );

        if let AttributeValue::Behaviour(name) = &behaviour_attr.value {
            assert_eq!(name.as_str(), "gen_server");
        } else {
            panic!("Expected Behaviour attribute");
        }
    }

    #[test]
    fn test_attribute_value_callback() {
        let func_type = FunctionType::new(
            vec![Type::Builtin(BuiltinType::Integer)],
            Type::Builtin(BuiltinType::Atom),
        );

        let callback_attr = Attribute::new(
            Atom::new("callback"),
            AttributeValue::Callback(Atom::new("init"), func_type),
        );

        if let AttributeValue::Callback(name, _) = &callback_attr.value {
            assert_eq!(name.as_str(), "init");
        } else {
            panic!("Expected Callback attribute");
        }
    }

    #[test]
    fn test_attribute_value_type() {
        let type_attr = Attribute::new(
            Atom::new("type"),
            AttributeValue::Type(Atom::new("my_type"), Type::Builtin(BuiltinType::Integer)),
        );

        if let AttributeValue::Type(name, _) = &type_attr.value {
            assert_eq!(name.as_str(), "my_type");
        } else {
            panic!("Expected Type attribute");
        }
    }

    #[test]
    fn test_attribute_value_opaque() {
        let opaque_attr = Attribute::new(
            Atom::new("opaque"),
            AttributeValue::Opaque(Atom::new("secret_type"), Type::Builtin(BuiltinType::Binary)),
        );

        if let AttributeValue::Opaque(name, _) = &opaque_attr.value {
            assert_eq!(name.as_str(), "secret_type");
        } else {
            panic!("Expected Opaque attribute");
        }
    }

    #[test]
    fn test_attribute_value_include() {
        let include_attr = Attribute::new(
            Atom::new("include"),
            AttributeValue::Include(StringLiteral::new("my_header.hrl")),
        );

        if let AttributeValue::Include(path) = &include_attr.value {
            assert_eq!(path.value, "my_header.hrl");
        } else {
            panic!("Expected Include attribute");
        }
    }

    #[test]
    fn test_attribute_value_include_lib() {
        let include_lib_attr = Attribute::new(
            Atom::new("include_lib"),
            AttributeValue::IncludeLib(StringLiteral::new("stdlib/include/qlc.hrl")),
        );

        if let AttributeValue::IncludeLib(path) = &include_lib_attr.value {
            assert_eq!(path.value, "stdlib/include/qlc.hrl");
        } else {
            panic!("Expected IncludeLib attribute");
        }
    }

    #[test]
    fn test_attribute_value_define() {
        let define_attr = Attribute::new(
            Atom::new("define"),
            AttributeValue::Define(
                Atom::new("DEBUG"),
                Expression::Literal(Literal::Atom(Atom::new("true"))),
            ),
        );

        if let AttributeValue::Define(name, _) = &define_attr.value {
            assert_eq!(name.as_str(), "DEBUG");
        } else {
            panic!("Expected Define attribute");
        }
    }

    #[test]
    fn test_attribute_value_conditional_compilation() {
        let ifdef_attr = Attribute::new(Atom::new("ifdef"), AttributeValue::Ifdef(Atom::new("DEBUG")));
        let ifndef_attr = Attribute::new(Atom::new("ifndef"), AttributeValue::Ifndef(Atom::new("RELEASE")));
        let else_attr = Attribute::new(Atom::new("else"), AttributeValue::Else);
        let endif_attr = Attribute::new(Atom::new("endif"), AttributeValue::Endif);

        assert!(matches!(ifdef_attr.value, AttributeValue::Ifdef(_)));
        assert!(matches!(ifndef_attr.value, AttributeValue::Ifndef(_)));
        assert!(matches!(else_attr.value, AttributeValue::Else));
        assert!(matches!(endif_attr.value, AttributeValue::Endif));
    }

    #[test]
    fn test_attribute_value_on_load() {
        let on_load_attr = Attribute::new(
            Atom::new("on_load"),
            AttributeValue::OnLoad(FunctionName::from_str("init_module", 0)),
        );

        if let AttributeValue::OnLoad(func_name) = &on_load_attr.value {
            assert_eq!(func_name.atom.as_str(), "init_module");
            assert_eq!(func_name.arity, 0);
        } else {
            panic!("Expected OnLoad attribute");
        }
    }

    #[test]
    fn test_attribute_value_custom() {
        let custom_expr = Expression::Literal(Literal::Atom(Atom::new("custom_value")));
        let custom_attr = Attribute::new(
            Atom::new("custom_attr"),
            AttributeValue::Custom(custom_expr),
        );

        assert!(matches!(custom_attr.value, AttributeValue::Custom(_)));
    }

    #[test]
    fn test_record_field_def_with_type() {
        let field = RecordFieldDef::new(Atom::new("name"))
            .with_type(Type::Builtin(BuiltinType::String));

        assert_eq!(field.name.as_str(), "name");
        assert!(field.type_.is_some());
        assert!(field.default.is_none());
    }

    #[test]
    fn test_record_field_def_with_default() {
        let field = RecordFieldDef::new(Atom::new("count"))
            .with_default(Expression::Literal(Literal::Integer(Integer::from_i64(0))));

        assert_eq!(field.name.as_str(), "count");
        assert!(field.type_.is_none());
        assert!(field.default.is_some());
    }

    #[test]
    fn test_record_field_def_complete() {
        let field = RecordFieldDef::new(Atom::new("value"))
            .with_type(Type::Builtin(BuiltinType::Integer))
            .with_default(Expression::Literal(Literal::Integer(Integer::from_i64(42))));

        assert_eq!(field.name.as_str(), "value");
        assert!(field.type_.is_some());
        assert!(field.default.is_some());
    }

    #[test]
    fn test_compile_option_variants() {
        // Test all compile option variants
        let export_all = CompileOption::ExportAll;
        assert!(matches!(export_all, CompileOption::ExportAll));

        let no_auto_import = CompileOption::NoAutoImport;
        assert!(matches!(no_auto_import, CompileOption::NoAutoImport));

        let inline = CompileOption::Inline(vec![FunctionName::from_str("func", 1)]);
        assert!(matches!(inline, CompileOption::Inline(_)));

        let optimize = CompileOption::Optimize(500);
        assert!(matches!(optimize, CompileOption::Optimize(500)));

        let warnings = CompileOption::Warnings(false);
        assert!(matches!(warnings, CompileOption::Warnings(false)));

        let verbose = CompileOption::Verbose(true);
        assert!(matches!(verbose, CompileOption::Verbose(true)));

        let debug_info = CompileOption::DebugInfo(true);
        assert!(matches!(debug_info, CompileOption::DebugInfo(true)));

        let custom = CompileOption::Custom(
            Atom::new("custom_option"),
            Some(Expression::Literal(Literal::Atom(Atom::new("value")))),
        );
        assert!(matches!(custom, CompileOption::Custom(_, _)));
    }

    #[test]
    fn test_function_has_type_spec() {
        let function = Function::new(FunctionName::from_str("test", 1), vec![]);
        // Currently always returns false, but tests the method exists
        assert!(!function.has_type_spec());
    }

    #[test]
    fn test_module_empty_state() {
        let module = Module::new(Atom::new("empty"));
        assert_eq!(module.name.as_str(), "empty");
        assert!(module.attributes.is_empty());
        assert!(module.type_specs.is_empty());
        assert!(module.functions.is_empty());
        assert!(module.eof_marker.is_none());
    }

    #[test]
    fn test_module_complex_scenario() {
        let mut module = Module::new(Atom::new("complex_module"));

        // Add module attribute
        module.add_attribute(Attribute::new(
            Atom::new("module"),
            AttributeValue::Module(Atom::new("complex_module")),
        ));

        // Add export attribute
        module.add_attribute(Attribute::new(
            Atom::new("export"),
            AttributeValue::Export(vec![
                FunctionName::from_str("public_api", 1),
            ]),
        ));

        // Add functions
        let public_func = Function::new(FunctionName::from_str("public_api", 1), vec![]);
        let private_func = Function::new(FunctionName::from_str("internal", 2), vec![]);

        module.add_function(public_func);
        module.add_function(private_func);

        // Add type spec
        let func_type = FunctionType::new(
            vec![Type::Builtin(BuiltinType::Integer)],
            Type::Builtin(BuiltinType::Integer),
        );
        let type_spec = TypeSpec {
            name: FunctionName::from_str("public_api", 1),
            type_: func_type,
            constraints: vec![],
        };
        module.add_type_spec(type_spec);

        // Verify state
        assert_eq!(module.attributes.len(), 2);
        assert_eq!(module.functions.len(), 2);
        assert_eq!(module.type_specs.len(), 1);

        // Test finding functions
        assert!(module.find_function(&Atom::new("public_api"), 1).is_some());
        assert!(module.find_function(&Atom::new("internal"), 2).is_some());
        assert!(module.find_function(&Atom::new("missing"), 1).is_none());

        // Test exported functions
        let exported = module.exported_functions();
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].name.atom.as_str(), "public_api");

        // Test getting attributes
        assert!(module.get_attribute("module").is_some());
        assert!(module.get_attribute("export").is_some());
        assert!(module.get_attribute("missing").is_none());
    }

    #[test]
    fn test_attribute_equality() {
        let attr1 = Attribute::new(
            Atom::new("test"),
            AttributeValue::Module(Atom::new("mod")),
        );
        let attr2 = Attribute::new(
            Atom::new("test"),
            AttributeValue::Module(Atom::new("mod")),
        );
        let attr3 = Attribute::new(
            Atom::new("different"),
            AttributeValue::Module(Atom::new("mod")),
        );

        assert_eq!(attr1, attr2);
        assert_ne!(attr1, attr3);
    }

    #[test]
    fn test_record_field_def_equality() {
        let field1 = RecordFieldDef::new(Atom::new("test"))
            .with_type(Type::Builtin(BuiltinType::Integer));
        let field2 = RecordFieldDef::new(Atom::new("test"))
            .with_type(Type::Builtin(BuiltinType::Integer));
        let field3 = RecordFieldDef::new(Atom::new("different"))
            .with_type(Type::Builtin(BuiltinType::Integer));

        assert_eq!(field1, field2);
        assert_ne!(field1, field3);
    }

    #[test]
    fn test_compile_option_equality() {
        let opt1 = CompileOption::Optimize(500);
        let opt2 = CompileOption::Optimize(500);
        let opt3 = CompileOption::Optimize(400);

        assert_eq!(opt1, opt2);
        assert_ne!(opt1, opt3);
    }

    #[test]
    fn test_function_equality() {
        let func1 = Function::new(FunctionName::from_str("test", 1), vec![]);
        let func2 = Function::new(FunctionName::from_str("test", 1), vec![]);
        let func3 = Function::new(FunctionName::from_str("different", 1), vec![]);

        assert_eq!(func1, func2);
        assert_ne!(func1, func3);
    }
}
