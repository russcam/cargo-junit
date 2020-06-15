extern crate clap;
extern crate duct;
extern crate nom;
extern crate sxd_document;
extern crate cargo_results;

use sxd_document::Package;
use sxd_document::writer::format_document;
use std::fs;
use std::str;
use std::path::Path;

extern crate time;

mod args;
mod cargo;
mod doc;

fn main() {
    let ref matches = args::get_args();
    let ref name = args::get_file_name(matches).unwrap();

    let suites = cargo::get_cargo_test_output(matches);
    let (totals, failures) = suites.suites.iter().fold((0, 0), |(total, failed), y| {
        (total + y.total, failed + y.failed)
    });

    if totals == 0 && !suites.unparsed.is_empty() {
        println!("{}", str::from_utf8(suites.unparsed.as_slice()).unwrap());
    }

    let package = Package::new();
    let d = package.as_document();

    let test_suites = doc::el(d, "testsuites")
        .attr("name", name)
        .attr("errors", failures)
        .attr("tests", totals)
        .attr("time", suites.time);

    doc::append_child(d, &test_suites);

    for suite in suites.suites {
        let test_suite = doc::el(d, "testsuite")
            .attr("name", suite.name)
            .attr("errors", suite.failed)
            .attr("failures", suite.failed)
            .attr("tests", suite.total)
            .append_to(&test_suites);

        for cargo_results::Test { name, error, .. } in suite.tests {
            let test_case = doc::el(d, "testcase")
                .attr("name", name)
                .append_to(&test_suite);

            if let Some(e) = error {
                doc::el(d, "failure")
                    .attr("message", e)
                    .append_to(&test_case);
            }
        }
    }

    let path = Path::new(name);
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).expect(&format!("could not create directory: {:?}", p));
    }

    let mut f =
        fs::File::create(path).expect(&format!("could not create file: {:?}", path));

    format_document(&d, &mut f)
        .ok()
        .expect(&format!("unable to output XML to {:?}", path));
}
