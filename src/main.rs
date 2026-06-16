//! Command-line interface for Frag.
//!
//! The binary keeps IO and argument handling here while delegating compiler
//! behavior to the `frag_compiler` library crate.

#![forbid(unsafe_code)]

use frag_compiler::diagnostic::{Diagnostic, Result};
use frag_compiler::lexer::{lex, TokenKind};
use frag_compiler::simulator::{SimOptions, SimulationResult};
use frag_compiler::{compile, graph, parser, simulator, verilog};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    if let Err(error) = run_cli() {
        eprintln!("{}", error);
        process::exit(1);
    }
}

fn run_cli() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_usage();
        return Ok(());
    }

    let commands = ["tokens", "ast", "ir", "verilog", "run", "graph"];
    let (command, file, rest) = if commands.contains(&args[0].as_str()) {
        if args.len() < 2 {
            return Err(Diagnostic::new(format!(
                "Missing file path for `{}` command",
                args[0]
            )));
        }
        (args[0].as_str(), args[1].as_str(), &args[2..])
    } else {
        ("verilog", args[0].as_str(), &args[1..])
    };

    match command {
        "tokens" => command_tokens(file),
        "ast" => command_ast(file),
        "ir" => command_ir(file),
        "verilog" => command_verilog(file, rest),
        "run" => command_run(file, rest),
        "graph" => command_graph(file, rest),
        _ => unreachable!(),
    }
}

fn command_tokens(file: &str) -> Result<()> {
    let source = read_source(file)?;
    let tokens = lex(&source).map_err(|error| with_file(error, file, &source))?;
    for token in tokens {
        if matches!(token.kind, TokenKind::Eof) {
            continue;
        }
        println!(
            "{} @ {}..{}",
            token_label(&token.kind),
            token.span.start,
            token.span.end
        );
    }
    Ok(())
}

fn command_ast(file: &str) -> Result<()> {
    let source = read_source(file)?;
    let ast = parser::parse_source(&source).map_err(|error| with_file(error, file, &source))?;
    println!("{:#?}", ast);
    Ok(())
}

fn command_ir(file: &str) -> Result<()> {
    let source = read_source(file)?;
    let output = compile(&source).map_err(|error| with_file(error, file, &source))?;
    println!("{}", output.ir);
    Ok(())
}

fn command_verilog(file: &str, args: &[String]) -> Result<()> {
    let source = read_source(file)?;
    let output = compile(&source).map_err(|error| with_file(error, file, &source))?;
    let verilog = verilog::emit(&output.ir);
    if let Some(path) = output_path(args)? {
        fs::write(&path, verilog).map_err(|error| {
            Diagnostic::new(format!("Failed to write `{}`: {}", path.display(), error))
        })?;
    } else {
        print!("{}", verilog);
    }
    Ok(())
}

fn command_run(file: &str, args: &[String]) -> Result<()> {
    let source = read_source(file)?;
    let output = compile(&source).map_err(|error| with_file(error, file, &source))?;
    let (options, vcd_path) = run_options(args)?;
    let result = simulator::run(&output.ir, &options)?;
    print!("{}", result);

    if let Some(path) = vcd_path {
        match &result {
            SimulationResult::Waveform(waveform) => {
                fs::write(&path, simulator::to_vcd(waveform)).map_err(|error| {
                    Diagnostic::new(format!("Failed to write `{}`: {}", path.display(), error))
                })?;
            }
            SimulationResult::TruthTable(_) => {
                return Err(Diagnostic::new(
                    "VCD output is available for sequential simulations",
                ));
            }
        }
    }

    Ok(())
}

fn command_graph(file: &str, args: &[String]) -> Result<()> {
    let source = read_source(file)?;
    let output = compile(&source).map_err(|error| with_file(error, file, &source))?;
    let (format, path) = graph_options(args)?;
    let text = match format.as_str() {
        "dot" => graph::emit_dot(&output.ir),
        "mermaid" => graph::emit_mermaid(&output.ir),
        _ => {
            return Err(Diagnostic::new(format!(
                "Unknown graph format `{}`; expected `dot` or `mermaid`",
                format
            )));
        }
    };

    if let Some(path) = path {
        fs::write(&path, text).map_err(|error| {
            Diagnostic::new(format!("Failed to write `{}`: {}", path.display(), error))
        })?;
    } else {
        print!("{}", text);
    }
    Ok(())
}

fn output_path(args: &[String]) -> Result<Option<std::path::PathBuf>> {
    let mut idx = 0;
    let mut path = None;
    while idx < args.len() {
        match args[idx].as_str() {
            "-o" | "--output" => {
                idx += 1;
                let Some(value) = args.get(idx) else {
                    return Err(Diagnostic::new("Missing path after output option"));
                };
                path = Some(Path::new(value).to_path_buf());
            }
            other => return Err(Diagnostic::new(format!("Unknown option `{}`", other))),
        }
        idx += 1;
    }
    Ok(path)
}

fn run_options(args: &[String]) -> Result<(SimOptions, Option<std::path::PathBuf>)> {
    let mut options = SimOptions::default();
    let mut vcd_path = None;
    let mut idx = 0;

    while idx < args.len() {
        match args[idx].as_str() {
            "--ticks" => {
                idx += 1;
                let Some(value) = args.get(idx) else {
                    return Err(Diagnostic::new("Missing number after `--ticks`"));
                };
                options.ticks = value
                    .parse::<usize>()
                    .map_err(|_| Diagnostic::new(format!("Invalid tick count `{}`", value)))?;
            }
            "--set" => {
                idx += 1;
                let Some(value) = args.get(idx) else {
                    return Err(Diagnostic::new("Missing assignments after `--set`"));
                };
                parse_input_overrides(value, &mut options.inputs)?;
            }
            "--vcd" => {
                idx += 1;
                let Some(value) = args.get(idx) else {
                    return Err(Diagnostic::new("Missing path after `--vcd`"));
                };
                vcd_path = Some(Path::new(value).to_path_buf());
            }
            other => return Err(Diagnostic::new(format!("Unknown option `{}`", other))),
        }
        idx += 1;
    }

    Ok((options, vcd_path))
}

fn graph_options(args: &[String]) -> Result<(String, Option<std::path::PathBuf>)> {
    let mut format = "dot".to_string();
    let mut path = None;
    let mut idx = 0;

    while idx < args.len() {
        match args[idx].as_str() {
            "--format" => {
                idx += 1;
                let Some(value) = args.get(idx) else {
                    return Err(Diagnostic::new("Missing value after `--format`"));
                };
                format = value.clone();
            }
            "-o" | "--output" => {
                idx += 1;
                let Some(value) = args.get(idx) else {
                    return Err(Diagnostic::new("Missing path after output option"));
                };
                path = Some(Path::new(value).to_path_buf());
            }
            other => return Err(Diagnostic::new(format!("Unknown option `{}`", other))),
        }
        idx += 1;
    }

    Ok((format, path))
}

fn parse_input_overrides(text: &str, inputs: &mut BTreeMap<String, u128>) -> Result<()> {
    for part in text.split(',').filter(|part| !part.trim().is_empty()) {
        let Some((name, value)) = part.split_once('=') else {
            return Err(Diagnostic::new(format!(
                "Invalid input override `{}`; expected name=value",
                part
            )));
        };
        inputs.insert(name.trim().to_string(), parse_int(value.trim())?);
    }
    Ok(())
}

fn parse_int(text: &str) -> Result<u128> {
    if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        u128::from_str_radix(rest, 16)
    } else if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        u128::from_str_radix(rest, 2)
    } else {
        text.parse::<u128>()
    }
    .map_err(|_| Diagnostic::new(format!("Invalid integer `{}`", text)))
}

fn read_source(file: &str) -> Result<String> {
    fs::read_to_string(file)
        .map_err(|error| Diagnostic::new(format!("Failed to read `{}`: {}", file, error)))
}

fn with_file(error: Diagnostic, file: &str, source: &str) -> Diagnostic {
    Diagnostic::new(format!("{}:\n{}", file, error.with_source(source)))
}

fn token_label(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Identifier(name) => format!("Identifier({})", name),
        TokenKind::Number(value) => format!("Number({})", value),
        TokenKind::BoolLiteral(value) => format!("Bool({})", value),
        TokenKind::Module => "Module".to_string(),
        TokenKind::Input => "Input".to_string(),
        TokenKind::Output => "Output".to_string(),
        TokenKind::Wire => "Wire".to_string(),
        TokenKind::Reg => "Reg".to_string(),
        TokenKind::Const => "Const".to_string(),
        TokenKind::On => "On".to_string(),
        TokenKind::Rising => "Rising".to_string(),
        TokenKind::Falling => "Falling".to_string(),
        TokenKind::If => "If".to_string(),
        TokenKind::Else => "Else".to_string(),
        TokenKind::Bit => "Bit".to_string(),
        TokenKind::BoolType => "BoolType".to_string(),
        TokenKind::Colon => "Colon".to_string(),
        TokenKind::Semicolon => "Semicolon".to_string(),
        TokenKind::Comma => "Comma".to_string(),
        TokenKind::LeftBrace => "LeftBrace".to_string(),
        TokenKind::RightBrace => "RightBrace".to_string(),
        TokenKind::LeftParen => "LeftParen".to_string(),
        TokenKind::RightParen => "RightParen".to_string(),
        TokenKind::Equal => "Equal".to_string(),
        TokenKind::Plus => "Plus".to_string(),
        TokenKind::Minus => "Minus".to_string(),
        TokenKind::Star => "Star".to_string(),
        TokenKind::Slash => "Slash".to_string(),
        TokenKind::Percent => "Percent".to_string(),
        TokenKind::Amp => "And".to_string(),
        TokenKind::Pipe => "Or".to_string(),
        TokenKind::Caret => "Xor".to_string(),
        TokenKind::Tilde => "Tilde".to_string(),
        TokenKind::Bang => "Not".to_string(),
        TokenKind::AmpAmp => "LogicAnd".to_string(),
        TokenKind::PipePipe => "LogicOr".to_string(),
        TokenKind::EqualEqual => "EqualEqual".to_string(),
        TokenKind::BangEqual => "NotEqual".to_string(),
        TokenKind::Less => "Less".to_string(),
        TokenKind::LessEqual => "LessEqual".to_string(),
        TokenKind::Greater => "Greater".to_string(),
        TokenKind::GreaterEqual => "GreaterEqual".to_string(),
        TokenKind::ShiftLeft => "ShiftLeft".to_string(),
        TokenKind::ShiftRight => "ShiftRight".to_string(),
        TokenKind::Eof => "Eof".to_string(),
    }
}

fn print_usage() {
    eprintln!(
        "Usage:
  frag <file.frag>                  Generate Verilog
  frag tokens <file.frag>           Print tokens
  frag ast <file.frag>              Print AST
  frag ir <file.frag>               Print netlist IR
  frag verilog <file.frag> [-o out] Generate Verilog
  frag run <file.frag> [--ticks N] [--set a=1,b=0] [--vcd out.vcd]
  frag graph <file.frag> [--format dot|mermaid] [-o out]"
    );
}
