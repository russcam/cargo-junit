use duct::cmd;
use clap::ArgMatches;
use cargo_results::Suite;
use nom::IResult;
use std;
use cargo_results::cargo_test_result_parser;
use time::PreciseTime;

pub struct TestSuites {
    stdout: std::process::Output,
    pub suites: Vec<Suite>,
    pub time: i64,
}

pub fn get_cargo_test_output(matches: &ArgMatches) -> TestSuites {
    let sub_match = matches.subcommand_matches("junit").unwrap();

    let features = sub_match
        .value_of("features")
        .map(|x| format!(" --features {}", x))
        .unwrap_or("".to_string());

    let t = get_test_output(features);
    let output = match t.0 {
        Ok(a) => a,
	Err(e) => panic!("{}", e),
    };

    let mut ts = TestSuites {
        stdout: output,
        suites: vec!(),
        time: t.1,
    };

    ts.suites = match cargo_test_result_parser(&ts.stdout.stdout) {
        IResult::Done(_, x) => x,
        IResult::Error(e) => panic!("Parser error {:?}", e),
        _ => panic!("Parser did not finish successfully"),
    };

    ts
}

fn get_test_output(features: String) -> (std::io::Result<std::process::Output>, i64) {
    let args = vec![format!("test{}", features)];

    let start = PreciseTime::now();
    let output = cmd("cargo", args)
        .env("RUSTFLAGS", "-A warnings")
        .stderr_to_stdout()
        .stdout_capture()
        .unchecked()
        .run();

    let end = PreciseTime::now();
    return (output, start.to(end).num_seconds());
}
