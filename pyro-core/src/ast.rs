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
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Call {
        function: Box<Expr>,
        generics: Vec<Type>,
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
    For {
        item_name: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Set {
        object: Expr,
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
    Break,
    Continue,
    Import(String),
    RecordDef {
        name: String,
        generics: Vec<String>,
        fields: Vec<(String, Type)>,
        methods: Vec<Stmt>,
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
    ClassDecl {
        name: String,
        parent: Option<String>,
        methods: Vec<Stmt>,
    },
    Try {
        body: Vec<Stmt>,
        catch_var: Option<String>,
        catch_body: Option<Vec<Stmt>>,
        finally_body: Option<Vec<Stmt>>,
    },
    Raise {
        error: Expr,
        cause: Option<Expr>,
    },
    Go(Box<Expr>),
    Extern {
        func_name: String,
        generics: Vec<String>,
        params: Vec<(String, Type)>,
        return_type: Type,
    },

}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
