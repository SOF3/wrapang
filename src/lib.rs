#![deny(missing_docs)]

//! Provides a wrapping angle struct that stores the angle in a uniformly-distributed `u32` field.

use std::fmt;
use std::num::Wrapping;
use std::ops;

use derive_more::{Add, AddAssign, Neg, Rem, Sub, SubAssign};

const TWO_PI: f64 = 6.2831853071795865f64;

/// A wrapping angle stored in the `u32` field.
///
/// When the angle is added beyond a full circle, it wraps back to zero. This behaviour is perfect
/// for storing direction bearings.
///
/// # Warning
/// Be careful when using the `Mul`/`Div`/`Rem` operators.
/// Since this struct wraps back to zero upon reaching a full circle,
/// multiplying an obtuse angle by 4 then dividing by 4 again yields an acute angle.
/// This is because this struct does not store the number of revolutions it has wrapped.
#[derive(Clone, Copy, Add, AddAssign, Sub, SubAssign, Neg, Rem, PartialEq, Eq)]
pub struct Angle(pub Wrapping<u32>);

impl Angle {
    /// Creates an angle from the radians representation.
    #[inline]
    pub fn from_radians(rad: f64) -> Self {
        assert!(
            rad.is_finite(),
            "Expected finite number of radians, got {:?}",
            rad
        );
        unsafe {
            // finite checked above
            Self::from_radians_unchecked(rad)
        }
    }

    /// Creates an angle from the radians representation.
    ///
    /// # Safety
    /// If the float is infinite or NaN, [cast from float to int is currently undefined
    /// behaviour][inf-ub].
    ///
    /// [inf-ub]: https://github.com/rust-lang/rust/issues/10184
    #[inline] // Inlining allows the compiler to catch potential const expr optimizations
    pub unsafe fn from_radians_unchecked(rad: f64) -> Self {
        let mut unit = (rad / TWO_PI).fract();
        if unit < 0.0 {
            unit += 1.0;
        }
        Self::from_unit_unchecked(unit)
    }

    /// Creates an angle from the degrees representation.
    #[inline]
    pub fn from_degrees(deg: f64) -> Self {
        let mut unit = (deg / 360f64).fract();
        if unit < 0.0 {
            unit += 1.0;
        }
        Self::from_unit(unit)
    }

    /// Creates an angle from the degrees representation.
    ///
    /// # Safety
    /// If the float is infinite or NaN, [cast from float to int is currently undefined
    /// behaviour][inf-ub].
    ///
    /// [inf-ub]: https://github.com/rust-lang/rust/issues/10184
    #[inline] // Inlining allows the compiler to catch potential const expr optimizations
    pub unsafe fn from_degreess_unchecked(rad: f64) -> Self {
        let mut unit = (rad / 360f64).fract();
        if unit < 0.0 {
            unit += 1.0;
        }
        Self::from_unit_unchecked(unit)
    }

    /// Creates an angle from a value in the range `[0, 1]` such that `0` represents a zero angle
    /// and `1`  represents a full angle.
    ///
    /// # Panics
    /// `unit` must be in the range `[0, 1]`.
    /// If possible, callers should try to make `unit` *strictly less than* `1`, by calling the
    /// `fract()` method on `unit` first.
    #[inline]
    pub fn from_unit(unit: f64) -> Self {
        assert!(
            unit.is_finite() && unit >= 0.0 && unit <= 1.0,
            "unit must be in the range [0, 1], got {:?}",
            unit
        );
        unsafe {
            // safety checked above
            Self::from_unit_unchecked(unit)
        }
    }

    /// Creates an angle from a value in the range `[0, 1)` such that
    ///
    /// # Safety
    /// `unit` must be in the range `[0, 1]`.
    /// If it is a finite float outside this range, it is *defined but unexpected* behaviour that
    /// `unit` will be clamped to this range.
    /// If it is an infinite float or NaN, it is *undefined behaviour* because [cast from float to
    /// int is currently UB][inf-ub].
    ///
    /// [inf-ub]: https://github.com/rust-lang/rust/issues/10184
    #[inline]
    pub unsafe fn from_unit_unchecked(unit: f64) -> Self {
        // We convert to `u64` first, because `1 - f64::MACHINE_EPSILON` should give `0u32`
        // instead of `0x_7fff_ffff_u32`.
        Angle(Wrapping(
            (0x1_0000_000u64 as f64 * unit).round() as u64 as u32
        ))
    }

    /// Returns the angle as a value in the range `[0, 1)`, where `0` is zero angle and `1` is a
    /// whole circle.
    #[inline]
    pub fn as_unit(self) -> f64 {
        (self.0).0 as f64 / 0x1_0000_000_u64 as f64
    }

    /// Returns the angle as a value in the range [-0.5, 0.5), where 0 is zero angle and 0.25 is a
    /// right angle (i.e. 90 degrees, &pi;/2 radians).
    #[inline]
    pub fn as_signed_unit(self) -> f64 {
        // The number is first cast to i32 (a no-op) so that values greater than half become signed
        (self.0).0 as i32 as f64 / 0x1_0000_000_u64 as f64
    }

    /// Creates an angle from the radians representation in the range [0, 2&pi;).
    #[inline]
    pub fn as_radians(self) -> f64 {
        self.as_unit() * TWO_PI
    }

    /// Creates an angle from the radians representation in the range [-&pi;, &pi;).
    #[inline]
    pub fn as_signed_radians(self) -> f64 {
        self.as_signed_unit() * TWO_PI
    }

    /// Creates an angle from the degrees representation in the range [0, 360).
    #[inline]
    pub fn as_degrees(self) -> f64 {
        self.as_unit() * 360f64
    }

    /// Creates an angle from the degrees representation in the range [-180, 180).
    #[inline]
    pub fn as_signed_degrees(self) -> f64 {
        self.as_signed_unit() * 360f64
    }

    /// Creates an angle from a `u32`.
    ///
    /// The resultant angle is `i / 2^32` of a whole circle.
    /// This function does **not** take an integer radians/degrees value.
    ///
    /// This method is only useful for serialization purposes or for hardcoding simple angles like
    /// a zero angle of a half angle.
    pub fn from_u32(i: u32) -> Self {
        Self(Wrapping(i))
    }

    /// Converts a `u32` to an angle.
    ///
    /// The resultant value is in a unit such that `2^32` is a whole circle.
    /// This function does **not** take an integer radians/degrees value.
    pub fn as_u32(self) -> u32 {
        (self.0).0
    }

    /// Computes the sine of this angle.
    #[inline] // f64::sin() is also inlined, so Angle::sin() might also benefit from inlining
    pub fn sin(self) -> f64 {
        self.as_radians().sin()
    }

    /// Computes the arcsine angle of a number.
    ///
    /// # Panics
    /// Panics if `x` is outside the range [-1, 1].
    #[inline]
    pub fn asin(x: f64) -> Self {
        Self::from_radians(x.asin())
    }

    /// Computes the cosine of this angle.
    #[inline]
    pub fn cos(self) -> f64 {
        self.as_radians().cos()
    }

    /// Computes the arccosine angle of a number.
    ///
    /// # Panics
    /// Panics if `x` is outside the range [-1, 1].
    #[inline]
    pub fn acos(x: f64) -> Self {
        Self::from_radians(x.acos())
    }

    /// Simultaneously computes the sine and cosine of this angle. Returns `(sin(self),
    /// cos(self))`.
    #[inline]
    pub fn sin_cos(self) -> (f64, f64) {
        self.as_radians().sin_cos()
    }

    /// Computes the tangent of this angle.
    #[inline]
    pub fn tan(self) -> f64 {
        self.as_radians().tan()
    }

    /// Computes the arctangent angle of a number.
    #[inline]
    pub fn atan(x: f64) -> Self {
        Self::from_radians(x.atan())
    }

    /// Computes the four-quadrant arctangent angle of a ratio.
    ///
    /// See [f64::atan2](https://doc.rust-lang.org/std/primitive.f64.html#method.atan2) for
    /// detailed semantics.
    #[inline]
    pub fn atan2(y: f64, x: f64) -> Self {
        Self::from_radians(y.atan2(x))
    }

    /// Rounds the angle to the nearest multiple of `unit`.
    ///
    /// This method treats the angle as unsigned.
    ///
    /// If two multiples are equally near, the larger one is preferred.
    ///
    /// # Warning
    /// Due to wrapping, this function may not work as expected if the whole circle is not an
    /// (almost-) integer multiple of the `unit` angle. This function is primarily expected to be
    /// used with integer factors of the whole circle, e.g. right angles (useful for compass
    /// bearing detection).
    #[must_use = "round() returns a new value and does not modify the receiver"]
    pub fn round(self, unit: Angle) -> Self {
        let times = self.as_u32() / unit.as_u32();
        let (min, max) = (unit.as_u32() * times, unit.as_u32() * (times + 1));
        let rounded = if max - self.as_u32() <= self.as_u32() - min {
            max
        } else {
            min
        };
        Angle::from_u32(rounded)
    }
}

impl fmt::Debug for Angle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Angle({}, {} of whole circle)", self.as_u32(), self.as_unit())
    }
}

/// Multiplies the angle by an integer, wrapping if overflowed.
impl ops::Mul<u32> for Angle {
    type Output = Angle;

    #[inline] // Inlining allows const arithmetic optimization
    fn mul(self, other: u32) -> Angle {
        Angle(self.0 * Wrapping(other))
    }
}

/// Divides the angle by an integer, wrapping if overflowed.
///
/// The angle is treated as signed.
///
/// # Warning
/// ## Wrapping
/// Due to wrapping, division might not work as expected.
/// For example, `Angle::from_degrees(270) * 2 / 2` is equal to `Angle::from_degrees(90)` instead
/// of the original value.
/// Always make sure that the divided angle is supposed to represent an angle in the range [-&pi;,
/// &pi;].
///
/// ```
/// use wrapang::Angle;
///
/// let angle = Angle::from_degrees(90.0);
/// assert_eq!(angle * 2 / 2, Angle::from_degrees(90.0));
///
/// let angle = Angle::from_degrees(-90.0);
/// assert_eq!(angle * 2 / 2, Angle::from_degrees(-90.0));
///
/// let big = Angle::from_degrees(270.0);
/// assert_eq!(angle, big);
/// assert_eq!(angle / 2, Angle::from_degrees(-45.0));
/// ```
///
/// ## Precision
/// This operation performs integer division, which involves rounding, and hence is a lossy
/// operation.
/// For example, `wrapang::HALF / 3 * 3` yields a value slightly smaller from `warpang::HALF`,
/// differing by 0.00000149%.
/// This difference is negligible alone, but it gets significant if accumulated.
/// Moreover, a single difference can still affect the result of `PartialEq`, so compare
/// approximated angles the same way floats are dealed with.
///
/// wrapang offers uniform-precision angle values, but this still does not imply perfect precision.
impl ops::Div<i32> for Angle {
    type Output = Angle;

    #[inline] // Inlining allows const arithmetic optimization
    fn div(self, other: i32) -> Angle {
        Angle(Wrapping((self.as_u32() as i32 / other) as u32))
    }
}

/// An angle of zero size.
pub const ZERO: Angle = Angle(Wrapping(0));
/// An angle approximating &pi;/6 radians (30 degrees).
pub const TWELVTH: Angle = Angle(Wrapping(0x_4000_0000 / 3));
/// An angle of &pi;/4 radians (45 degrees).
pub const EIGHTH: Angle = Angle(Wrapping(0x_2000_0000 / 3));
/// An angle approximating &pi;/3 radians (60 degrees).
pub const SIXTH: Angle = Angle(Wrapping(0x_8000_0000 / 3));
/// An angle of &pi;/2 radians (90 degrees).
pub const QUARTER: Angle = Angle(Wrapping(0x_4000_0000));
/// An angle approximating 2&pi;/3 radians (120 degrees).
pub const THIRD: Angle = Angle(Wrapping((0x_1_0000_0000u64 / 3u64) as u32));
/// An angle of &pi; radians (180 degress).
pub const HALF: Angle = Angle(Wrapping(0x_8000_0000));
/// An angle of 3&pi;/2 radians (270 degress).
pub const COUN: Angle = Angle(Wrapping(0x_c000_0000));
