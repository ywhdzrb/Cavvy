use logos::Logos;
use crate::error::{EolResult, lexer_error};
use crate::error::SourceLocation;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*\*/")]
pub enum Token {
    // 关键字
    #[token("public")]
    Public,
    #[token("private")]
    Private,
    #[token("protected")]
    Protected,
    #[token("static")]
    Static,
    #[token("final")]
    Final,
    #[token("abstract")]
    Abstract,
    #[token("native")]
    Native,
    #[token("class")]
    Class,
    #[token("void")]
    Void,
    #[token("int")]
    Int,
    #[token("long")]
    Long,
    #[token("float")]
    Float,
    #[token("double")]
    Double,
    #[token("bool")]
    Bool,
    #[token("string")]
    String,
    #[token("char")]
    Char,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("do")]
    Do,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("new")]
    New,
    #[token("this")]
    This,
    #[token("super")]
    Super,
    
    // 标识符
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    // 字面量
    #[regex(r"-?\d+", |lex| lex.slice().parse::<i64>().ok())]
    IntegerLiteral(Option<i64>),
    
    #[regex(r"-?\d+\.\d+([eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().ok())]
    FloatLiteral(Option<f64>),
    
    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    StringLiteral(String),
    
    #[regex(r"'([^'\\]|\\.)'", |lex| {
        let s = lex.slice();
        s.chars().nth(1)
    })]
    CharLiteral(Option<char>),
    
    // 运算符
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token(">>>")]
    UnsignedShr,
    #[token("~")]
    Tilde,
    
    // 赋值运算符
    #[token("=")]
    Assign,
    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("*=")]
    MulAssign,
    #[token("/=")]
    DivAssign,
    #[token("%=")]
    ModAssign,
    
    // 自增自减
    #[token("++")]
    Inc,
    #[token("--")]
    Dec,
    
    // 分隔符
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    
    // 换行（用于跟踪行号）- 支持 Windows \r\n 和 Unix \n
    #[regex(r"\r?\n")]
    Newline,
}

#[derive(Debug, Clone)]
pub struct TokenWithLocation {
    pub token: Token,
    pub loc: SourceLocation,
}

pub struct Lexer<'a> {
    source: &'a str,
    inner: logos::Lexer<'a, Token>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            inner: Token::lexer(source),
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> EolResult<Vec<TokenWithLocation>> {
        let mut tokens = Vec::new();
        
        while let Some(token_result) = self.inner.next() {
            match token_result {
                Ok(token) => {
                    let span = self.inner.span();
                    let loc = SourceLocation {
                        line: self.line,
                        column: self.column,
                    };
                    
                    // 更新行号和列号
                    if token == Token::Newline {
                        self.line += 1;
                        self.column = 1;
                        continue; // 不保留换行token
                    } else {
                        self.column += span.end - span.start;
                    }
                    
                    tokens.push(TokenWithLocation { token, loc });
                }
                Err(_) => {
                    let span = self.inner.span();
                    let error_char = &self.source[span.clone()];
                    return Err(lexer_error(
                        self.line,
                        self.column,
                        format!("Unexpected character: '{}'", error_char)
                    ));
                }
            }
        }
        
        // 添加EOF标记 - 使用Identifier作为哨兵值
        tokens.push(TokenWithLocation {
            token: Token::Identifier(String::new()), // 用作EOF标记
            loc: SourceLocation {
                line: self.line,
                column: self.column,
            },
        });
        
        Ok(tokens)
    }
}

pub fn lex(source: &str) -> EolResult<Vec<TokenWithLocation>> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}
