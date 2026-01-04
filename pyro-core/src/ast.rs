#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    List,
    Tuple,
    Set,
    Dict,
    ListMutable,
    TupleMutable,
    SetMutable,
    DictMutable,
    UserDefined(String, Vec<Type>),
    Union(Vec<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    LiteralInt(i64),
    LiteralFloat(f64),
    LiteralBool(bool),
    LiteralString(String),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: String,
    },
    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Set(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl {
        name: String,
        typ: Option<Type>,
        value: Expr,
        mutable: bool,
    },
    Expr(Expr),
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Assign {
        name: String,
        value: Expr,
    },
    FnDecl {
        name: String,
        generics: Vec<String>,
        params: Vec<(String, Type)>,
        return_type: Type,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Import(String),
    StructDef {
        name: String,
        generics: Vec<String>,
        fields: Vec<(String, Type)>,
    },
    InterfaceDef {
        name: String,
        generics: Vec<String>,
        methods: Vec<(String, Vec<(String, Type)>, Type)>,
    },
    TypeAlias {
        name: String,
        generics: Vec<String>,
        alias: Type,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
