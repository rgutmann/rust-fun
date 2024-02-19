
use super::*;
//use mini_redis::blocking_client;

/// Generating prime numbers with the simple optimization that only dividers until number / 2 need to be checked.
pub fn prime_numbers_redis(_end: u32) -> Result<Vec<u32>, MathError> {
    Err(MathError::NotImplemented)
}

#[cfg(test)]
mod tests {
    use super::*;
    use MathError;

    #[test]
    fn test_primes_redis() {
        println!("Testing prime_numbers with redis storage");
        //assert_eq!(prime_numbers_redis(20), vec![1, 2, 3, 5, 7, 11, 13, 17, 19]);
        assert_eq!(prime_numbers_redis(20), Result::Err(MathError::NotImplemented));
    }

}
