//! This module provides an ok-ish parallel sort. Not so funny, but better than
//! spawning a thread everytime.
//!
//! # Examples
//! ```rust
//! use mergesort_cmp::parallel;
//! use std::sync::Arc;
//!
//! let count = 10000;
//! let expected = (0 .. count).collect::<Vec<_>>();
//! let reversed = (0 .. count).rev().collect::<Vec<_>>();
//! let array: Arc<[i32]> = Arc::from(reversed);
//!
//! let sorted = parallel::sort(&array);
//!
//! assert_eq!(expected, sorted);
//! ```

use std::{cmp::Ordering, ops::Range, sync::Arc, thread};

/// A parallel merge sort. This function uses the default order, sorts the whole
/// array, and spawns 1 thread per logical CPU. For customization, see
/// [`SortOptions`].
///
/// It uses the default comparison order and sorts the whole array.
///
/// # Examples
/// ```rust
/// use mergesort_cmp::parallel;
/// use std::sync::Arc;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
/// let array: Arc<[i32]> = Arc::from(&array as &[_]);
///
/// let sorted = parallel::sort(&array);
///
/// assert_eq!(sorted, &[-95, -12, -1, 5, 7, 12, 20000, 20001, 91293]);
/// ```
pub fn sort<T>(array: &Arc<[T]>) -> Vec<T>
where
    T: Ord + Clone + Send + Sync + 'static,
{
    SortOptions::default_order().run(array)
}

/// Options to configure the parallel merge sort.
///
/// # Examples
///
/// ## Default Order
/// ```rust
/// use mergesort_cmp::parallel;
/// use std::sync::Arc;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
/// let array: Arc<[i32]> = Arc::from(&array as &[_]);
///
/// let sorted = parallel::SortOptions::default_order().run(&array);
///
/// assert_eq!(sorted, &[-95, -12, -1, 5, 7, 12, 20000, 20001, 91293]);
/// ```
///
/// ## Reverse Order
/// ```rust
/// use mergesort_cmp::parallel;
/// use std::sync::Arc;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
/// let array: Arc<[i32]> = Arc::from(&array as &[_]);
///
/// let sorted = parallel::SortOptions::reverse_order().run(&array);
///
/// assert_eq!(sorted, &[91293, 20001, 20000, 12, 7, 5, -1, -12, -95]);
/// ```
///
/// ## Separates Even Numbers From Odd Numbers
/// ```rust
/// use mergesort_cmp::parallel;
/// use std::sync::Arc;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
/// let array: Arc<[i32]> = Arc::from(&array as &[_]);
///
/// let compare = |left: &i32, right: &i32| {
///     (left & 1).cmp(&(right & 1)).then(left.cmp(right))
/// };
/// let sorted = parallel::SortOptions::from_compare(compare).run(&array);
///
/// assert_eq!(sorted, &[-12, 12, 20000, -95, -1, 5 ,7, 20001, 91293]);
/// ```
///
/// ## Custom Range And Custom Thread Number
/// ```rust
/// use mergesort_cmp::parallel;
/// use std::sync::Arc;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
/// let array: Arc<[i32]> = Arc::from(&array as &[_]);
///
/// let sorted = parallel::SortOptions::default_order()
///     .range(3 .. 7)
///     .threads(8)
///     .run(&array);
///
/// assert_eq!(sorted, &[-95, 12, 20000, 20001]);
/// ```
pub struct SortOptions<F> {
    /// On how many threads the sorting will be executed.
    threads: usize,
    /// Comparison function.
    compare: Arc<F>,
    /// What range of the array will be sorted. `None` automatically selects
    /// the full array.
    range: Option<Range<usize>>,
}

impl<T> SortOptions<fn(&T, &T) -> Ordering>
where
    T: Ord,
{
    /// Initializes the options using the default comparison order.
    pub fn default_order() -> Self {
        Self {
            threads: num_cpus::get(),
            compare: Arc::new(Ord::cmp),
            range: None,
        }
    }

    /// Initalizes the options using the reversed comparison order.
    pub fn reverse_order() -> Self {
        Self {
            threads: num_cpus::get(),
            compare: Arc::new(|a, b| b.cmp(a)),
            range: None,
        }
    }
}

impl<F> SortOptions<F> {
    /// Initalizes the options from a custom comparison function.
    pub fn from_compare(compare: F) -> Self {
        Self {
            threads: num_cpus::get(),
            compare: Arc::new(compare),
            range: None,
        }
    }

    /// Sets the number of threads used.
    pub fn threads(&mut self, threads: usize) -> &mut Self {
        self.threads = threads;
        self
    }

    /// Sets the number of threads to the number of logical CPUs (default).
    pub fn thread_per_cpu(&mut self) -> &mut Self {
        self.threads(num_cpus::get())
    }

    /// Sets the number of threads to the number of physical CPUs.
    pub fn thread_per_physical_cpu(&mut self) -> &mut Self {
        self.threads(num_cpus::get_physical())
    }

    /// Sets the range of the array on which sort will happen.
    pub fn range(&mut self, range: Range<usize>) -> &mut Self {
        self.range = Some(range);
        self
    }

    /// Sets the array to be fully sorted (default).
    pub fn full_range(&mut self) -> &mut Self {
        self.range = None;
        self
    }

    /// Sorts the given array using the given options.
    pub fn run<T>(&self, array: &Arc<[T]>) -> Vec<T>
    where
        F: Fn(&T, &T) -> Ordering + Send + Sync + 'static,
        T: Clone + Send + Sync + 'static,
    {
        let range = self.range.clone().unwrap_or(0 .. array.len());
        split(array, range, &self.compare, self.threads)
    }
}

/// Performs the "split" step of the merge sort algorithm, and then merges the
/// sorted halves.
fn split<T, F>(
    array: &Arc<[T]>,
    range: Range<usize>,
    compare: &Arc<F>,
    threads: usize,
) -> Vec<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn(&T, &T) -> Ordering + Send + Sync + 'static,
{
    if range.len() > 1 {
        // The middle index: start + (end - start + 1)/2
        let half = range.start + (range.len() + 1) / 2;

        // The lower half range.
        let lower_range = range.start .. half;

        // The upper half range.
        let upper_range = half .. range.end;

        // If there are threads, do the split in separated threads.
        let (lower, upper) = if threads > 1 {
            // Spawns the thread that sorts the lower half.
            let upper_handle = {
                // Clones the array's ARC (Atomic Reference Counter).
                let array = array.clone();
                // Clones the comparison function's ARC.
                let compare = compare.clone();

                // Executes the split on the upper half.
                thread::spawn(move || {
                    split(&array, upper_range, &compare, threads / 2)
                })
            };

            // Executes the split on the lower half.
            let lower = split(array, lower_range, compare, threads / 2);
            // Joins the lower thread.
            let upper = upper_handle.join().expect("thread failed");

            (lower, upper)
        } else {
            // Executes the split on the lower half.
            let lower = split(array, lower_range, compare, 1);
            // Executes the split on the upper half.
            let upper = split(array, upper_range, compare, 1);

            (lower, upper)
        };

        // Merges the two halves.
        merge(lower, upper, compare)
    } else {
        // Converts the range of a reference counted, immutable array into a
        // mutable, owned vector. Returns it.
        array[range].to_vec()
    }
}

/// Merges two halves of a sorting target.
fn merge<T, F>(lower: Vec<T>, upper: Vec<T>, compare: &Arc<F>) -> Vec<T>
where
    F: Fn(&T, &T) -> Ordering,
{
    let mut merged = Vec::with_capacity(lower.len() + upper.len());
    // Iterator over the lower half. Takes the vector away.
    let mut lower_iter = lower.into_iter();
    // Iterator over the upper half. Takes the vector away.
    let mut upper_iter = upper.into_iter();

    // Initializes the "pivot".
    let mut pivot = lower_iter.next();

    // Intercalates the merge of the upper half with the merge lower half,
    // according to the pivot element.
    while merge_while_less(&mut upper_iter, &mut pivot, &mut merged, &compare)
        && merge_while_less(&mut lower_iter, &mut pivot, &mut merged, &compare)
    {
    }

    // Returns the merged vector.
    merged
}

/// Merges the given half into the merged elements vector while the yielded
/// elements are less than the pivot. When a greater than or equal element is
/// found, it becomes the new pivot. Returns whether there is a pivot.
fn merge_while_less<I, F>(
    mut half: I,
    pivot: &mut Option<I::Item>,
    merged: &mut Vec<I::Item>,
    compare: &Arc<F>,
) -> bool
where
    I: Iterator,
    F: Fn(&I::Item, &I::Item) -> Ordering,
{
    // Finds out if there is a pivot. It will set the pivot to None.
    let pivot_elem = match pivot.take() {
        // Some pivot? Good. Use it.
        Some(elem) => elem,
        // Append the remaining items from the iterator and return.
        None => {
            merged.extend(half);
            return false;
        },
    };

    // Loops until there is an element and it is less.
    loop {
        // Gets the next element, if any.
        let elem = match half.next() {
            // Some element? Good. Use it.
            Some(elem) => elem,
            // No element? Append the pivot and return.
            None => {
                merged.push(pivot_elem);
                return true;
            },
        };

        // Is greater than or equal? Change pivot and return.
        if compare(&elem, &pivot_elem) >= Ordering::Equal {
            *pivot = Some(elem);
            // Don't forget to save the previous pivot.
            merged.push(pivot_elem);
            return true;
        }

        // Less? Ok, add it ot the merged vector.
        merged.push(elem);
    }
}
