// parser.rs

use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::core::{Statement, StatementKind};

/// A very minimal line-based parser for demos.
/// Parses:
//   let x = "hello";
//   console.print(x);
//   func main { ... }
//   }
// Ignores nested blocks for simplicity.
pub fn parse_file(path: &str) -> Vec<Statement> {
    let file = File::open(path).expect("Unable to open script");
    let reader = BufReader::new(file);
    let mut stmts: Vec<Statement> = Vec::new();
    let mut func_stack: Vec<Statement> = Vec::new();

    for (idx, line) in reader.lines().enumerate() {
        let ln = idx + 1; // 1-based
        let raw = line.unwrap().trim().to_string();
        if raw.is_empty() { continue; }

        // Function start: func NAME {
        if raw.starts_with("func ") && raw.ends_with('{') {
            let name = raw
                .trim_start_matches("func ")
                .trim_end_matches('{')
                .trim()
                .to_string();
            let mut stmt = Statement::new(StatementKind::Func, Some(ln));
            stmt.func_name = Some(name);
            func_stack.push(stmt);
            continue;
        }

        // Block end
        if raw == "}" {
            if let Some(mut completed) = func_stack.pop() {
                if func_stack.is_empty() {
                    stmts.push(completed);
                } else {
                    func_stack.last_mut().unwrap().body.push(completed);
                }
            }
            continue;
        }

        // let var = val;
        if raw.starts_with("let ") {
            let parts: Vec<&str> = raw
                .trim_end_matches(';')
                .trim_start_matches("let ")
                .splitn(2, '=')
                .map(str::trim)
                .collect();
            let mut s = Statement::new(StatementKind::Let, Some(ln));
            s.args.push(parts[0].to_string()); // var
            s.args.push(parts[1].to_string()); // expr
            if let Some(ctx) = func_stack.last_mut() {
                ctx.body.push(s);
            } else {
                stmts.push(s);
            }
            continue;
        }

        // console.print(...)
        if raw.starts_with("console.print") {
            let inner = raw
                .trim_start_matches("console.print")
                .trim_start_matches('(')
                .trim_end_matches(");")
                .to_string();
            let mut s = Statement::new(StatementKind::ConsolePrint, Some(ln));
            // split by comma, trim spaces
            for arg in inner.split(',') {
                s.args.push(arg.trim().to_string());
            }
            if let Some(ctx) = func_stack.last_mut() {
                ctx.body.push(s);
            } else {
                stmts.push(s);
            }
            continue;
        }

        // other statements: unparsed as generic Call or raw tokens
        let mut s = Statement::new(StatementKind::Call, Some(ln));
        s.func_name = Some(raw.clone());
        if let Some(ctx) = func_stack.last_mut() {
            ctx.body.push(s);
        } else {
            stmts.push(s);
        }
    }

    stmts
}
