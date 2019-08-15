use crate::feature::*;
use core::UriBound;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::ffi::CStr;
use std::fmt;
use std::marker::PhantomData;
use std::num::NonZeroU32;

/// Representation of a URI for fast comparisons.
///
/// A URID is basically a number which represents a URI, which makes the identification of other features faster and easier. The mapping of URIs to URIDs is handled by the host and plugins can retrieve them using the [`Map`](struct.Map.html) feature. A given URID can also be converted back to a URI with the [`Unmap`](struct.Unmap.html) feature.
///
/// This struct has an optional type parameter `T` which defaults to `()`. In this case, the type can represent any URID at all, but if `T` is a `UriBound`, the type can only describe the URID of the given bound. This makes creation easier, since you only need the mapper to create it, and also turns it into an atomic [`URIDCache`](trait.URIDCache.html), which can be used to build bigger caches.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct URID<T = ()>(NonZeroU32, PhantomData<T>);

/// Abstraction of types that store URIDs.
///
/// This trait makes the creation of static URID caches easy: You simply define the cache and derive `URIDCache` for it, and you have a single method to create it.
///
/// Usage example:
///
///     use lv2_core::UriBound;
///     use lv2_urid::*;
///
///     struct MyTypeA();
///     
///     unsafe impl UriBound for MyTypeA {
///         const URI: &'static [u8] = b"urn:my-type-a\0";
///     }
///     
///     struct MyTypeB();
///     
///     unsafe impl UriBound for MyTypeB {
///         const URI: &'static [u8] = b"urn:my-type-b\0";
///     }
///
///     #[derive(URIDCache)]
///     struct MyCache {
///         my_type_a: URID<MyTypeA>,
///         my_type_b: URID<MyTypeB>,
///     }
///
///     fn build_cache(map: &Map) -> MyCache {
///         MyCache::from_map(map).unwrap()
///     }
pub trait URIDCache: Sized {
    /// Construct the cache from the mapper.
    fn from_map(map: &Map) -> Option<Self>;
}

impl<T> URID<T> {
    /// Create a URID without checking for type or value validity.
    ///
    /// If `T` is a UriBound, `urid` may only be the URID which is mapped to the URI returned by `T::uri()`. Additionally, a URID may never be zero. Since all of these constraints are not checked by this method, it is unsafe.
    pub unsafe fn new_unchecked(urid: u32) -> Self {
        Self(NonZeroU32::new_unchecked(urid), PhantomData)
    }

    /// Return the raw URID number.
    pub fn get(&self) -> u32 {
        self.0.get()
    }

    /// Unmap the URID to its string representation.
    pub fn into_cstr<'a>(self, unmap: &'a Unmap) -> Option<&'a CStr> {
        unmap.unmap(self)
    }
}

impl URID<()> {
    /// Create a URID instance.
    ///
    /// Since URIDs may never be zero, this method will return `None` if `urid` is zero.
    pub fn new(urid: u32) -> Option<Self> {
        Some(Self(NonZeroU32::new(urid)?, PhantomData))
    }

    /// Create a URID by retrieving the URID of the given URI.
    ///
    /// This is basically an alias for `map.map_uri(uri)`.
    pub fn from_uri(map: &Map, uri: &CStr) -> Option<Self> {
        map.map_uri(uri)
    }
}

impl<T: UriBound> URID<T> {
    /// Create a URID by retrieving the URID of the given type.
    pub fn from_type(map: &Map) -> Option<Self> {
        map.map_type::<T>()
    }

    /// Transform the type-specific URID into a generalized one.
    pub fn into_general(self) -> URID<()> {
        URID::new(self.get()).unwrap()
    }
}

impl<T> fmt::Debug for URID<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> PartialEq<u32> for URID<T> {
    fn eq(&self, other: &u32) -> bool {
        self.get() == *other
    }
}

impl<T> PartialEq<URID<T>> for u32 {
    fn eq(&self, other: &URID<T>) -> bool {
        *self == other.get()
    }
}

impl<T> PartialOrd<u32> for URID<T> {
    fn partial_cmp(&self, other: &u32) -> Option<Ordering> {
        self.get().partial_cmp(other)
    }
}

impl<T> PartialOrd<URID<T>> for u32 {
    fn partial_cmp(&self, other: &URID<T>) -> Option<Ordering> {
        self.partial_cmp(&other.get())
    }
}

impl<T: UriBound> URIDCache for URID<T> {
    fn from_map(map: &Map) -> Option<Self> {
        Self::from_type(map)
    }
}

#[cfg(test)]
mod tests {
    use crate::URID;

    #[test]
    fn test_urid_size() {
        use std::mem::size_of;

        let size = size_of::<u32>();

        assert_eq!(size, size_of::<URID>());
        assert_eq!(size, size_of::<Option<URID>>());
    }
}
