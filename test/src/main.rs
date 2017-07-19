extern crate agilent33521a_ioc_test;

use agilent33521a_ioc_test::tests::*;

fn main() {
    match test_enable_channel_output() {
        Ok(_) => println!("Test Successful"),
        Err(error) => println!("Test FAILED: {}", error),
    }
}
