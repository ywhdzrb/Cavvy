pub mod error;
pub mod types;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod codegen;

use error::EolResult;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(&self, source: &str, output_path: &str) -> EolResult<()> {
        // 1. 词法分析
        let tokens = lexer::lex(source)?;
        
        // 调试：打印所有token
        #[cfg(debug_assertions)]
        {
            println!("Tokens:");
            for (i, t) in tokens.iter().enumerate() {
                println!("  {}: {:?} at {}", i, t.token, t.loc);
            }
            println!();
        }
        
        // 2. 语法分析
        let ast = parser::parse(tokens)?;
        
        // 3. 语义分析
        let mut analyzer = semantic::SemanticAnalyzer::new();
        analyzer.analyze(&ast)?;
        
        // 4. 代码生成 - 生成LLVM IR
        let mut ir_gen = codegen::IRGenerator::new();
        let mut ir = ir_gen.generate(&ast)?;
        
        // 在文件开头插入全局字符串声明
        let global_strings = ir_gen.get_global_strings();
        let mut global_decls = String::new();
        for (s, name) in global_strings {
            let escaped = s.replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\0A")
                .replace("\r", "\\0D")
                .replace("\t", "\\09");
            let len = s.len() + 1;
            global_decls.push_str(&format!("{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1\n", 
                name, len, escaped));
        }
        
        // 在target triple后插入全局声明
        if let Some(pos) = ir.find("target triple") {
            if let Some(newline_pos) = ir[pos..].find('\n') {
                let insert_pos = pos + newline_pos + 1;
                ir.insert_str(insert_pos, &format!("\n{}", global_decls));
            }
        }
        
        // 输出到文件
        std::fs::write(output_path, ir)
            .map_err(|e| error::EolError::Io(e.to_string()))?;
        
        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_lexer() {
        let source = r#"public class hello {
    public static void main() {
        print("Hello, World");
    }
}"#;
        let tokens = lexer::lex(source).unwrap();
        println!("Tokens:");
        for (i, t) in tokens.iter().enumerate() {
            println!("  {}: {:?} at {}", i, t.token, t.loc);
        }
    }

    #[test]
    fn test_hello_parser() {
        let source = r#"public class hello {
    public static void main() {
        print("Hello, World");
    }
}"#;
        let tokens = lexer::lex(source).unwrap();
        let ast = parser::parse(tokens).unwrap();
        println!("AST: {:?}", ast);
    }
}
