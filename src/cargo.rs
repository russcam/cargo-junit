use duct::cmd;
use clap::ArgMatches;
use cargo_results::Suite;
use nom::IResult;
use std;
use std::str;
use cargo_results::cargo_test_result_parser;
use time::PreciseTime;

pub struct TestSuites {
    stdout: std::process::Output,
    pub suites: Vec<Suite>,
    pub unparsed: Vec<u8>,
    pub time: i64,
}

pub fn get_cargo_test_output(matches: &ArgMatches) -> TestSuites {
    let sub_match = matches.subcommand_matches("junit").unwrap();

    let mut test_args = vec!["test".to_string()];

    let test_name = sub_match
        .value_of("testname")
        .map(|x| x.to_string())
        .unwrap_or("".to_string());

    if !test_name.is_empty() {
        test_args.push(test_name);
    }

    let features = sub_match
        .value_of("features")
        .map(|x| x)
        .unwrap_or("");

    if !features.is_empty() {
        test_args.push("--features".to_string());
        test_args.push(features.to_string());
    }

    let test_threads = sub_match
        .value_of("test-threads")
        .map(|x| x)
        .unwrap_or("");

    if !test_threads.is_empty() {
        test_args.push("--".to_string());
        test_args.push(format!("--test-threads={}", test_threads));
    }

    let t = get_test_output(test_args);
    let output = match t.0 {
        Ok(a) => a,
	    Err(e) => panic!("{}", e),
    };

    let mut ts = TestSuites {
        stdout: output,
        suites: vec![],
        unparsed: vec![],
        time: t.1,
    };

    match cargo_test_result_parser(&ts.stdout.stdout) {
        IResult::Done(y, x) => {
            ts.unparsed = y.to_vec();
            ts.suites = x;
        },
        IResult::Error(e) => panic!("Parser error {:?}\n\n{}", e, str::from_utf8(&ts.stdout.stdout).unwrap()),
        _ => panic!("Parser did not finish successfully"),
    };

    ts
}

fn get_test_output(test_args: Vec<String>) -> (std::io::Result<std::process::Output>, i64) {
    let start = PreciseTime::now();
    let output = cmd("cargo", test_args)
        .env("RUSTFLAGS", "-A warnings")
        .stderr_to_stdout()
        .stdout_capture()
        .unchecked()
        .run();
    let end = PreciseTime::now();
    return (output, start.to(end).num_seconds());
}
