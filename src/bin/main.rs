//! Compares the sequential and the parallel merge sorts.

use merge::{parallel, sequential};
use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};
use std::{env, process::exit, str::FromStr, sync::Arc, time::Instant};

fn main() {
    let seed = choose_seed();

    println!("Using seed {}", seed);
    println!();

    let mut rng = StdRng::seed_from_u64(seed);

    run_cases("small", &gen_small_cases(&mut rng));
    println!();
    run_cases("medium", &gen_medium_cases(&mut rng));
    println!();
    run_cases("large", &gen_large_cases(&mut rng));
    println!();
    run_cases("huge", &gen_huge_cases(&mut rng));
}

/// Chooses a seed. If a command line argument is given, it is used as a seed.
/// Otherwise, a random seed is chosen.
fn choose_seed() -> u64 {
    let mut args = env::args();

    args.next();

    let maybe_seed = args.next();

    if args.next().is_some() {
        eprintln!("No more than one argument is allowed (the seed)");
        exit(1);
    }

    match maybe_seed {
        Some(string) => match u64::from_str(&string) {
            Ok(seed) => seed,
            Err(err) => {
                eprintln!("{}", err);
                exit(1);
            },
        },
        None => rand::thread_rng().gen(),
    }
}

/// Runs the given cases for all algorithms and counts the seconds. Prints the
/// result to the screen.
fn run_cases(name: &str, cases: &[Arc<[i64]>]) {
    let then = Instant::now();

    for case in cases {
        sequential::sort(case);
    }

    let elapsed = then.elapsed();

    println!("Sequential took {}s for {}", elapsed.as_secs_f64(), name);

    let then = Instant::now();

    for case in cases {
        parallel::sort(case);
    }

    let elapsed = then.elapsed();

    println!("Parallel took {}s for {}", elapsed.as_secs_f64(), name);
}

/// Generates random small cases.
fn gen_small_cases<R>(rng: R) -> Vec<Arc<[i64]>>
where
    R: Rng,
{
    gen_cases_of_size(rng, 400, 10, 200)
}

/// Generates random medium-sized cases.
fn gen_medium_cases<R>(rng: R) -> Vec<Arc<[i64]>>
where
    R: Rng,
{
    gen_cases_of_size(rng, 200, 500, 10000)
}

/// Generates random large cases.
fn gen_large_cases<R>(rng: R) -> Vec<Arc<[i64]>>
where
    R: Rng,
{
    gen_cases_of_size(rng, 50, 20000, 400000)
}

/// Generates random huge cases.
fn gen_huge_cases<R>(rng: R) -> Vec<Arc<[i64]>>
where
    R: Rng,
{
    gen_cases_of_size(rng, 10, 800000, 2000000)
}

/// Generates random cases of size in the given interval [min_elems, max_elems].
fn gen_cases_of_size<R>(
    mut rng: R,
    num_cases: usize,
    min_elems: usize,
    max_elems: usize,
) -> Vec<Arc<[i64]>>
where
    R: Rng,
{
    let mut targets = Vec::with_capacity(num_cases);

    for _ in 0 .. num_cases {
        let size = rng.sample(Uniform::new_inclusive(min_elems, max_elems));
        let mut target = Vec::<i64>::with_capacity(size);

        for _ in 0 .. size {
            target.push(rng.gen());
        }

        targets.push(Arc::from(target));
    }

    targets
}
