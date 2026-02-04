use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    String,
    Char,
    Object(String),
    Array(Box<Type>),
    Function(Box<FunctionType>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
    pub is_static: bool,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub methods: HashMap<String, MethodInfo>,
    pub fields: HashMap<String, FieldInfo>,
    pub parent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub class_name: String,
    pub params: Vec<ParameterInfo>,
    pub return_type: Type,
    pub is_public: bool,
    pub is_static: bool,
    pub is_native: bool,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: Type,
    pub is_public: bool,
    pub is_static: bool,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: Type,
}

impl Type {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            Type::Void => 0,
            Type::Int32 => 4,
            Type::Int64 => 8,
            Type::Float32 => 4,
            Type::Float64 => 8,
            Type::Bool => 1,
            Type::Char => 1,
            Type::String => 8, // 指针大小
            Type::Object(_) => 8, // 引用类型
            Type::Array(_) => 8, // 指针大小
            Type::Function(_) => 8, // 函数指针
        }
    }

    pub fn is_primitive(&self) -> bool {
        matches!(self, 
            Type::Int32 | 
            Type::Int64 | 
            Type::Float32 | 
            Type::Float64 | 
            Type::Bool | 
            Type::Char
        )
    }

    pub fn is_reference_type(&self) -> bool {
        matches!(self, Type::String | Type::Object(_) | Type::Array(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Int32 | Type::Int64)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Int32 => write!(f, "int"),
            Type::Int64 => write!(f, "long"),
            Type::Float32 => write!(f, "float"),
            Type::Float64 => write!(f, "double"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Char => write!(f, "char"),
            Type::Object(name) => write!(f, "{}", name),
            Type::Array(inner) => write!(f, "{}[]", inner),
            Type::Function(func_type) => {
                write!(f, "fn(")?;
                for (i, param) in func_type.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", func_type.return_type)
            }
        }
    }
}

pub struct TypeRegistry {
    pub classes: HashMap<String, ClassInfo>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    pub fn register_class(&mut self, class_info: ClassInfo) -> crate::error::EolResult<()> {
        let name = class_info.name.clone();
        if self.classes.contains_key(&name) {
            return Err(crate::error::semantic_error(
                0, 0,
                format!("Class '{}' already defined", name)
            ));
        }
        self.classes.insert(name, class_info);
        Ok(())
    }

    pub fn get_class(&self, name: &str) -> Option<&ClassInfo> {
        self.classes.get(name)
    }

    pub fn get_method(&self, class_name: &str, method_name: &str) -> Option<&MethodInfo> {
        self.classes.get(class_name)
            .and_then(|c| c.methods.get(method_name))
    }

    pub fn class_exists(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
