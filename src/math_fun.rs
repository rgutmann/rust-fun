
pub fn prime_numbers(start: i32, end: i32) -> Vec<i32> {
    let mut result = Vec::with_capacity(1000);
    for i in start..end {
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

    return result;
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_primes() {
        println!("Testing prime_numbers");
        assert_eq!(prime_numbers(1, 20), vec![1, 2, 3, 5, 7, 11, 13, 17, 19]);
    }
}