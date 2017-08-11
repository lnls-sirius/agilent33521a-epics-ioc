extern crate agilent33521a_ioc_test;
extern crate termion;

use termion::color::{Fg, Green, Red, Yellow};
use termion::style::{Bold, Reset};

use agilent33521a_ioc_test::tests::run_tests;

fn main() {
    match run_tests() {
        Ok(test_results) => {
            let mut successful_tests = 0;
            let mut failed_tests = 0;

            for test_result in test_results {
                match *test_result.result() {
                    Ok(_) => successful_tests += 1,
                    Err(ref error) => {
                        println!(
                            "{bold}{red}Fail: {reset}{yellow}{name}{reset}: \
                             {message}",
                            bold = Bold,
                            name = test_result.name(),
                            message = error,
                            red = Fg(Red),
                            yellow = Fg(Yellow),
                            reset = Reset,
                        );
                        failed_tests += 1;
                    }
                }
            }

            if failed_tests > 0 {
                println!("");
            }

            if successful_tests == 1 {
                println!(
                    "{bold}1 test {green}succeeded{reset}",
                    bold = Bold,
                    green = Fg(Green),
                    reset = Reset
                );
            } else if successful_tests > 1 {
                println!(
                    "{bold}{count} tests {green}succeeded{reset}",
                    bold = Bold,
                    count = successful_tests,
                    green = Fg(Green),
                    reset = Reset
                );
            }

            if failed_tests == 1 {
                println!(
                    "{bold}1 test {red}failed{reset}",
                    bold = Bold,
                    red = Fg(Red),
                    reset = Reset
                );
            } else if failed_tests > 1 {
                println!(
                    "{bold}{count} tests {red}failed{reset}",
                    bold = Bold,
                    count = failed_tests,
                    red = Fg(Red),
                    reset = Reset
                );
            }
        }
        Err(error) => println!("Test execution failure: {}", error),
    }
}
