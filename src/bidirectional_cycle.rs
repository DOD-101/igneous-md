use std::iter::Iterator;

// TODO: This implementing needs to be looked over closer at a future data.
// As of current it seems to work, but I haven't taken the time to properly inspect it.

// This struct holds our original iterator and maintains a cached version of its elements
#[derive(Debug, Clone)]
pub struct BiCycle<I>
where
    I: Iterator,
{
    // The original iterator we're adapting
    orig: I,
    // Cache of items we've seen so far
    cache: Vec<I::Item>,
    // Current position in the cache
    position: usize,
    // Whether we've exhausted the original iterator
    exhausted: bool,
}

// This is the extension trait that adds the bi_cycle() method to iterators
pub trait BiCyclable: Iterator {
    fn bi_cycle(self) -> BiCycle<Self>
    where
        Self: Sized,
    {
        BiCycle {
            orig: self,
            cache: Vec::new(),
            position: 0,
            exhausted: false,
        }
    }
}

// Implement BiCyclable for all types that implement Iterator
impl<T: Iterator> BiCyclable for T {}

// Implement Iterator for BiCycle
impl<I> Iterator for BiCycle<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // If we haven't exhausted the original iterator and we're at the end of our cache
        if !self.exhausted {
            // Try to get the next item from the original iterator
            if let Some(item) = self.orig.next() {
                self.cache.push(item);
            } else {
                self.exhausted = true;
                // If we have no cached items, we're done
                if self.cache.is_empty() {
                    return None;
                }
            }
        }

        // At this point, we must have items in the cache
        let item = self.cache[self.position].clone();
        self.position = (self.position + 1) % self.cache.len();
        Some(item)
    }
}

// Implement DoubleEndedIterator for BiCycle
impl<I> DoubleEndedIterator for BiCycle<I>
where
    I: Iterator,
    I::Item: Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // If we haven't exhausted the original iterator, we need to consume it all
        if !self.exhausted {
            self.cache.extend(self.orig.by_ref());
            self.exhausted = true;
            if self.cache.is_empty() {
                return None;
            }
        }

        // Now we can move backwards through our cache
        if self.position == 0 {
            self.position = self.cache.len() - 1;
        } else {
            self.position -= 1;
        }

        Some(self.cache[self.position].clone())
    }
}
