use std::sync::Arc;

use memmap2::Mmap;

/// A self-referencing wrapper around [`qfilter::FilterRef`] backed by an [`Arc<Mmap>`].
///
/// This is analogous to [`ArcBytes`](crate::arc_bytes::ArcBytes): it stores a `FilterRef<'static>`
/// whose borrowed buffer actually points into the mmap, which is kept alive by the `Arc<Mmap>`.
/// This avoids copying the filter's byte buffer during deserialization — the `FilterRef` borrows
/// directly from the memory-mapped file.
pub struct ArcFilterRef {
    /// The filter with an erased lifetime. Safe because `_backing` keeps the data alive.
    filter: qfilter::FilterRef<'static>,
    /// Prevents the mmap from being dropped while `filter` references it.
    _backing: Arc<Mmap>,
}

// Safety: The backing Arc<Mmap> is Send+Sync, and FilterRef is just a read-only view into it.
unsafe impl Send for ArcFilterRef {}
unsafe impl Sync for ArcFilterRef {}

impl ArcFilterRef {
    /// Creates an `ArcFilterRef` by deserializing a `FilterRef` that borrows from the given mmap
    /// data.
    ///
    /// # Safety
    ///
    /// `data` must be a subslice of `backing` and `backing` must be the sole owner keeping that
    /// memory alive. The returned `ArcFilterRef` will hold a clone of `backing` to ensure the
    /// memory remains valid.
    pub unsafe fn from_mmap_slice(backing: &Arc<Mmap>, data: &[u8]) -> Result<Self, pot::Error> {
        // Deserialize borrowing from `data`, which lives inside the mmap.
        let filter: qfilter::FilterRef<'_> = pot::from_slice(data)?;
        // Safety: erase the lifetime — the backing Arc<Mmap> keeps the data alive.
        let filter: qfilter::FilterRef<'static> = unsafe { std::mem::transmute(filter) };
        Ok(Self {
            filter,
            _backing: backing.clone(),
        })
    }

    /// Returns a reference to the underlying [`FilterRef`].
    #[inline]
    pub fn as_filter_ref(&self) -> &qfilter::FilterRef<'static> {
        &self.filter
    }

    /// Convenience: check if a fingerprint is probably in the filter.
    #[inline]
    pub fn contains_fingerprint(&self, hash: u64) -> bool {
        self.filter.contains_fingerprint(hash)
    }

    /// Convenience: check if the filter is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.filter.is_empty()
    }

    /// Convenience: number of items in the filter.
    #[inline]
    pub fn len(&self) -> u64 {
        self.filter.len()
    }

    /// Convert to an owned [`qfilter::Filter`] (copies the buffer).
    pub fn to_owned_filter(&self) -> qfilter::Filter {
        self.filter.to_owned()
    }
}

impl std::fmt::Debug for ArcFilterRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArcFilterRef")
            .field("filter", &self.filter)
            .finish()
    }
}
