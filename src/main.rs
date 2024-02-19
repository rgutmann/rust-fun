extern crate chrono;

mod math_fun;
use math_fun::*;

use chrono::Local;
use std::time::Instant;

fn main() {
    println!("\n{:?}\nPrime numbers 1..10000 calculation start...", Local::now());
    let start_prime = Instant::now();
    let primes_result = prime_numbers(1,10000);
    let duration_prime_millis = Instant::now().duration_since(start_prime).as_millis();
    println!("Prime numbers found: {:?}", primes_result);
    println!("<- Calculation took {}ms\n", duration_prime_millis);

    println!("\n{:?}\nPrime numbers with primes 1..10000 calculation start...", Local::now());
    let start_prime = Instant::now();
    let primes_result = prime_numbers_with_primes(10000);
    let duration_prime_millis = Instant::now().duration_since(start_prime).as_millis();
    println!("Prime numbers found: {:?}", primes_result);
    println!("<- Calculation took {}ms\n", duration_prime_millis);

}
