extern crate clap;
extern crate duct;
extern crate nom;
extern crate sxd_document;
extern crate cargo_results;

use sxd_document::Package;
use sxd_document::writer::format_document;
use std::fs;

mod doc;
mod args;
mod cargo;

fn main() {
    let ref matches = args::get_args();
    let ref name = args::get_file_name(matches).unwrap();
    let suites = cargo::get_cargo_test_output(matches);
    let (totals, failures) = suites.suites.iter().fold((0, 0), |(total, failed), y| {
        (total + y.total, failed + y.failed)
    });

    let package = Package::new();
    let d = package.as_document();

    let test_suites = doc::el(d, "testsuites")
        .attr("name", name)
        .attr("errors", failures)
        .attr("tests", totals);

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

    let mut f =
        fs::File::create(format!("{}", name)).expect(&format!("could not create file: {}", name));

    format_document(&d, &mut f)
        .ok()
        .expect(&format!("unable to output XML to {}", name));
}
