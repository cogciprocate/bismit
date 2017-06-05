//! Copied from: `https://doc.rust-lang.org/rand/src/rand/lib.rs.html` and
//! `https://doc.rust-lang.org/rand/src/rand/distributions/range.rs.html`.
//!
//! Types below are duplicated here because for some yet unknown reason,
//! deriving `Debug` from the crates.io version (possibly repo as well) does
//! not work. Deriving `Debug` from a struct containing an `XorShiftRng`, for
//! example, errors with:
//!
//! error[E0277]: the trait bound `rand::XorShiftRng: std::fmt::Debug` is not satisfied
//!    --> src/cortex/synapses.rs:106:5
//!     |
//! 106 |     rng: XorShiftRng,
//!     |     ^^^^^^^^^^^^^^^^ `rand::XorShiftRng` cannot be formatted using `:?`; if it is defined in your crate, add `#[derive(Debug)]` or manually implement it
//!     |
//!     = help: the trait `std::fmt::Debug` is not implemented for `rand::XorShiftRng`
//!     = note: required because of the requirements on the impl of `std::fmt::Debug` for `&rand::XorShiftRng`
//!     = note: required for the cast to the object type `std::fmt::Debug`
//!
//!
//! <shrug/>
//!


use rand::{SeedableRng, Rng, Rand};
use rand::os::OsRng;
use rand::distributions::{Sample, IndependentSample};
// use rand::distributions::range::{SampleRange};
use std::num::Wrapping as w;

#[allow(bad_style)]
type w32 = w<u32>;


/// An Xorshift[1] random number
/// generator.
///
/// The Xorshift algorithm is not suitable for cryptographic purposes
/// but is very fast. If you do not know for sure that it fits your
/// requirements, use a more secure one such as `IsaacRng` or `OsRng`.
///
/// [1]: Marsaglia, George (July 2003). ["Xorshift
/// RNGs"](http://www.jstatsoft.org/v08/i14/paper). *Journal of
/// Statistical Software*. Vol. 8 (Issue 14).
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug)]
pub struct XorShiftRng {
    x: w32,
    y: w32,
    z: w32,
    w: w32,
}

impl XorShiftRng {
    /// Creates a new XorShiftRng instance which is not seeded.
    ///
    /// The initial values of this RNG are constants, so all generators created
    /// by this function will yield the same stream of random numbers. It is
    /// highly recommended that this is created through `SeedableRng` instead of
    /// this function
    pub fn new_unseeded() -> XorShiftRng {
        XorShiftRng {
            x: w(0x193a6754),
            y: w(0xa8a7d469),
            z: w(0x97830e05),
            w: w(0x113ba7bb),
        }
    }
}

impl Rng for XorShiftRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let x = self.x;
        let t = x ^ (x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        let w_ = self.w;
        self.w = w_ ^ (w_ >> 19) ^ (t ^ (t >> 8));
        self.w.0
    }
}

impl SeedableRng<[u32; 4]> for XorShiftRng {
    /// Reseed an XorShiftRng. This will panic if `seed` is entirely 0.
    fn reseed(&mut self, seed: [u32; 4]) {
        assert!(!seed.iter().all(|&x| x == 0),
                "XorShiftRng.reseed called with an all zero seed.");

        self.x = w(seed[0]);
        self.y = w(seed[1]);
        self.z = w(seed[2]);
        self.w = w(seed[3]);
    }

    /// Create a new XorShiftRng. This will panic if `seed` is entirely 0.
    fn from_seed(seed: [u32; 4]) -> XorShiftRng {
        assert!(!seed.iter().all(|&x| x == 0),
                "XorShiftRng::from_seed called with an all zero seed.");

        XorShiftRng {
            x: w(seed[0]),
            y: w(seed[1]),
            z: w(seed[2]),
            w: w(seed[3]),
        }
    }
}

impl Rand for XorShiftRng {
    fn rand<R: Rng>(rng: &mut R) -> XorShiftRng {
        let mut tuple: (u32, u32, u32, u32) = rng.gen();
        while tuple == (0, 0, 0, 0) {
            tuple = rng.gen();
        }
        let (x, y, z, w_) = tuple;
        XorShiftRng { x: w(x), y: w(y), z: w(z), w: w(w_) }
    }
}



/// Create a weak random number generator with a default algorithm and seed.
///
/// It returns the fastest `Rng` algorithm currently available in Rust without
/// consideration for cryptography or security. If you require a specifically
/// seeded `Rng` for consistency over time you should pick one algorithm and
/// create the `Rng` yourself.
///
/// This will read randomness from the operating system to seed the
/// generator.
pub fn weak_rng() -> XorShiftRng {
    match OsRng::new() {
        Ok(mut r) => r.gen(),
        Err(e) => panic!("weak_rng: failed to create seeded RNG: {:?}", e)
    }
}


/// Sample values uniformly between two bounds.
///
/// This gives a uniform distribution (assuming the RNG used to sample
/// it is itself uniform & the `SampleRange` implementation for the
/// given type is correct), even for edge cases like `low = 0u8`,
/// `high = 170u8`, for which a naive modulo operation would return
/// numbers less than 85 with double the probability to those greater
/// than 85.
///
/// Types should attempt to sample in `[low, high)`, i.e., not
/// including `high`, but this may be very difficult. All the
/// primitive integer types satisfy this property, and the float types
/// normally satisfy it, but rounding may mean `high` can occur.
///
/// # Example
///
/// ```rust
/// use rand::distributions::{IndependentSample, Range};
///
/// fn main() {
///     let between = Range::new(10, 10000);
///     let mut rng = rand::thread_rng();
///     let mut sum = 0;
///     for _ in 0..1000 {
///         sum += between.ind_sample(&mut rng);
///     }
///     println!("{}", sum);
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Range<X> {
    low: X,
    range: X,
    accept_zone: X
}

impl<X: SampleRange + PartialOrd> Range<X> {
    /// Create a new `Range` instance that samples uniformly from
    /// `[low, high)`. Panics if `low >= high`.
    pub fn new(low: X, high: X) -> Range<X> {
        assert!(low < high, "Range::new called with `low >= high`");
        SampleRange::construct_range(low, high)
    }
}

impl<Sup: SampleRange> Sample<Sup> for Range<Sup> {
    #[inline]
    fn sample<R: Rng>(&mut self, rng: &mut R) -> Sup { self.ind_sample(rng) }
}
impl<Sup: SampleRange> IndependentSample<Sup> for Range<Sup> {
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> Sup {
        SampleRange::sample_range(self, rng)
    }
}


/// The helper trait for types that have a sensible way to sample
/// uniformly between two values. This should not be used directly,
/// and is only to facilitate `Range`.
pub trait SampleRange : Sized {
    /// Construct the `Range` object that `sample_range`
    /// requires. This should not ever be called directly, only via
    /// `Range::new`, which will check that `low < high`, so this
    /// function doesn't have to repeat the check.
    fn construct_range(low: Self, high: Self) -> Range<Self>;

    /// Sample a value from the given `Range` with the given `Rng` as
    /// a source of randomness.
    fn sample_range<R: Rng>(r: &Range<Self>, rng: &mut R) -> Self;
}

macro_rules! integer_impl {
    ($ty:ty, $unsigned:ident) => {
        impl SampleRange for $ty {
            // we play free and fast with unsigned vs signed here
            // (when $ty is signed), but that's fine, since the
            // contract of this macro is for $ty and $unsigned to be
            // "bit-equal", so casting between them is a no-op & a
            // bijection.

            #[inline]
            fn construct_range(low: $ty, high: $ty) -> Range<$ty> {
                let range = (w(high as $unsigned) - w(low as $unsigned)).0;
                let unsigned_max: $unsigned = ::std::$unsigned::MAX;

                // this is the largest number that fits into $unsigned
                // that `range` divides evenly, so, if we've sampled
                // `n` uniformly from this region, then `n % range` is
                // uniform in [0, range)
                let zone = unsigned_max - unsigned_max % range;

                Range {
                    low: low,
                    range: range as $ty,
                    accept_zone: zone as $ty
                }
            }

            #[inline]
            fn sample_range<R: Rng>(r: &Range<$ty>, rng: &mut R) -> $ty {
                loop {
                    // rejection sample
                    let v = rng.gen::<$unsigned>();
                    // until we find something that fits into the
                    // region which r.range evenly divides (this will
                    // be uniformly distributed)
                    if v < r.accept_zone as $unsigned {
                        // and return it, with some adjustments
                        return (w(r.low) + w((v % r.range as $unsigned) as $ty)).0;
                    }
                }
            }
        }
    }
}

integer_impl! { i8, u8 }
integer_impl! { i16, u16 }
integer_impl! { i32, u32 }
integer_impl! { i64, u64 }
integer_impl! { isize, usize }
integer_impl! { u8, u8 }
integer_impl! { u16, u16 }
integer_impl! { u32, u32 }
integer_impl! { u64, u64 }
integer_impl! { usize, usize }

macro_rules! float_impl {
    ($ty:ty) => {
        impl SampleRange for $ty {
            fn construct_range(low: $ty, high: $ty) -> Range<$ty> {
                Range {
                    low: low,
                    range: high - low,
                    accept_zone: 0.0 // unused
                }
            }
            fn sample_range<R: Rng>(r: &Range<$ty>, rng: &mut R) -> $ty {
                r.low + r.range * rng.gen::<$ty>()
            }
        }
    }
}

float_impl! { f32 }
float_impl! { f64 }