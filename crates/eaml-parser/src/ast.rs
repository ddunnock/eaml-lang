//! AST node types, typed arenas, and ID newtypes for the EAML parser.
//!
//! All AST nodes carry a [`Span`] for source location tracking.
//! The [`Ast`] struct uses typed `Vec` arenas with newtype index IDs
//! to prevent cross-arena indexing errors.

use eaml_errors::Span;
use lasso::Spur;
use std::ops::Index;

// ============================================================================
// Typed ID newtypes
// ============================================================================

/// Index into [`Ast::exprs`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub u32);

/// Index into [`Ast::type_exprs`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeExprId(pub u32);

/// Index into [`Ast::models`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelDeclId(pub u32);

/// Index into [`Ast::schemas`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaDeclId(pub u32);

/// Index into [`Ast::prompts`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PromptDeclId(pub u32);

/// Index into [`Ast::tools`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToolDeclId(pub u32);

/// Index into [`Ast::agents`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentDeclId(pub u32);

/// Index into [`Ast::imports`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImportDeclId(pub u32);

/// Index into [`Ast::lets`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LetDeclId(pub u32);

// ============================================================================
// Top-level declaration wrapper
// ============================================================================

/// A reference to any top-level declaration in the AST.
#[derive(Debug, Clone)]
pub enum DeclId {
    Model(ModelDeclId),
    Schema(SchemaDeclId),
    Prompt(PromptDeclId),
    Tool(ToolDeclId),
    Agent(AgentDeclId),
    Import(ImportDeclId),
    Let(LetDeclId),
    /// Error recovery placeholder.
    Error(Span),
}

// ============================================================================
// Program
// ============================================================================

/// The top-level program node, holding all declarations in source order.
#[derive(Debug, Clone)]
pub struct Program {
    pub declarations: Vec<DeclId>,
    pub span: Span,
}

// ============================================================================
// Ast arena
// ============================================================================

/// Typed arena storage for all AST nodes.
#[derive(Debug, Clone)]
pub struct Ast {
    pub models: Vec<ModelDecl>,
    pub schemas: Vec<SchemaDecl>,
    pub prompts: Vec<PromptDecl>,
    pub tools: Vec<ToolDecl>,
    pub agents: Vec<AgentDecl>,
    pub imports: Vec<ImportDecl>,
    pub lets: Vec<LetDecl>,
    pub exprs: Vec<Expr>,
    pub type_exprs: Vec<TypeExpr>,
}

impl Ast {
    /// Creates a new empty AST with no allocated nodes.
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            schemas: Vec::new(),
            prompts: Vec::new(),
            tools: Vec::new(),
            agents: Vec::new(),
            imports: Vec::new(),
            lets: Vec::new(),
            exprs: Vec::new(),
            type_exprs: Vec::new(),
        }
    }

    pub fn alloc_expr(&mut self, expr: Expr) -> ExprId {
        let id = ExprId(self.exprs.len() as u32);
        self.exprs.push(expr);
        id
    }

    pub fn alloc_type_expr(&mut self, type_expr: TypeExpr) -> TypeExprId {
        let id = TypeExprId(self.type_exprs.len() as u32);
        self.type_exprs.push(type_expr);
        id
    }

    pub fn alloc_model(&mut self, decl: ModelDecl) -> ModelDeclId {
        let id = ModelDeclId(self.models.len() as u32);
        self.models.push(decl);
        id
    }

    pub fn alloc_schema(&mut self, decl: SchemaDecl) -> SchemaDeclId {
        let id = SchemaDeclId(self.schemas.len() as u32);
        self.schemas.push(decl);
        id
    }

    pub fn alloc_prompt(&mut self, decl: PromptDecl) -> PromptDeclId {
        let id = PromptDeclId(self.prompts.len() as u32);
        self.prompts.push(decl);
        id
    }

    pub fn alloc_tool(&mut self, decl: ToolDecl) -> ToolDeclId {
        let id = ToolDeclId(self.tools.len() as u32);
        self.tools.push(decl);
        id
    }

    pub fn alloc_agent(&mut self, decl: AgentDecl) -> AgentDeclId {
        let id = AgentDeclId(self.agents.len() as u32);
        self.agents.push(decl);
        id
    }

    pub fn alloc_import(&mut self, decl: ImportDecl) -> ImportDeclId {
        let id = ImportDeclId(self.imports.len() as u32);
        self.imports.push(decl);
        id
    }

    pub fn alloc_let(&mut self, decl: LetDecl) -> LetDeclId {
        let id = LetDeclId(self.lets.len() as u32);
        self.lets.push(decl);
        id
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Index impls
// ============================================================================

impl Index<ExprId> for Ast {
    type Output = Expr;
    fn index(&self, id: ExprId) -> &Expr {
        &self.exprs[id.0 as usize]
    }
}

impl Index<TypeExprId> for Ast {
    type Output = TypeExpr;
    fn index(&self, id: TypeExprId) -> &TypeExpr {
        &self.type_exprs[id.0 as usize]
    }
}

impl Index<ModelDeclId> for Ast {
    type Output = ModelDecl;
    fn index(&self, id: ModelDeclId) -> &ModelDecl {
        &self.models[id.0 as usize]
    }
}

impl Index<SchemaDeclId> for Ast {
    type Output = SchemaDecl;
    fn index(&self, id: SchemaDeclId) -> &SchemaDecl {
        &self.schemas[id.0 as usize]
    }
}

impl Index<PromptDeclId> for Ast {
    type Output = PromptDecl;
    fn index(&self, id: PromptDeclId) -> &PromptDecl {
        &self.prompts[id.0 as usize]
    }
}

impl Index<ToolDeclId> for Ast {
    type Output = ToolDecl;
    fn index(&self, id: ToolDeclId) -> &ToolDecl {
        &self.tools[id.0 as usize]
    }
}

impl Index<AgentDeclId> for Ast {
    type Output = AgentDecl;
    fn index(&self, id: AgentDeclId) -> &AgentDecl {
        &self.agents[id.0 as usize]
    }
}

impl Index<ImportDeclId> for Ast {
    type Output = ImportDecl;
    fn index(&self, id: ImportDeclId) -> &ImportDecl {
        &self.imports[id.0 as usize]
    }
}

impl Index<LetDeclId> for Ast {
    type Output = LetDecl;
    fn index(&self, id: LetDeclId) -> &LetDecl {
        &self.lets[id.0 as usize]
    }
}

// ============================================================================
// Declaration node structs
// ============================================================================

/// A `model` declaration.
#[derive(Debug, Clone)]
pub struct ModelDecl {
    pub name: Spur,
    pub model_id: TemplateString,
    pub provider: TemplateString,
    pub caps: Vec<(Spur, Span)>,
    pub span: Span,
}

/// A `schema` declaration.
#[derive(Debug, Clone)]
pub struct SchemaDecl {
    pub name: Spur,
    pub fields: Vec<FieldDef>,
    pub span: Span,
}

/// A field within a schema declaration.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: Spur,
    pub type_expr: TypeExprId,
    pub span: Span,
}

/// A `prompt` declaration.
#[derive(Debug, Clone)]
pub struct PromptDecl {
    pub name: Spur,
    pub params: Vec<Param>,
    pub requires: Option<RequiresClause>,
    pub return_type: TypeExprId,
    pub body: PromptBody,
    pub span: Span,
}

/// The body of a prompt declaration.
#[derive(Debug, Clone)]
pub struct PromptBody {
    pub fields: Vec<PromptField>,
    pub span: Span,
}

/// A field within a prompt body.
#[derive(Debug, Clone)]
pub enum PromptField {
    User(TemplateString),
    System(TemplateString),
    Temperature(Span),
    MaxTokens(Span),
    MaxRetries(Span),
}

/// A `requires` clause listing capabilities.
#[derive(Debug, Clone)]
pub struct RequiresClause {
    pub caps: Vec<(Spur, Span)>,
    pub span: Span,
}

/// A `tool` declaration.
#[derive(Debug, Clone)]
pub struct ToolDecl {
    pub name: Spur,
    pub params: Vec<Param>,
    pub return_type: TypeExprId,
    pub body: ToolBody,
    pub span: Span,
}

/// The body of a tool declaration.
#[derive(Debug, Clone)]
pub enum ToolBody {
    PythonBridge {
        description: Option<TemplateString>,
        code_span: Span,
        span: Span,
    },
    Native {
        stmts: Vec<ExprId>,
        span: Span,
    },
    Empty(Span),
}

/// An `agent` declaration.
#[derive(Debug, Clone)]
pub struct AgentDecl {
    pub name: Spur,
    pub fields: Vec<AgentField>,
    pub span: Span,
}

/// A field within an agent declaration.
#[derive(Debug, Clone)]
pub enum AgentField {
    Model(Spur, Span),
    Tools(Vec<(Spur, Span)>, Span),
    System(TemplateString),
    MaxTurns(Span),
    OnError(ErrorPolicy, Span),
}

/// Error handling policy for agents.
#[derive(Debug, Clone)]
pub enum ErrorPolicy {
    Fail,
    RetryThenFail { retries_span: Span },
}

/// An `import` declaration.
#[derive(Debug, Clone)]
pub enum ImportDecl {
    Eaml {
        path: TemplateString,
        alias: Option<Spur>,
        span: Span,
    },
    Python {
        module: TemplateString,
        alias: Option<Spur>,
        span: Span,
    },
}

/// A `let` declaration.
#[derive(Debug, Clone)]
pub struct LetDecl {
    pub name: Spur,
    pub type_expr: TypeExprId,
    pub value: ExprId,
    pub span: Span,
}

/// A function/prompt parameter.
#[derive(Debug, Clone)]
pub struct Param {
    pub name: Spur,
    pub type_expr: TypeExprId,
    pub default: Option<ExprId>,
    pub span: Span,
}

// ============================================================================
// Template strings
// ============================================================================

/// A template string with interpolations.
#[derive(Debug, Clone)]
pub struct TemplateString {
    pub parts: Vec<TemplatePart>,
    pub span: Span,
}

/// A part of a template string.
#[derive(Debug, Clone)]
pub enum TemplatePart {
    Text(Span),
    Interpolation(ExprId, Span),
}

// ============================================================================
// Expressions
// ============================================================================

/// An expression in the EAML language.
#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    IntLit(Span),
    FloatLit(Span),
    StringLit(TemplateString),
    BoolLit(bool, Span),
    NullLit(Span),
    // Identifiers
    Ident(Spur, Span),
    // Binary operations
    BinaryOp {
        left: ExprId,
        op: BinOp,
        right: ExprId,
        span: Span,
    },
    // Unary operations
    UnaryOp {
        op: UnaryOp,
        operand: ExprId,
        span: Span,
    },
    // Await
    Await {
        operand: ExprId,
        span: Span,
    },
    // Postfix
    FieldAccess {
        object: ExprId,
        field: Spur,
        span: Span,
    },
    FnCall {
        callee: ExprId,
        args: Vec<Arg>,
        span: Span,
    },
    Index {
        object: ExprId,
        index: ExprId,
        span: Span,
    },
    // Grouping
    Paren {
        inner: ExprId,
        span: Span,
    },
    // Template string in expression position
    TemplateStr(TemplateString),
    // Statements that can appear as expressions
    If {
        condition: ExprId,
        then_block: Vec<ExprId>,
        else_block: Option<Vec<ExprId>>,
        span: Span,
    },
    Return {
        value: Option<ExprId>,
        span: Span,
    },
    Let {
        name: Spur,
        type_expr: TypeExprId,
        value: ExprId,
        span: Span,
    },
    // Error recovery
    Error(Span),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// A function call argument (optionally named).
#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Option<Spur>,
    pub value: ExprId,
    pub span: Span,
}

// ============================================================================
// Type expressions
// ============================================================================

/// A type expression in the EAML language.
#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(Spur, Span),
    Bounded {
        base: Spur,
        params: Vec<BoundParam>,
        span: Span,
    },
    Array(TypeExprId, Span),
    Optional(TypeExprId, Span),
    LiteralUnion {
        members: Vec<Span>,
        span: Span,
    },
    Grouped(TypeExprId, Span),
    Error(Span),
}

impl TypeExpr {
    /// Returns the span of this type expression.
    pub fn span(&self) -> &Span {
        match self {
            TypeExpr::Named(_, span)
            | TypeExpr::Bounded { span, .. }
            | TypeExpr::Array(_, span)
            | TypeExpr::Optional(_, span)
            | TypeExpr::LiteralUnion { span, .. }
            | TypeExpr::Grouped(_, span)
            | TypeExpr::Error(span) => span,
        }
    }
}

/// A parameter in a bounded type expression.
#[derive(Debug, Clone)]
pub struct BoundParam {
    pub name: Option<Spur>,
    pub value_span: Span,
    pub span: Span,
}
