// core.rs

use std::collections::{HashMap, HashSet};
use std::process;

pub const APP_VERSION: &str = "v0.02 Ignition";
pub const LATEST_DATE: &str = "7/3/2025";

/// TCAS diagnostic record
#[derive(Debug)]
pub struct TCASError {
    pub code: &'static str,     // e.g. "TCAS101"
    pub level: &'static str,    // "error" or "warning"
    pub message: String,
    pub line: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatementKind {
    Let,
    ConsolePrint,
    Include,
    Func,
    Call,
    Input,
    Remove,
    Repeat,
    For,
    Foreach,
    While,
    Loop,
    If,
    Else,
}

#[derive(Clone, Debug)]
pub struct Statement {
    pub kind: StatementKind,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
    pub func_name: Option<String>,
    pub else_branch: Option<Box<Statement>>,
    pub line: Option<usize>,
}

impl Statement {
    /// Create a statement tagged with a source line
    pub fn new(kind: StatementKind, line: Option<usize>) -> Self {
        Statement {
            kind,
            args: Vec::new(),
            body: Vec::new(),
            func_name: None,
            else_branch: None,
            line,
        }
    }
}

/// Run TCAS checks in 10 passes, collect errors/warnings,
/// exit if any errors or warnings>threshold.
pub fn run_tcas(stmts: &[Statement], warning_threshold: usize) {
    let mut issues: Vec<TCASError> = Vec::new();

    for _ in 0..10 {
        // TCAS101: missing main()
        let has_main = stmts.iter().any(|s| {
            s.kind == StatementKind::Func
                && s.func_name.as_deref() == Some("main")
        });
        if !has_main {
            issues.push(TCASError {
                code: "TCAS101",
                level: "error",
                message: "Missing required func(main)".into(),
                line: None,
            });
        }

        // TCAS202: duplicate let at top level
        let mut seen = HashSet::new();
        for s in stmts.iter().filter(|s| s.kind == StatementKind::Let) {
            let var = &s.args[0];
            if !seen.insert(var.clone()) {
                issues.push(TCASError {
                    code: "TCAS202",
                    level: "warning",
                    message: format!("Duplicate global let({})", var),
                    line: s.line,
                });
            }
        }

        // add more checks here...
    }

    let errs = issues.iter().filter(|i| i.level == "error").count();
    let warns = issues.len() - errs;

    if errs > 0 || warns > warning_threshold {
        eprintln!("\n🚨 TCAS Preflight Check Failed:");
        for issue in &issues {
            match issue.line {
                Some(ln) => eprintln!(
                    "[{}] {} at line {} → {}",
                    issue.level.to_uppercase(),
                    issue.code,
                    ln,
                    issue.message
                ),
                None => eprintln!(
                    "[{}] {} → {}",
                    issue.level.to_uppercase(),
                    issue.code,
                    issue.message
                ),
            }
        }
        process::exit(1);
    }
}

/// Evaluate all statements, after TCAS validation
pub fn eval_all(stmts: &[Statement], warning_threshold: usize) {
    // 1) Preflight
    run_tcas(stmts, warning_threshold);

    // 2) Env & funcs
    let mut env: HashMap<String, String> = HashMap::new();
    let mut funcs: HashMap<String, Statement> = HashMap::new();

    // 3) Load global lets, prints, funcs
    for s in stmts {
        match s.kind {
            StatementKind::Let => {
                let val = eval_expr(&s.args[1], &env);
                env.insert(s.args[0].clone(), val);
            }
            StatementKind::ConsolePrint => {
                let out = s.args.iter()
                    .map(|a| eval_expr(a, &env))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("{}", out);
            }
            StatementKind::Func => {
                let name = s.func_name.clone().unwrap();
                funcs.insert(name, s.clone());
            }
            _ => {}
        }
    }

    // 4) Call main()
    if let Some(main_fn) = funcs.get("main") {
        eval_block(&main_fn.body, &mut env, &funcs);
    }
}

fn eval_block(
    stmts: &[Statement],
    env: &mut HashMap<String, String>,
    funcs: &HashMap<String, Statement>,
) {
    for s in stmts {
        match s.kind {
            StatementKind::Let => {
                let val = eval_expr(&s.args[1], env);
                env.insert(s.args[0].clone(), val);
            }
            StatementKind::ConsolePrint => {
                let out = s.args.iter()
                    .map(|a| eval_expr(a, env))
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("{}", out);
            }
            // … handle other kinds (Input, Remove, Call, Repeat, etc.) …
            _ => {}
        }
    }
}

/// Strict expr evaluator: only
/// - quoted string literals
/// - numeric literals
/// - defined variables
fn eval_expr(tok: &str, env: &HashMap<String, String>) -> String {
    if tok.starts_with('"') && tok.ends_with('"') {
        return tok[1..tok.len()-1].to_string();
    }
    if tok.parse::<f64>().is_ok() {
        return tok.to_string();
    }
    if let Some(v) = env.get(tok) {
        return v.clone();
    }
    eprintln!("🚫 ERROR: '{}' is not a string literal, number, or defined variable", tok);
    process::exit(1);
  }
