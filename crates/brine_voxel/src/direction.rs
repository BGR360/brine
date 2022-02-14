use num_traits::{CheckedAdd, CheckedSub};

use crate::{Axis, AxisSign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    XNeg,
    XPos,
    YNeg,
    YPos,
    ZNeg,
    ZPos,
}

impl Direction {
    /// Returns the possible values of this enum as an array.
    #[inline]
    pub const fn values() -> [Self; 6] {
        [
            Self::XNeg,
            Self::XPos,
            Self::YNeg,
            Self::YPos,
            Self::ZNeg,
            Self::ZPos,
        ]
    }

    /// Returns the direction for the given axis and sign.
    #[inline]
    pub const fn from_axis_and_sign(axis: Axis, sign: AxisSign) -> Direction {
        match (axis, sign) {
            (Axis::X, AxisSign::Neg) => Direction::XNeg,
            (Axis::X, AxisSign::Pos) => Direction::XPos,
            (Axis::Y, AxisSign::Neg) => Direction::YNeg,
            (Axis::Y, AxisSign::Pos) => Direction::YPos,
            (Axis::Z, AxisSign::Neg) => Direction::ZNeg,
            (Axis::Z, AxisSign::Pos) => Direction::ZPos,
        }
    }

    /// Returns the axis of the direction.
    #[inline]
    pub const fn axis(self) -> Axis {
        match self {
            Direction::XNeg => Axis::X,
            Direction::XPos => Axis::X,
            Direction::YNeg => Axis::Y,
            Direction::YPos => Axis::Y,
            Direction::ZNeg => Axis::Z,
            Direction::ZPos => Axis::Z,
        }
    }

    /// Returns the sign of the direction.
    #[inline]
    pub const fn sign(self) -> AxisSign {
        match self {
            Direction::XNeg => AxisSign::Neg,
            Direction::XPos => AxisSign::Pos,
            Direction::YNeg => AxisSign::Neg,
            Direction::YPos => AxisSign::Pos,
            Direction::ZNeg => AxisSign::Neg,
            Direction::ZPos => AxisSign::Pos,
        }
    }

    /// Returns the direction with the same axis and opposite sign.
    ///
    /// # Example
    ///
    /// ```
    /// # use brine_voxel::*;
    /// assert_eq!(Direction::XPos.opposite(), Direction::XNeg);
    /// ```
    #[inline]
    pub const fn opposite(self) -> Self {
        Self::from_axis_and_sign(self.axis(), self.sign().opposite())
    }

    /// Returns the direction that is orthogonal to `self` and `other` with sign
    /// `sign`.
    ///
    /// # Example
    ///
    /// ```
    /// # use brine_voxel::*;
    /// assert_eq!(
    ///     Direction::XPos.orthogonal_with(Direction::YNeg, AxisSign::Pos),
    ///     Direction::ZPos
    /// );
    /// assert_eq!(
    ///     Direction::XPos.orthogonal_with(Direction::YNeg, AxisSign::Neg),
    ///     Direction::ZNeg
    /// );
    /// ```
    ///
    /// # Panics
    ///
    /// If `self` and `other` are on the same axis.
    #[inline]
    pub const fn orthogonal_with(self, other: Direction, sign: AxisSign) -> Direction {
        Self::from_axis_and_sign(self.axis().orthogonal_with(other.axis()), sign)
    }

    /// Returns the direction that results from rotating `self` about `axis` by
    /// `degrees` degrees.
    ///
    /// # Example
    ///
    /// ```
    /// # use brine_voxel::*;
    /// assert_eq!(Direction::XNeg.with_rotation(Axis::Z, 90), Direction::YPos);
    /// assert_eq!(Direction::YPos.with_rotation(Axis::Z, -90), Direction::XNeg);
    /// ```
    ///
    /// # Panics
    ///
    /// If `degrees` is not a whole multiple of 90.
    #[inline]
    pub const fn with_rotation(self, axis: Axis, degrees: i32) -> Self {
        match (self.axis(), axis) {
            (Axis::X, Axis::X) | (Axis::Y, Axis::Y) | (Axis::Z, Axis::Z) => return self,

            _ => {}
        }

        let rotations = match Self::quarter_rotations(degrees) {
            i if i < 0 => (i + 4).abs() % 4,
            i => i % 4,
        } as usize;

        let new_axis = match rotations % 2 {
            0 => self.axis(),
            1 => self.axis().orthogonal_with(axis),
            _ => unreachable!(),
        };

        let new_sign = match (self.axis(), axis) {
            (Axis::X, Axis::Y) => {
                [AxisSign::Pos, AxisSign::Pos, AxisSign::Neg, AxisSign::Neg][rotations]
            }
            (Axis::X, Axis::Z) => {
                [AxisSign::Pos, AxisSign::Neg, AxisSign::Neg, AxisSign::Pos][rotations]
            }
            (Axis::Y, Axis::Z) => {
                [AxisSign::Pos, AxisSign::Pos, AxisSign::Neg, AxisSign::Neg][rotations]
            }
            (Axis::Y, Axis::X) => {
                [AxisSign::Pos, AxisSign::Neg, AxisSign::Neg, AxisSign::Pos][rotations]
            }
            (Axis::Z, Axis::X) => {
                [AxisSign::Pos, AxisSign::Pos, AxisSign::Neg, AxisSign::Neg][rotations]
            }
            (Axis::Z, Axis::Y) => {
                [AxisSign::Pos, AxisSign::Neg, AxisSign::Neg, AxisSign::Pos][rotations]
            }
            _ => unreachable!(),
        };

        let new_sign = match self.sign() {
            AxisSign::Pos => new_sign,
            AxisSign::Neg => new_sign.opposite(),
        };

        Self::from_axis_and_sign(new_axis, new_sign)
    }

    #[inline]
    pub const fn with_rotation_x(self, degrees: i32) -> Self {
        self.with_rotation(Axis::X, degrees)
    }

    #[inline]
    pub const fn with_rotation_y(self, degrees: i32) -> Self {
        self.with_rotation(Axis::Y, degrees)
    }

    #[inline]
    pub const fn with_rotation_z(self, degrees: i32) -> Self {
        self.with_rotation(Axis::Z, degrees)
    }

    #[inline]
    pub const fn with_rotation_xyz(self, degrees_x: i32, degrees_y: i32, degrees_z: i32) -> Self {
        self.with_rotation_x(degrees_x)
            .with_rotation_y(degrees_y)
            .with_rotation_z(degrees_z)
    }

    #[inline]
    const fn quarter_rotations(degrees: i32) -> i32 {
        let rotations = degrees / 90;
        let remainder = degrees % 90;
        if remainder != 0 {
            panic!("input must be a whole multiple of 90");
        }
        rotations
    }

    /// Translates the given `[x, y, z]` position by `distance` units in this direction.
    ///
    /// Returns `None` if the addition or subtractions would over/underflow.
    ///
    /// # Example
    ///
    /// ```
    /// # use brine_voxel::*;
    /// let pos: [u8; 3] = [0, 0, 0];
    ///
    /// assert_eq!(Direction::XNeg.translate_pos(pos, 1), None);
    /// assert_eq!(Direction::XPos.translate_pos(pos, 1), Some([1, 0, 0]));
    /// assert_eq!(Direction::YNeg.translate_pos(pos, 1), None);
    /// assert_eq!(Direction::YPos.translate_pos(pos, 1), Some([0, 1, 0]));
    /// assert_eq!(Direction::ZNeg.translate_pos(pos, 1), None);
    /// assert_eq!(Direction::ZPos.translate_pos(pos, 1), Some([0, 0, 1]));
    /// ```
    #[inline]
    pub fn translate_pos<T>(&self, [x, y, z]: [T; 3], distance: T) -> Option<[T; 3]>
    where
        T: Copy + CheckedAdd<Output = T> + CheckedSub<Output = T>,
    {
        match self {
            Direction::XNeg => x.checked_sub(&distance).map(|x| [x, y, z]),
            Direction::XPos => x.checked_add(&distance).map(|x| [x, y, z]),
            Direction::YNeg => y.checked_sub(&distance).map(|y| [x, y, z]),
            Direction::YPos => y.checked_add(&distance).map(|y| [x, y, z]),
            Direction::ZNeg => z.checked_sub(&distance).map(|z| [x, y, z]),
            Direction::ZPos => z.checked_add(&distance).map(|z| [x, y, z]),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_rotations(direction: Direction, axis: Axis, expected: [Direction; 4]) {
        assert_eq!(direction.with_rotation(axis, -360), expected[0]);
        assert_eq!(direction.with_rotation(axis, -270), expected[1]);
        assert_eq!(direction.with_rotation(axis, -180), expected[2]);
        assert_eq!(direction.with_rotation(axis, -90), expected[3]);
        assert_eq!(direction.with_rotation(axis, 0), expected[0]);
        assert_eq!(direction.with_rotation(axis, 90), expected[1]);
        assert_eq!(direction.with_rotation(axis, 180), expected[2]);
        assert_eq!(direction.with_rotation(axis, 270), expected[3]);
        assert_eq!(direction.with_rotation(axis, 360), expected[0]);
    }

    #[test]
    fn test_x_rotations() {
        assert_rotations(
            Direction::XNeg,
            Axis::X,
            [
                Direction::XNeg,
                Direction::XNeg,
                Direction::XNeg,
                Direction::XNeg,
            ],
        );

        assert_rotations(
            Direction::XPos,
            Axis::X,
            [
                Direction::XPos,
                Direction::XPos,
                Direction::XPos,
                Direction::XPos,
            ],
        );

        assert_rotations(
            Direction::XNeg,
            Axis::Y,
            [
                Direction::XNeg,
                Direction::ZNeg,
                Direction::XPos,
                Direction::ZPos,
            ],
        );

        assert_rotations(
            Direction::XPos,
            Axis::Y,
            [
                Direction::XPos,
                Direction::ZPos,
                Direction::XNeg,
                Direction::ZNeg,
            ],
        );

        assert_rotations(
            Direction::XNeg,
            Axis::Z,
            [
                Direction::XNeg,
                Direction::YPos,
                Direction::XPos,
                Direction::YNeg,
            ],
        );

        assert_rotations(
            Direction::XPos,
            Axis::Z,
            [
                Direction::XPos,
                Direction::YNeg,
                Direction::XNeg,
                Direction::YPos,
            ],
        );
    }

    #[test]
    fn test_y_rotations() {
        assert_rotations(
            Direction::YNeg,
            Axis::Y,
            [
                Direction::YNeg,
                Direction::YNeg,
                Direction::YNeg,
                Direction::YNeg,
            ],
        );

        assert_rotations(
            Direction::YPos,
            Axis::Y,
            [
                Direction::YPos,
                Direction::YPos,
                Direction::YPos,
                Direction::YPos,
            ],
        );

        assert_rotations(
            Direction::YNeg,
            Axis::X,
            [
                Direction::YNeg,
                Direction::ZPos,
                Direction::YPos,
                Direction::ZNeg,
            ],
        );

        assert_rotations(
            Direction::YPos,
            Axis::X,
            [
                Direction::YPos,
                Direction::ZNeg,
                Direction::YNeg,
                Direction::ZPos,
            ],
        );

        assert_rotations(
            Direction::YNeg,
            Axis::Z,
            [
                Direction::YNeg,
                Direction::XNeg,
                Direction::YPos,
                Direction::XPos,
            ],
        );

        assert_rotations(
            Direction::YPos,
            Axis::Z,
            [
                Direction::YPos,
                Direction::XPos,
                Direction::YNeg,
                Direction::XNeg,
            ],
        );
    }

    #[test]
    fn test_z_rotations() {
        assert_rotations(
            Direction::ZNeg,
            Axis::Z,
            [
                Direction::ZNeg,
                Direction::ZNeg,
                Direction::ZNeg,
                Direction::ZNeg,
            ],
        );

        assert_rotations(
            Direction::ZPos,
            Axis::Z,
            [
                Direction::ZPos,
                Direction::ZPos,
                Direction::ZPos,
                Direction::ZPos,
            ],
        );

        assert_rotations(
            Direction::ZNeg,
            Axis::X,
            [
                Direction::ZNeg,
                Direction::YNeg,
                Direction::ZPos,
                Direction::YPos,
            ],
        );

        assert_rotations(
            Direction::ZPos,
            Axis::X,
            [
                Direction::ZPos,
                Direction::YPos,
                Direction::ZNeg,
                Direction::YNeg,
            ],
        );

        assert_rotations(
            Direction::ZNeg,
            Axis::Y,
            [
                Direction::ZNeg,
                Direction::XPos,
                Direction::ZPos,
                Direction::XNeg,
            ],
        );

        assert_rotations(
            Direction::ZPos,
            Axis::Y,
            [
                Direction::ZPos,
                Direction::XNeg,
                Direction::ZNeg,
                Direction::XPos,
            ],
        );
    }
}
