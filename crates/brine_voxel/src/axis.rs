use crate::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    /// Returns the possible values of this enum as an array.
    #[inline]
    pub const fn values() -> [Self; 3] {
        [Self::X, Self::Y, Self::Z]
    }

    /// Turns this axis into a [`Direction`] using the provided sign.
    ///
    /// # Example
    ///
    /// ```
    /// # use brine_voxel::*;
    /// assert_eq!(Axis::X.with_sign(AxisSign::Pos), Direction::XPos);
    /// ```
    #[inline]
    pub const fn with_sign(self, sign: AxisSign) -> Direction {
        Direction::from_axis_and_sign(self, sign)
    }

    /// Returns the axis that is orthogonal to `self` and `other`.
    ///
    /// # Example
    ///
    /// ```
    /// # use brine_voxel::*;
    /// assert_eq!(Axis::X.orthogonal_with(Axis::Y), Axis::Z);
    /// assert_eq!(Axis::X.orthogonal_with(Axis::Z), Axis::Y);
    /// ```
    ///
    /// # Panics
    ///
    /// If `self` is equal to `other`.
    #[inline]
    pub const fn orthogonal_with(self, other: Axis) -> Axis {
        match (self, other) {
            (Axis::X, Axis::Y) | (Axis::Y, Axis::X) => Axis::Z,
            (Axis::X, Axis::Z) | (Axis::Z, Axis::X) => Axis::Y,
            (Axis::Y, Axis::Z) | (Axis::Z, Axis::Y) => Axis::X,
            _ => panic!("no orthogonal axis for identical axes"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AxisSign {
    Neg,
    Pos,
}

impl AxisSign {
    /// Returns the possible values of this enum as an array.
    #[inline]
    pub const fn values() -> [Self; 2] {
        [Self::Neg, Self::Pos]
    }

    /// Returns the opposite sign.
    ///
    /// # Example
    ///
    /// ```
    /// # use brine_voxel::*;
    /// assert_eq!(AxisSign::Pos.opposite(), AxisSign::Neg);
    /// ```
    #[inline]
    pub const fn opposite(self) -> AxisSign {
        match self {
            AxisSign::Neg => AxisSign::Pos,
            AxisSign::Pos => AxisSign::Neg,
        }
    }
}
