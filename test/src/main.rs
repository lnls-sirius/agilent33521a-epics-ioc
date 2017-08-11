extern crate agilent33521a_ioc_test;

use agilent33521a_ioc_test::tests::*;

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
                            " Test {} failed: {}",
                            test_result.name(),
                            error
                        );
                        failed_tests += 1;
                    }
                }
            }

            if successful_tests == 1 {
                println!("1 test succeeded");
            } else if successful_tests > 1 {
                println!("{} tests succeeded", successful_tests);
            }

            if failed_tests == 1 {
                println!("1 test failed");
            } else if failed_tests > 1 {
                println!("{} tests failed", failed_tests);
            }
        }
        Err(error) => println!("Test execution failure: {}", error),
    }
}
