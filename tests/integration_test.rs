use std::path::PathBuf;
use std::{env, fs, process::Command};

use regex::Regex;

fn lox_command() -> Command {
    // Create full path to binary
    let mut path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    path.push(env!("CARGO_PKG_NAME"));
    path.set_extension(env::consts::EXE_EXTENSION);
    // println!("{}", path.display());
    Command::new(path.into_os_string())
}

struct RuntimeError {
    line_prefix: String,
    message: String,
}

struct Expected {
    out: Vec<String>,
    compile_err: Vec<String>,
    runtime_err: Option<RuntimeError>,
}

fn parse_comments(path: &PathBuf) -> Expected {
    let output_re = Regex::new(r"// expect: ?(.*)").unwrap();
    let error_re = Regex::new(r"// (Error.*)").unwrap();
    let error_line_re = Regex::new(r"// \[(?:c )?line (\d+)\] (Error.*)").unwrap();
    let runtime_error_re = Regex::new(r"// expect runtime error: (.+)").unwrap();

    let mut expected = Expected {
        out: vec![],
        compile_err: vec![],
        runtime_err: None,
    };

    let content = fs::read_to_string(path).unwrap();
    for (i, line) in content.lines().enumerate() {
        if let Some(m) = output_re.captures(line) {
            let s = m.get(1).unwrap().as_str().to_owned();
            expected.out.push(s);
        }
        if let Some(m) = error_line_re.captures(line) {
            let line = m.get(1).unwrap().as_str();
            let msg = m.get(2).unwrap().as_str();
            let s = format!("[line {}] {}", line, msg);
            expected.compile_err.push(s);
        }
        if let Some(m) = error_re.captures(line) {
            let msg = m.get(1).unwrap().as_str();
            let s = format!("[line {}] {}", i + 1, msg);
            expected.compile_err.push(s);
        }
        if let Some(m) = runtime_error_re.captures(line) {
            let message = m.get(1).unwrap().as_str().to_owned();
            let line_prefix = format!("[line {}]", i + 1);
            expected.runtime_err = Some(RuntimeError {
                line_prefix,
                message,
            });
        }
    }
    expected
}

fn path(path: &str) -> String {
    format!("{}/tests/cases/{}", env!("CARGO_MANIFEST_DIR"), path)
}

fn get_test_cases(case_dir: &str) -> Option<Vec<String>> {
    let mut res = Vec::new();

    let path = PathBuf::from(path(case_dir));
    for d in fs::read_dir(path).ok()? {
        let d = d.ok()?;
        if d.metadata().ok()?.is_file() {
            if let Ok(name) = d.file_name().into_string() {
                res.push(format!("{}/{}", case_dir, name));
            }
        }
    }

    res.push("empty_file.lox".to_string());
    res.push("precedence.lox".to_string());

    Some(res)
}

fn run(case_dir: &str) {
    let cases = get_test_cases(case_dir).unwrap_or(Vec::new());
    if cases.len() == 0 {
        println!("No test cases in {}", case_dir);
        return;
    }

    println!("Testing: {}", case_dir);
    for case in cases {
        println!("Case: {}", case);

        let pb = PathBuf::from(path(&case));
        let expected = parse_comments(&pb);
        let output = lox_command().arg(pb).output().unwrap();

        let stdout: Vec<String> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|x| x.to_owned())
            .collect();
        let err_out: Vec<String> = String::from_utf8(output.stderr)
            .unwrap()
            .lines()
            .map(|x| x.to_owned())
            .collect();

        /*
        println!("stdout:");
        for o in &stdout {
            println!("{}", o);
        }
        println!("err out: ");
        for e in &err_out {
            println!("{}", e);
        }
        */

        if let Some(e) = expected.runtime_err {
            assert_eq!(e.message, err_out[0], "Runtime error should match");
            assert_eq!(
                err_out[1][0..e.line_prefix.len()],
                e.line_prefix,
                "Runtime error line should match"
            );
        } else {
            assert_eq!(expected.compile_err, err_out, "Compile error should match");
        }

        assert_eq!(expected.out, stdout, "Output should match");

        println!("success");
    }
}

#[test]
fn assignment_test() {
    run("assignment");
}

#[test]
fn block_test() {
    run("block");
}

#[test]
fn bool_test() {
    run("bool");
}

#[test]
fn comment_test() {
    run("comments");
}

#[test]
fn expression_test() {
    run("expressions");
}

#[test]
fn logical_operator_test() {
    run("logical_operator");
}

#[test]
fn nil_test() {
    run("nil");
}

#[test]
fn operator_test() {
    run("operator");
}

#[test]
fn print_test() {
    run("block");
}

#[test]
fn scanning_test() {
    run("scanning");
}

#[test]
fn string_test() {
    run("string");
}

#[test]
fn variable_test() {
    run("variable");
}

#[test]
fn function_test() {
    run("function");
}
