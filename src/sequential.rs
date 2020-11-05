//! This module provides a regular merge sort.
//!
//! # Examples
//! ```rust
//! use merge::sequential::sort;
//!
//! let count = 10000;
//! let expected = (0 .. count).collect::<Vec<_>>();
//! let reversed = (0 .. count).rev().collect::<Vec<_>>();
//!
//! let sorted = sort(&reversed);
//!
//! assert_eq!(expected, sorted);
//! ```

use std::cmp::Ordering;

/// Sorts the given array using the default order. Uses a merge sort.
///
/// # Examples
/// ```rust
/// use merge::sequential;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
///
/// let sorted = sequential::sort(&array);
///
/// assert_eq!(sorted, &[-95, -12, -1, 5, 7, 12, 20000, 20001, 91293]);
/// ```
pub fn sort<T>(array: &[T]) -> Vec<T>
where
    T: Ord + Clone,
{
    sort_by(array, Ord::cmp)
}

/// A sorter parameterized by a comparison function. Uses a merge sort.
///
/// It uses the given comparison function to compare and sorts the whole array.
///
/// # Examples
/// ```rust
/// use merge::sequential;
///
/// let array = [-1, 5, 91293, 12, -95, 20000, 20001, -12, 7];
///
/// let sorted = sequential::sort_by(&array, |a, b| b.cmp(&a));
///
/// assert_eq!(sorted, &[91293, 20001, 20000, 12, 7, 5, -1, -12, -95]);
/// ```
pub fn sort_by<T, F>(array: &[T], mut compare: F) -> Vec<T>
where
    T: Ord + Clone,
    F: FnMut(&T, &T) -> Ordering,
{
    split(array, &mut compare)
}

/// Performs the "split" step of the merge sort algorithm, and then merges the
/// sorted halves.
fn split<T, F>(array: &[T], compare: &mut F) -> Vec<T>
where
    T: Clone,
    F: FnMut(&T, &T) -> Ordering,
{
    if array.len() > 1 {
        // The middle index: (length + 1)/2
        let half = (array.len() + 1) / 2;

        // Splits the slice in two.
        let (lower_slice, upper_slice) = array.split_at(half);

        // Executes the split on the lower half.
        let lower = split(lower_slice, compare);
        // Executes the split on the upper half.
        let upper = split(upper_slice, compare);

        // Merges the two halves.
        merge(lower, upper, compare)
    } else {
        // Converts the range of a immutable referenced array into a mutable,
        // owned vector. Returns it.
        array.to_vec()
    }
}

/// Merges two halves of a sorting target.
fn merge<T, F>(lower: Vec<T>, upper: Vec<T>, compare: &mut F) -> Vec<T>
where
    F: FnMut(&T, &T) -> Ordering,
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
    while merge_while_less(&mut upper_iter, &mut pivot, &mut merged, compare)
        && merge_while_less(&mut lower_iter, &mut pivot, &mut merged, compare)
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
    compare: &mut F,
) -> bool
where
    I: Iterator,
    F: FnMut(&I::Item, &I::Item) -> Ordering,
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
