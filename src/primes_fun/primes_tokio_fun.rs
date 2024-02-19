use std::thread;
use std::option::Option::{ Some, None };
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task;
use crate::prime_numbers_with_primes;

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

#[derive(Clone,Default)]
struct PrimesBlockStorage {
    map:Arc<RwLock<HashMap<u32, Option<Vec<u32>>>>>
}
impl PrimesBlockStorage {
    pub fn read_lock(&self) -> RwLockReadGuard<'_, HashMap<u32, Option<Vec<u32>>>> {
        self.map.read().unwrap()
    }
    pub fn write_lock(&self) -> RwLockWriteGuard<'_, HashMap<u32, Option<Vec<u32>>>> {
        self.map.write().unwrap()
    }
}

/// Calculate prime numbers in blocks concurrently.
///  Shows usage of:
///  - using tokio io task async function impl
///  - using tokio blocking tasks for cpu-bound work
///  - creating a new thread
///  - limiting task count by semaphore
///  - mpsc channel communication
///  - concurrent access to a central HashMap storage
///  - default init for structs
pub async fn prime_numbers_with_tokio(start: u32, end: u32, block_size: usize) -> Vec<u32> {

    // create reference counter for limiting worker task count to number of cpu cores
    let num_cpus = 1; // TODO num_cpus::get();
    println!("Found {num_cpus} cpus - using this as max concurrent task count");
    let sem = Arc::new(Semaphore::new(num_cpus));

    // create send/receiver for channel communication between worker tasks and storage thread
    let (tx, rx) : (Sender<BlockResult>, Receiver<BlockResult>)= mpsc::channel();

    let block_storage = PrimesBlockStorage {..Default::default()};
    // create storage thread
    let storage = block_storage.clone();
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(result) => {
                    match result.result {
                        Some(ref block) => println!("< received block {:?} with {} primes", result.block_start, block.len()),
                        None => println!("- received block {:?} for which calculation started", result.block_start)
                    }
                    // insert/update to storage in any case
                    storage.clone().write_lock().insert(result.block_start, result.result);
                },
                Err(error) => {
                    println!("received error {:?}", error);
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
        let storage = block_storage.clone();
        task::spawn_blocking(move || {
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
                for prime_to_check in block_start..block_end {

                    let mut prime_found: bool = true;
                    let max_divider = (prime_to_check / 2) + 1;

                    // always start with the first precalc block and then work upwards
                    // check if I am this block => the first and last block...
                    let mut needed_precalc_block_start = 1;

                    // I'm not the first and will use previous precalc_blocks
                    loop { // over all precalc_blocks

                        // get new block, wait for it if necessary
                        loop {
                            match storage.read_lock().get(&needed_precalc_block_start) {
                                None | Some(None) => {
                                    // precalc_block not found or in calculation
                                    println!("  - waiting for precalc block {needed_precalc_block_start}");
                                    thread::sleep(Duration::from_millis(10));
                                    continue; // continue waiting for this precalc_block
                                },
                                Some(Some(ref precalc_block_result)) => {
                                    //println!("  - now working with precalc block {needed_precalc_block_start}");

                                    // use precalc block content and always check if we're finished with this prime calc
                                    // iterate through precalc_block until end of it is reached
                                    for precalc_prime in (*precalc_block_result).clone() {
                                        // test prime_to_check against precalc_prime
                                        if precalc_prime != 1 && prime_to_check % precalc_prime == 0 {
                                            // prime_to_check is not a prime
                                            prime_found = false;
                                            break;
                                        }
                                        // always check if prime calc is finished yet
                                        if precalc_prime > max_divider {
                                            break;
                                        }
                                    }
                                    // end of precalc_block reading & usage - we're done here
                                    break; // into next block reading
                                }
                            }
                        } // end of loop over block reading

                        if !prime_found || (needed_precalc_block_start + (block_size as u32)) > max_divider {
                            break; // either no prime or finished with checking this prime
                        }  else {
                            // if not finished, get next block for next round
                            needed_precalc_block_start = needed_precalc_block_start + (block_size as u32);
                            continue; // with next block
                        }

                    } // end of loop over all precalced blocks

                    if prime_found {
                        //println!("  - found prime {prime_to_check}");
                        result.push(prime_to_check);
                    }

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
    let storage = block_storage.clone();
    for block_start in (first_block_start..=last_block_start).step_by(block_size) {
        match storage.read_lock().get(&block_start) {
            None | Some(None) => {
                // precalc_block not found or in calculation
                panic!("block {} not found after calc is finished?!? this should never happen!", block_start)
            },
            Some(Some(ref precalc_block_result)) => {
                println!("  - now working with precalc block {} with {} primes", block_start, (*precalc_block_result).len());
                (*precalc_block_result).iter()
                    .filter(|x|{ *x >= &start && *x <= &end })
                    .for_each(|x| { full_result.push(*x) });
            }
        }
    }
    full_result
}

#[cfg(test)]
mod tests {
    use crate::prime_numbers_with_primes_between;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[tokio::test]
    async fn test_primes_redis() {
        println!("Testing prime_numbers with tokio");
        let result = prime_numbers_with_tokio(1, 10000, 1000).await;
        assert_eq!(result, prime_numbers_with_primes(10000));
        let result = prime_numbers_with_tokio(123, 9876, 1000).await;
        assert_eq!(result, prime_numbers_with_primes_between(123, 9876).unwrap());
    }
}
