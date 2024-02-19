extern crate chrono;
extern crate core;

mod primes_fun;

use primes_fun::{prime_numbers, prime_numbers_between, prime_numbers_with_primes, prime_numbers_with_primes_between};

use chrono::Local;
use num_format::{Locale, ToFormattedString};
use std::time::Instant;

macro_rules! dbg {
    ($x:expr) => {
        println!("{} = {:?}",stringify!($x),$x);
    }
}

fn format_number(number: u32) -> String {
    number.to_formatted_string(&Locale::en)
}
/*
impl U32 for u32 {
    fn display(number: u32_d) -> String {
        number.to_formatted_string(&Locale::en)
    }
}
*/

#[tokio::main]
async fn main() {
    let debug = false;

    let end = 10000u32;
    println!("\n{:?}\nPrime numbers 1..{} calculation start...", Local::now(), format_number(end));
    let start_prime = Instant::now();
    let primes_result = prime_numbers(end);
    let duration_prime_millis = Instant::now().duration_since(start_prime).as_millis();
    if debug { println!("Prime numbers found: {:?}", primes_result); }
    println!("<- Calculation of {} prime numbers took {}ms\n", format_number(primes_result.len() as u32), duration_prime_millis);

    let end = 100000u32;
    println!("\n{:?}\nPrime numbers with primes 1..{} calculation start...", Local::now(), format_number(end));
    let start_prime = Instant::now();
    let primes_result = prime_numbers_with_primes(end);
    let duration_prime_millis = Instant::now().duration_since(start_prime).as_millis();
    if debug { println!("Prime numbers found: {:?}", primes_result); }
    println!("<- Calculation of {} prime numbers took {}ms\n", format_number(primes_result.len() as u32), duration_prime_millis);

    let start = 10000u32;
    let end = 20000u32;
    println!("\n{:?}\nPrime numbers {}..{} calculation start...", Local::now(), format_number(start), format_number(end));
    let start_prime = Instant::now();
    let primes_result = prime_numbers_between(start, end).unwrap();
    let duration_prime_millis = Instant::now().duration_since(start_prime).as_millis();
    if debug { println!("Prime numbers found: {:?}", primes_result); }
    println!("<- Calculation of {} prime numbers took {}ms\n", format_number(primes_result.len() as u32), duration_prime_millis);

    let start = 100000u32;
    let end = 200000u32;
    println!("\n{:?}\nPrime numbers with primes {}..{} calculation start...", Local::now(), format_number(start), format_number(end));
    let start_prime = Instant::now();
    let primes_result = prime_numbers_with_primes_between(start, end).unwrap();
    let duration_prime_millis = Instant::now().duration_since(start_prime).as_millis();
    if debug { println!("Prime numbers found: {:?}", primes_result); }
    println!("<- Calculation of {} prime numbers took {}ms\n", format_number(primes_result.len() as u32), duration_prime_millis);

}
