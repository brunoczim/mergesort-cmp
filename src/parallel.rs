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

/// A parallel sorter. Uses a merge sort.
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
    let range = 0 .. array.len();
    sort_range(array, range)
}

/// A parallel sorter parameterized by a comparison function. Uses a merge sort.
///
/// It uses the given comparison function to compare and sorts the whole array.
///
/// # Examples
/// ```rust
/// use mergesort_cmp::parallel;
/// use std::sync::Arc;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
/// let array: Arc<[i32]> = Arc::from(&array as &[_]);
///
/// let sorted = parallel::sort_by(&array, |a, b| b.cmp(&a));
///
/// assert_eq!(sorted, &[91293, 20001, 20000, 12, 7, 5, -1, -12, -95]);
/// ```
pub fn sort_by<T, F>(array: &Arc<[T]>, compare: F) -> Vec<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn(&T, &T) -> Ordering + Send + Sync + 'static,
{
    let range = 0 .. array.len();
    sort_range_by(array, range, compare)
}

/// A parallel sorter parameterized by a range of the array. Uses a merge sort.
///
/// It uses the default comparison order and sorts only the given range.
///
/// # Examples
/// ```rust
/// use mergesort_cmp::parallel;
/// use std::sync::Arc;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
/// let array: Arc<[i32]> = Arc::from(&array as &[_]);
///
/// let sorted = parallel::sort_range(&array, 3 .. 7);
///
/// assert_eq!(sorted, &[-95, 12, 20000, 20001]);
/// ```
pub fn sort_range<T>(array: &Arc<[T]>, range: Range<usize>) -> Vec<T>
where
    T: Ord + Clone + Send + Sync + 'static,
{
    sort_range_by(array, range, Ord::cmp)
}

/// A parallel sorter parameterized by a comparison function, and a range on the
/// array. Uses a merge sort.
///
/// It uses the given comparison function to compare and sorts the given range
/// of the array.
///
/// # Examples
/// ```rust
/// use mergesort_cmp::parallel;
/// use std::sync::Arc;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
/// let array: Arc<[i32]> = Arc::from(&array as &[_]);
///
/// let sorted = parallel::sort_range_by(&array, 3 .. 7, |a, b| b.cmp(&a));
///
/// assert_eq!(sorted, &[20001, 20000, 12, -95]);
/// ```
pub fn sort_range_by<T, F>(
    array: &Arc<[T]>,
    range: Range<usize>,
    compare: F,
) -> Vec<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn(&T, &T) -> Ordering + Send + Sync + 'static,
{
    let num_cpus = num_cpus::get();
    let threads = if num_cpus.is_power_of_two() {
        num_cpus
    } else {
        num_cpus.next_power_of_two()
    };
    let compare_arc = Arc::new(compare);
    split(&array, range, &compare_arc, threads)
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
