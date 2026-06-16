use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn generated_verilog_is_accepted_by_external_tools() {
    let required = env::var_os("FRAG_REQUIRE_EXTERNAL_TOOLS").is_some();
    let missing = ["iverilog", "verilator", "dot"]
        .into_iter()
        .filter(|tool| !tool_exists(tool))
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        if required {
            panic!("missing required external tools: {}", missing.join(", "));
        }
        eprintln!(
            "skipping external HDL toolchain test; missing: {}",
            missing.join(", ")
        );
        return;
    }

    let temp = fresh_temp_dir("frag-toolchain");
    let fresh_frag = temp.join("fresh_probe.frag");
    let fresh_verilog = temp.join("fresh_probe.v");
    let fresh_dot = temp.join("fresh_probe.dot");
    let fresh_svg = temp.join("fresh_probe.svg");
    let counter_vcd = temp.join("counter.vcd");

    fs::write(&fresh_frag, fresh_probe_source()).expect("write fresh source");

    run_checked(
        "frag check fresh source",
        Command::new(frag_bin()).arg("check").arg(&fresh_frag),
    );
    run_checked(
        "frag verilog fresh source",
        Command::new(frag_bin())
            .arg("verilog")
            .arg(&fresh_frag)
            .arg("-o")
            .arg(&fresh_verilog),
    );
    run_checked(
        "iverilog fresh generated Verilog",
        tool_command("iverilog")
            .arg("-g2012")
            .arg("-tnull")
            .arg(&fresh_verilog),
    );
    run_checked(
        "verilator fresh generated Verilog",
        tool_command("verilator")
            .arg("--lint-only")
            .arg("-Wno-DECLFILENAME")
            .arg(&fresh_verilog),
    );
    run_checked(
        "frag graph fresh source",
        Command::new(frag_bin())
            .arg("graph")
            .arg(&fresh_frag)
            .arg("--format")
            .arg("dot")
            .arg("-o")
            .arg(&fresh_dot),
    );
    run_checked(
        "Graphviz fresh DOT",
        tool_command("dot")
            .arg("-Tsvg")
            .arg(&fresh_dot)
            .arg("-o")
            .arg(&fresh_svg),
    );

    assert!(
        fs::metadata(&fresh_svg).expect("svg exists").len() > 0,
        "Graphviz should create a non-empty SVG"
    );

    run_checked(
        "frag VCD generation",
        Command::new(frag_bin())
            .arg("run")
            .arg("examples/counter.frag")
            .arg("--ticks")
            .arg("8")
            .arg("--vcd")
            .arg(&counter_vcd),
    );
    let vcd = fs::read_to_string(counter_vcd).expect("vcd is readable");
    assert!(vcd.contains("$enddefinitions $end"));
    assert!(vcd.contains("count"));

    for example in fs::read_dir("examples").expect("examples directory exists") {
        let path = example.expect("directory entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("frag") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|name| name.to_str())
            .expect("example has a stem");
        let verilog = temp.join(format!("{stem}.v"));

        run_checked(
            &format!("frag verilog {}", path.display()),
            Command::new(frag_bin())
                .arg("verilog")
                .arg(&path)
                .arg("-o")
                .arg(&verilog),
        );
        run_checked(
            &format!("iverilog {}", verilog.display()),
            tool_command("iverilog")
                .arg("-g2012")
                .arg("-tnull")
                .arg(&verilog),
        );
        run_checked(
            &format!("verilator {}", verilog.display()),
            tool_command("verilator")
                .arg("--lint-only")
                .arg("-Wno-DECLFILENAME")
                .arg(&verilog),
        );
    }
}

fn fresh_probe_source() -> &'static str {
    r#"
module FreshProbe123 {
    input alpha: u4;
    input beta: u4;
    input flag: bit;
    input clk: bit;

    output mixed: u4;
    output active: bit;

    wire masked: u4;
    reg sample: u4;

    const offset: u4 = 2;
    const mask: u4 = offset + 1;

    masked = (alpha ^ beta) & mask;
    mixed = masked + sample;
    active = (mixed != 0) && flag;

    on rising(clk) {
        sample = alpha + offset;
    }
}
"#
}

fn frag_bin() -> &'static str {
    env!("CARGO_BIN_EXE_frag")
}

fn fresh_temp_dir(prefix: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("test-artifacts");
    path.push(format!("{}-{}", prefix, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn tool_exists(tool: &str) -> bool {
    let mut command = if cfg!(windows) {
        let mut command = Command::new("where.exe");
        command.arg(tool_program(tool));
        command
    } else {
        let mut command = Command::new("which");
        command.arg(tool);
        command
    };
    command.env("PATH", tool_path());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    command
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn tool_command(tool: &str) -> Command {
    let mut command = Command::new(tool_program(tool));
    command.env("PATH", tool_path());
    if cfg!(windows) {
        command.env("VERILATOR_ROOT", r"C:\msys64\mingw64\share\verilator");
    }
    command
}

fn tool_program(tool: &str) -> &str {
    if cfg!(windows) && tool == "verilator" {
        "verilator.cmd"
    } else {
        tool
    }
}

fn tool_path() -> OsString {
    let mut paths = Vec::new();
    if cfg!(windows) {
        paths.push(PathBuf::from(r"C:\msys64\mingw64\bin"));
        paths.push(PathBuf::from(r"C:\msys64\usr\bin"));
    }
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
    env::join_paths(paths).expect("valid PATH")
}

fn run_checked(label: &str, command: &mut Command) {
    let output = command.output().unwrap_or_else(|error| {
        panic!("failed to start {label}: {error}");
    });

    if !output.status.success() {
        panic!(
            "{label} failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
