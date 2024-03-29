use std::thread;
use std::option::Option::{ Some, None };
use std::sync::{mpsc, Arc};
use std::sync::mpsc::{Sender, Receiver};
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task;
use chashmap::{CHashMap};
use crate::prime_numbers_with_primes;
use crate::primes_fun::MathError;

#[derive(Debug, Clone, Eq)]
struct BlockResult {
    block_start : u32,
    result : Option<Vec<u32>>
}
impl PartialEq for BlockResult {
    fn eq(&self, other: &Self) -> bool {
        self.block_start == other.block_start
    }
}
impl Hash for BlockResult {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block_start.hash(state);
    }
}

/// Calculate prime numbers in blocks concurrently.
///  Shows usage of:
///  - creating a new threads
///  - using tokio task async function impl
///  - limiting task count by semaphore
///  - multi thread/task mpsc channel communication
///  - using custom impls for HashMap objects
///  - concurrent access to a central HashMap storage
pub async fn prime_numbers_with_tokio(start: u32, end: u32, block_size: usize) -> Result<Vec<u32>, MathError> {

    if start > end {
        return Err(MathError::EndBeforeStart);
    }

    // create reference counter for limiting worker task count to number of cpu cores
    let num_cpus = num_cpus::get();
    println!("Found {num_cpus} cpus - using this as max concurrent task count");
    let sem = Arc::new(Semaphore::new(num_cpus));

    // create central storage
    let db :Arc<CHashMap<u32, Option<Vec<u32>>>> = Arc::new(CHashMap::new());

    // create send/receiver for channel communication between worker tasks and storage thread
    let (tx, rx) :(Sender<BlockResult>, Receiver<BlockResult>) = mpsc::channel();

    //let block_storage = PrimesBlockStorage {..Default::default()};
    // create storage thread
    let db_handle = db.clone();
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(result) => {
                    match result.result {
                        Some(ref block) => println!("<DB received block {:?} with {} primes", result.block_start, block.len()),
                        None => println!("-DB received block {:?} for which calculation started", result.block_start)
                    }
                    // insert/update to storage in any case
                    db_handle.insert(result.block_start, result.result);
                },
                Err(_error) => {
                    // normal at the end -> ignore
                    //println!("received error {:?}", error);
                    break;
                }
            };
        }
    });

    // for primes, calculation has always start from 1... result filtering will be done in the end
    for block_start in (1..=end).step_by(block_size) {
        let tx_clone = mpsc::Sender::clone(&tx);
        let permit = sem.clone().acquire_owned().await.unwrap();
        println!("Acquired one permit, currently {} are still available", sem.as_ref().available_permits());
        let db_handle = db.clone();
        task::spawn(async move {
            // always calculate the whole block
            let block_end = block_start + (block_size as u32) - 1;
            println!("Calculating {}..{} primes block", block_start, block_end);
            // tell others I'm working on this block
            tx_clone.send(BlockResult{ block_start, result: None }).unwrap();

            // start calculation
            let mut result = Vec::with_capacity(1024);

            if block_start == 1 {

                // if I am the first, calculate block normally (should only happen once imho)
                result = prime_numbers_with_primes(block_size as u32);
                println!("  - calculation of first precalc block");

            } else {

                // I'm not the first and will use previous precalc_block results
                'next_prime: for prime_to_check in block_start..block_end {

                    let max_divider = (prime_to_check / 2) + 1;

                    // always start with the first precalc block and then work upwards
                    // check if I am this block => the first and last block...
                    let mut needed_precalc_block_start = 1;

                    // I'm not the first and will use previous precalc_blocks
                    'next_block: loop { // over all precalc_blocks

                        'read_block: loop {
                            let potential_readguard_precalc_block = db_handle.get(&needed_precalc_block_start);

                            if let Some(readguard_precalc_block) = potential_readguard_precalc_block {
                                if let Some(precalc_block) = readguard_precalc_block.as_ref() {
                                    // use precalc block content and always check if we're finished with this prime calc
                                    // iterate through precalc_block until end of it is reached
                                    for precalc_prime in precalc_block {
                                        // test prime_to_check against precalc_prime
                                        if *precalc_prime != 1 && prime_to_check % *precalc_prime == 0 {
                                            // prime_to_check is not a prime
                                            continue 'next_prime; // into next prime
                                        }
                                        // always check if prime calc is finished yet
                                        if *precalc_prime > max_divider {
                                            // current precalc_prime exceeds max_divider
                                            break 'read_block; // into next prime
                                        }
                                    }
                                    // end of precalc_block reading & usage - we're done here
                                    break 'read_block; // into next block
                                }
                            }
                            // precalc_block is not (yet) found or already in calculation - wait for it...
                            //println!("  - block {block_start} waiting for precalc block {needed_precalc_block_start}");
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        } // end of loop over block reading

                        if (needed_precalc_block_start + (block_size as u32)) > max_divider {
                            // next precalc_block_start would exceed max_divider
                            // println!("  - found prime {prime_to_check}");
                            result.push(prime_to_check);
                            continue 'next_prime;
                        }  else {
                            // if not finished, get next block for next round
                            needed_precalc_block_start = needed_precalc_block_start + (block_size as u32);
                            continue 'next_block;
                        }

                    } // end of loop over all precalced blocks

                } // end of loop over all primes to check

            } // end of block to check

            println!("Finished {}..{} primes block", block_start, block_end);
            drop(permit);
            let block_result = BlockResult {
                block_start,
                result: Some(result)
            };
            tx_clone.send(block_result).unwrap();
        });
    }

    // available permits should never reach the max again, wait until all are finished
    loop {
        let permit_count= sem.as_ref().available_permits();
        if permit_count < num_cpus {
            tokio::time::sleep(Duration::from_millis(100)).await;
            println!("Still waiting until all permits are available again, currently {permit_count} of {num_cpus} are available");
        } else { break };
    }

    // calculation is done, now we need to filter the results (start..end)
    let mut full_result = vec![];
    let first_block_start = start - ((start-1) % (block_size as u32));
    let last_block_start = end - ((end-1) % (block_size as u32));
    println!("Collecting blocks from {first_block_start} to {last_block_start} with step {block_size}");
    let db_handle = db.clone();
    for block_start in (first_block_start..=last_block_start).step_by(block_size) {
        let potential_readguard_prime_block = db_handle.get(&block_start);
        if let Some(readguard_prime_block) = potential_readguard_prime_block {
            if let Some(prime_block) = readguard_prime_block.as_ref() {
                println!("  - now working with precalc block {} with {} primes", block_start, (*prime_block).len());
                (*prime_block).iter()
                    .filter(|x|{ *x >= &start && *x <= &end })
                    .for_each(|x| { full_result.push(*x) });
            } else {
                panic!("block {} not finished after calc is finished?!? this should never happen!", block_start)
            }
        } else {
                panic!("block {} not found after calc is finished?!? this should never happen!", block_start)
        }
    }
    // return result
    Ok(full_result)

}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::{format_number, prime_numbers_with_primes_between};
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn test_primes_tokio() {
        println!("Testing prime_numbers with tokio");
        let result = prime_numbers_with_tokio(1, 100000, 20000).await;
        assert_eq!(result, Ok(prime_numbers_with_primes(100000)));

        let start_tokio = Instant::now();
        let result = prime_numbers_with_tokio(123456, 987654, 50000).await;
        let duration_tokio_millis = Instant::now().duration_since(start_tokio).as_millis();
        println!("... now calculating the same without tokio ...");
        let start_prime = Instant::now();
        assert_eq!(result, prime_numbers_with_primes_between(123456, 987654));
        let duration_prime_millis = Instant::now().duration_since(start_prime).as_millis();
        println!("<- Calculation of {} prime numbers took {}ms with tokio and {}ms without\n", format_number(result.unwrap().len() as u32), duration_tokio_millis, duration_prime_millis);
    }
}
