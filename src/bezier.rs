// https://pomax.github.io/bezierinfo

use num_traits::Float;
use core::ops::*;
use core::iter::Sum;
use vec::repr_c::{
    Vec3 as CVec3,
    Vec4 as CVec4,
};

// WISH: into_iter, iter_mut, etc (for concisely applying the same xform to all points)
// WISH: AABBs from beziers
// WISH: OOBBs from beziers
// WISH: "Tracing a curve at fixed distance intervals"
// WISH: project a point on a curve using e.g binary search after a coarse linear search

macro_rules! bezier_impl_any {
    ($Bezier:ident $Point:ident) => {
        impl<T> $Bezier<T> {
            /// Evaluates the normalized tangent at interpolation factor `t`.
            pub fn normalized_tangent(self, t: T) -> $Point<T> where T: Float + Sum {
                self.evaluate_derivative(t).normalized()
            }
            // WISH: better length approximation estimations (e.g see https://math.stackexchange.com/a/61796)
            /// Approximates the curve's length by subdividing it into step_count+1 segments.
            pub fn length_by_discretization(self, step_count: u32) -> T
                where T: Float + AddAssign + Sum
            {
	            let mut length = T::zero();
	            let mut prev_point = self.evaluate(T::zero());
                for i in 1..step_count+2 {
    		        let t = T::from(i).unwrap()/(T::from(step_count).unwrap()+T::one());
    		        let next_point = self.evaluate(t);
                    length += (next_point - prev_point).magnitude();
    		        prev_point = next_point;
                }
	            length
            }

        }
    }
}

macro_rules! bezier_impl_quadratic {
    ($(#[$attrs:meta])* $QuadraticBezier:ident $Point:ident $LineSegment:ident) => {
        
        $(#[$attrs])*
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, /*PartialOrd, Ord*/)]
		#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
        pub struct $QuadraticBezier<T>(pub $Point<T>, pub $Point<T>, pub $Point<T>);
        
        impl<T: Float> $QuadraticBezier<T> {
            /// Evaluates the position of the point lying on the curve at interpolation factor `t`.
            ///
            /// This is one of the most important Bézier curve operations,
            /// because, in one way or another, it is used to render a curve
            /// to the screen.
            /// The common use case is to successively evaluate a curve at a set of values
            /// that range from 0 to 1, to approximate the curve as an array of
            /// line segments which are then rendered.
            pub fn evaluate(self, t: T) -> $Point<T> {
                let l = T::one();
                let two = l+l;
                self.0*(l-t)*(l-t) + self.1*two*(l-t)*t + self.2*t*t
            }
            /// Evaluates the derivative tangent at interpolation factor `t`, which happens to give
            /// a non-normalized tangent vector.
            ///
            /// See also `normalized_tangent()`.
            pub fn evaluate_derivative(self, t: T) -> $Point<T> {
                let l = T::one();
                let n = l+l;
                (self.1-self.0)*(l-t)*n + (self.2-self.1)*t*n
            }
            /// Creates a quadratic Bézier curve from a single segment.
            pub fn from_line_segment(line: $LineSegment<T>) -> Self {
                $QuadraticBezier(line.a, line.a, line.b)
            }
            /// Returns the constant matrix M such that,
            /// given `T = [1, t*t, t*t*t]` and `P` the vector of control points,
            /// `dot(T * M, P)` evalutes the Bezier curve at 't'.
            ///
            /// This function name is arguably dubious.
	        pub fn matrix() -> Mat3<T> {
                let zero = T::zero();
                let one = T::one();
                let two = one+one;
                Mat3 {
                    rows: CVec3::new(
                        Vec3::new( one,  zero, zero),
                        Vec3::new(-two,  two, zero),
                        Vec3::new( one, -two, one),
                    )
                }
            }
            /// Splits this quadratic Bézier curve into two curves, at interpolation factor `t`.
            // NOTE that some computations may be reused, but the compiler can
            // reason about these. Clarity wins here IMO.
            pub fn split(self, t: T) -> (Self, Self) {
                let l = T::one();
                let two = l+l;
                let first = $QuadraticBezier(
                    self.0,
                    self.1*t - self.0*(t-l),
                    self.2*t*t - self.1*two*t*(t-l) + self.0*(t-l)*(t-l),
                );
                let second = $QuadraticBezier(
                    self.2*t*t - self.1*two*t*(t-l) + self.0*(t-l)*(t-l),
                    self.2*t - self.1*(t-l),
                    self.2,
                );
                (first, second)
            }
        }
        
        impl<T> From<Vec3<$Point<T>>> for $QuadraticBezier<T> {
            fn from(v: Vec3<$Point<T>>) -> Self {
                $QuadraticBezier(v.x, v.y, v.z)
            }
        }
        impl<T> From<$QuadraticBezier<T>> for Vec3<$Point<T>> {
            fn from(v: $QuadraticBezier<T>) -> Self {
                Vec3::new(v.0, v.1, v.2)
            }
        }
        
        bezier_impl_any!($QuadraticBezier $Point);
    }
}

macro_rules! bezier_impl_cubic {
    ($(#[$attrs:meta])* $CubicBezier:ident $Point:ident $LineSegment:ident) => {
        
        $(#[$attrs])*
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, /*PartialOrd, Ord*/)]
		#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
        pub struct $CubicBezier<T>(pub $Point<T>, pub $Point<T>, pub $Point<T>, pub $Point<T>);

        impl<T: Float> $CubicBezier<T> {
            /// Evaluates the position of the point lying on the curve at interpolation factor `t`.
            ///
            /// This is one of the most important Bézier curve operations,
            /// because, in one way or another, it is used to render a curve
            /// to the screen.
            /// The common use case is to successively evaluate a curve at a set of values
            /// that range from 0 to 1, to approximate the curve as an array of
            /// line segments which are then rendered.
            pub fn evaluate(self, t: T) -> $Point<T> {
                let l = T::one();
                let three = l+l+l;
		        self.0*(l-t)*(l-t)*(l-t) + self.1*three*(l-t)*(l-t)*t + self.2*three*(l-t)*t*t + self.3*t*t*t
            }
            /// Evaluates the derivative tangent at interpolation factor `t`, which happens to give
            /// a non-normalized tangent vector.
            ///
            /// See also `normalized_tangent()`.
            pub fn evaluate_derivative(self, t: T) -> $Point<T> {
                let l = T::one();
        	    let n = l+l+l;
                let two = l+l;
        		(self.1-self.0)*(l-t)*(l-t)*n + (self.2-self.1)*two*(l-t)*t*n + (self.3-self.2)*t*t*n
        	}
            /// Creates a cubic Bézier curve from a single segment.
            pub fn from_line_segment(line: $LineSegment<T>) -> Self {
                $CubicBezier(line.a, line.a, line.b, line.b)
            }
            /// Returns the constant matrix M such that,
            /// given `T = [1, t*t, t*t*t, t*t*t*t]` and `P` the vector of control points,
            /// `dot(T * M, P)` evalutes the Bezier curve at 't'.
            ///
            /// This function name is arguably dubious.
	        pub fn matrix() -> Mat4<T> {
                let zero = T::zero();
                let one = T::one();
                let three = one+one+one;
                let six = three + three;
                Mat4 {
                    rows: CVec4::new(
                        Vec4::new( one,  zero,  zero, zero),
                        Vec4::new(-three,  three,  zero, zero),
                        Vec4::new( three, -six,  three, zero),
                        Vec4::new(-one,  three, -three, one),
                    )
                }
            }
            /// Splits this cubic Bézier curve into two curves, at interpolation factor `t`.
            // NOTE that some computations may be reused, but the compiler can
            // reason about these. Clarity wins here IMO.
            pub fn split(self, t: T) -> (Self, Self) {
                let l = T::one();
                let two = l+l;
                let three = l+l+l;
                let first = $CubicBezier(
                    self.0,
                    self.1*t - self.0*(t-l),
                    self.2*t*t - self.1*two*t*(t-l) + self.0*(t-l)*(t-l),
                    self.3*t*t*t - self.2*three*t*t*(t-l) + self.1*three*t*(t-l)*(t-l) - self.0*(t-l)*(t-l)*(t-l),
                );
                let second = $CubicBezier(
                    self.3*t*t*t - self.2*three*t*t*(t-l) + self.1*three*t*(t-l)*(t-l) - self.0*(t-l)*(t-l)*(t-l),
                    self.3*t*t - self.2*two*t*(t-l) + self.1*(t-l)*(t-l),
                    self.3*t - self.2*(t-l),
                    self.3,
                );
                (first, second)
            }
            // WISH: CubicBezier::circle(radius)
            // pub fn circle(radius: T, curve_count: u32) ->
        }
        
        impl<T> From<Vec4<$Point<T>>> for $CubicBezier<T> {
            fn from(v: Vec4<$Point<T>>) -> Self {
                $CubicBezier(v.x, v.y, v.z, v.w)
            }
        }
        impl<T> From<$CubicBezier<T>> for Vec4<$Point<T>> {
            fn from(v: $CubicBezier<T>) -> Self {
                Vec4::new(v.0, v.1, v.2, v.3)
            }
        }
        
        bezier_impl_any!($CubicBezier $Point);
    }
}

macro_rules! impl_all_beziers {
    () => {
        bezier_impl_quadratic!{
            /// A 2D curve with one control point.
            QuadraticBezier2 Vec2 LineSegment2
        }
        bezier_impl_quadratic!{
            /// A 3D curve with one control point.
            QuadraticBezier3 Vec3 LineSegment3
        }
        bezier_impl_cubic!{
            /// A 2D curve with two control points.
            CubicBezier2 Vec2 LineSegment2
        }
        bezier_impl_cubic!{
            /// A 3D curve with two control points.
            CubicBezier3 Vec3 LineSegment3
        }
    };
}

#[cfg(all(nightly, feature="repr_simd"))]
pub mod repr_simd {
    use super::*;
    use vec::repr_simd::{Vec3, Vec4, Vec2};
    use mat::repr_simd::row_major::{Mat3, Mat4};
    use geom::repr_simd::{LineSegment2, LineSegment3};
    impl_all_beziers!{}
}
pub mod repr_c {
    use super::*;
    use  vec::repr_c::{Vec3, Vec4, Vec2};
    use  mat::repr_c::row_major::{Mat3, Mat4};
    use geom::repr_c::{LineSegment2, LineSegment3};
    impl_all_beziers!{}
}

pub use self::repr_c::*;