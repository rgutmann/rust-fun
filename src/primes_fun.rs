#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathError {
    EndBeforeStart,
}

/// Generating prime numbers with the simple optimization that only dividers until number / 2 need to be checked.
pub fn prime_numbers_between(start: u32, end: u32) -> Result<Vec<u32>, MathError> {
    if start > end {
        return Err(MathError::EndBeforeStart);
    }

    let mut result = Vec::with_capacity(1024);
    for i in start..(end + 1) {
        let mut found: bool = true;

        for j in 2..(i / 2 + 1) {
            if (i % j) == 0 {
                found = false;
                break;
            }
        }

        if found {
            result.push(i);
        };
    }

    Ok(result)
}

/// Generating prime numbers with the simple optimization that only dividers until number / 2 need to be checked.
pub fn prime_numbers(end: u32) -> Vec<u32> {
    prime_numbers_between(1, end).expect("prime numbers returned")
}

/// Generating prime numbers with the optimization that only prime dividers need to be checked.
pub fn prime_numbers_with_primes(end: u32) -> Vec<u32> {
    let mut result = Vec::with_capacity(1024);
    for i in 1..(end + 1) {
        let mut found: bool = true;

        for j in &result {
            if *j == 1 {
                continue;
            } else if *j < (i / 2 + 1) {
                if (i % *j) == 0 {
                    found = false;
                    break;
                }
            } else { break; }
        }

        if found {
            result.push(i);
        };
    }

    result
}

///  Generating prime numbers with the optimization that only prime dividers need to be checked.
pub fn prime_numbers_with_primes_between(start: u32, end: u32) -> Result<Vec<u32>, MathError> {
    if start > end {
        return Err(MathError::EndBeforeStart);
    }

    let result = prime_numbers_with_primes(end);
    let mut index = 0;
    for (i, x) in result.iter().enumerate() {
        if *x >= start {
            index = i;
            break;
        }
    }

    Ok(result[index..].to_vec())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_primes() {
        println!("Testing prime_numbers");
        assert_eq!(prime_numbers(20), vec![1, 2, 3, 5, 7, 11, 13, 17, 19]);
    }

    #[test]
    fn test_primes_between() {
        println!("Testing prime_numbers_between");
        assert_eq!(prime_numbers_between(20, 1).err().expect("error returned"), MathError::EndBeforeStart);
        assert_eq!(prime_numbers_between(1, 20), Ok(vec![1, 2, 3, 5, 7, 11, 13, 17, 19]));
        assert_eq!(prime_numbers_between(4, 20), Ok(vec![5, 7, 11, 13, 17, 19]));
    }

    #[test]
    fn test_primes_with_primes() {
        println!("Testing prime_numbers_with_primes");
        assert_eq!(prime_numbers_with_primes(1000), prime_numbers_between(1, 1000).expect("no primes returned"));
    }

    #[test]
    fn test_primes_with_primes_between() {
        println!("Testing prime_numbers_with_primes_between");
        assert_eq!(prime_numbers_with_primes_between(20, 1).err().expect("no error returned"), MathError::EndBeforeStart);
        assert_eq!(prime_numbers_with_primes_between(1, 100).expect("no primes returned"), prime_numbers_between(1, 100).expect("no primes returned"));
        assert_eq!(prime_numbers_with_primes_between(100, 1000).expect("no primes returned"), prime_numbers_between(100, 1000).expect("no primes returned"));
    }
}
