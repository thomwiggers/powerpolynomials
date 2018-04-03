#![feature(i128)]
extern crate rand;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate lazy_static;

use rand::{thread_rng, Rng};
use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read,Write};
use std::sync::Mutex;

type ExponentMap = HashMap<u64, u128>;

lazy_static! {
    static ref BEST_RESULTS: Mutex<HashMap<u32, ExponentMap>> = Mutex::new(HashMap::new());
}

/// Find a_0 × 1^n + a_1 × 2^n + \ldots + a_i × i^n == target
/// with the smallest a_j
fn exponent_polynomial(mut target: u128, i: u64, n: u32) -> ExponentMap {
    let mut result = HashMap::new();
    let mut rng = thread_rng();
    let mut items = (1..i).map(|x| (x, (x as u128).pow(n))).collect::<Vec<(u64, u128)>>();
    rng.shuffle(&mut items);
    for (x, xpow) in items.into_iter() {
        let c = target / xpow;
        if c == 0 {
            //println!("{}**{} = {} is too large to fit in {}", x, n, xpow, target);
            continue;
        }
        //println!("{}^{}={} fits {} times in {}", x, n, xpow, c, target);
        result.insert(x, c);
        target %= xpow;
        if target == 0 {
            break;
        }
    }

    result
}

fn compute_result(result: &ExponentMap, n: u32) -> u128 {
    result.iter().map(|(x, c)| (*x as u128).pow(n) * (*c as u128)).sum::<u128>()
}

fn get_prior_result(i: u64, target: u128, n: u32) -> ExponentMap {
    let map = BEST_RESULTS.lock().unwrap();
    if !map.contains_key(&(i as u32)) {
        exponent_polynomial(target, i, n)
    } else {
        map.get(&(i as u32)).unwrap().clone()
    }
}

fn find_smallest_exponent_polynomial(tries: u32, target: u128, i: u64, n: u32) -> ExponentMap{
    let mut best = get_prior_result(i, target, n);
    debug_assert_eq!(compute_result(&best, n), target);
    for _ in 0..tries {
        let new_attempt = exponent_polynomial(target, i, n);
        debug_assert_eq!(compute_result(&new_attempt, n), target);
        best = get_best(best, new_attempt);
    }
    best
}


fn sum_coefficients(result: &ExponentMap) -> u128 {
    result.iter().map(|(_, c)| c).sum::<u128>()
}

fn get_best(a: ExponentMap, b: ExponentMap) -> ExponentMap {
    if sum_coefficients(&a) > sum_coefficients(&b) {
        b
    } else {
        a
    }
}


/// Compute $a_0 × 1^n + a_1 × 2^n + \ldots + a_i × i^n = (i+1)^n$
/// Return the coefficients a_0 .. a_i
fn attack(tries: u32, i: u64, n: u32) -> ExponentMap {
    let target: u128 = (i as u128).pow(n);
    println!("Solving for i={}, n={}, i**n = {}", i, n, target);

    find_smallest_exponent_polynomial(tries, target, i, n)
}

/*
fn load_stored_results() {
    if let Ok(mut file) = File::open("results.txt") {
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let deserialized: HashMap<u32, ExponentMap> = serde_json::from_str(&contents).unwrap();
        let mut map = BEST_RESULTS.lock().unwrap();
        for (k, v) in deserialized.into_iter() {
            map.insert(k, v);
        }
    }
}

fn store_results(results: &HashMap<u32, ExponentMap>) {
    let mut file = File::create("results.txt").unwrap();
    let serialized = serde_json::to_string(&results).unwrap();
    file.write_all(&serialized.into_bytes()).unwrap();
}
*/

fn main() {

    if env::args().count() != 4 {
        println!("Usage: {} i n tries", env::args().nth(0).unwrap());
        panic!("Missing arguments, got {}", env::args().count());
    }
    let n: u32;
    let i;
    let tries;
    {
        i = env::args().nth(1).unwrap().parse().unwrap();
        n = env::args().nth(2).unwrap().parse().unwrap();
        tries = env::args().nth(3).unwrap().parse().unwrap();
    }
    //load_stored_results();

    let mut results= HashMap::new();
    for n in 2..n+1 {
        results.insert(n, attack(tries, i, n));
    }

    //store_results(&results);

    println!("Solutions");
    for k in 2..n+1 {
        let result = results.get(&k).unwrap();
        print!("{}; ", k);
        for j in (2..i).rev() {
            match result.get(&(j as u64)) {
                Some(c) => print!("{},", c),
                None => print!("0,"),
            }
        }
        match result.get(&1) {
            Some(c) => print!("{}", c),
            None => print!("0"),
        }
        println!("");
    }
}

#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_sum_correct() {
        let i = 10;
        let n = 3;
        let result = attack(1, i, n);
        assert_eq!(compute_result(&result, n), (i as u128).pow(n));
        let i = 25;
        let n = 25;
        let result = attack(1, i, n);
        assert_eq!(compute_result(&result, n), (i as u128).pow(n));
    }

    #[test]
    fn test_pow_correct() {
        assert_eq!(10u32.pow(3), 1000);
    }
}
