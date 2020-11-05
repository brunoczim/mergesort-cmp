//! Compares the sequential and the parallel merge sorts.

use mergesort_cmp::{parallel, sequential};
use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};
use std::{env, process::exit, str::FromStr, sync::Arc, time::Instant};

type Data = i64;

fn main() {
    let seed = choose_seed();

    println!("Using seed {}", seed);

    let mut rng = StdRng::seed_from_u64(seed);

    println!();
    CaseSet::tiny(&mut rng).run_for_all_targets();
    println!();
    CaseSet::small(&mut rng).run_for_all_targets();
    println!();
    CaseSet::medium(&mut rng).run_for_all_targets();
    println!();
    CaseSet::big(&mut rng).run_for_all_targets();
    println!();
    CaseSet::large(&mut rng).run_for_all_targets();
    println!();
    CaseSet::huge(&mut rng).run_for_all_targets();
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

/// A set of test cases generated randomly.
#[derive(Debug, Clone)]
struct CaseSet<'name> {
    name: &'name str,
    min_size: usize,
    max_size: usize,
    cases: Vec<Arc<[Data]>>,
}

impl<'name> CaseSet<'name> {
    /// Generates a random case set, of `count` number of cases, and `min_size`
    /// and `max_size` as bounds for the array sizes ([min, max], i.e. max
    /// inclusive).
    fn random<R>(
        name: &'name str,
        count: usize,
        min_size: usize,
        max_size: usize,
        mut rng: R,
    ) -> CaseSet
    where
        R: Rng,
    {
        let mut cases = Vec::with_capacity(count);

        for _ in 0 .. count {
            let size = rng.sample(Uniform::new_inclusive(min_size, max_size));
            let mut case = Vec::<Data>::with_capacity(size);

            for _ in 0 .. size {
                case.push(rng.gen());
            }

            cases.push(Arc::from(case));
        }

        Self { name, min_size, max_size, cases }
    }

    /// Generates case set of "tiny" array sizes.
    fn tiny<R>(rng: R) -> Self
    where
        R: Rng,
    {
        Self::random("tiny", 5120, 1, 50, rng)
    }

    /// Generates case set of "small" array sizes.
    fn small<R>(rng: R) -> Self
    where
        R: Rng,
    {
        Self::random("small", 1280, 100, 500, rng)
    }

    /// Generates case set of "medium" array sizes.
    fn medium<R>(rng: R) -> Self
    where
        R: Rng,
    {
        Self::random("medium", 320, 1000, 5000, rng)
    }

    /// Generates case set of "big" array sizes.
    fn big<R>(rng: R) -> Self
    where
        R: Rng,
    {
        Self::random("big", 80, 10000, 50000, rng)
    }

    /// Generates case set of "large" array sizes.
    fn large<R>(rng: R) -> Self
    where
        R: Rng,
    {
        Self::random("large", 20, 100000, 500000, rng)
    }

    /// Generates case set of "huge" array sizes.
    fn huge<R>(rng: R) -> Self
    where
        R: Rng,
    {
        Self::random("huge", 5, 1000000, 5000000, rng)
    }

    /// Runs the case set for the given target sort function.
    fn run_for_target<F>(&self, target_name: &str, mut target: F)
    where
        F: FnMut(&Arc<[Data]>) -> Vec<Data>,
    {
        let then = Instant::now();

        for case in &self.cases {
            target(case);
        }

        let elapsed = then.elapsed();

        println!("Target {} took {}s", target_name, elapsed.as_secs_f64(),)
    }

    /// Runs the case set for all targets sort function.
    fn run_for_all_targets(&self) {
        println!(
            "Case set {}, min size = {}, max size = {}, cases = {}",
            self.name,
            self.min_size,
            self.max_size,
            self.cases.len()
        );

        self.run_for_target("sequential", |array| sequential::sort(array));

        let mut logical_cpus = parallel::SortOptions::default_order();
        logical_cpus.thread_per_cpu();
        self.run_for_target("parallel logical", |array| {
            logical_cpus.run(array)
        });

        let mut physical_cpus = parallel::SortOptions::default_order();
        physical_cpus.thread_per_physical_cpu();
        self.run_for_target("parallel physical", |array| {
            physical_cpus.run(array)
        });
    }
}
