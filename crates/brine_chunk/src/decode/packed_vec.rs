//! Implementation of packed vectors for decoding packed block data.

use std::fmt;

/// A packed vector of unsigned integers with a fixed number of bits per entry.
///
/// The data is stored as a vector of u64 words. If N is the number of bits per
/// entry, then the first entry is stored in the least significant N bits of the
/// first word in that vector, and the next entry is stored in the next N least
/// significant bits, and so on, advancing into subsequent words as words become
/// filled with entries.
///
/// A single entry may span between multiple words if N is not a divisor of 64.
/// In that case, the bits that cannot fit into the word are stored in the least
/// significant bits of the next word. See the example below.
///
/// # Example
///
/// If the bits per entry is 10, and the first two words are
/// `[0x01001880C0060020, 0x0200D0068004C020]`, then the first 12 entries are
/// `[32, 384, 0, 515, 24, 64, 512, 768, 4, 416, 256, 3]`:
///
/// ```txt
///                  [6]: 512  [5]: 64    [4]: 24    [3]: 515   [2]: 0     [1]: 384   [0]: 32
///                       ____ __________ __________ __________ __________ __________ __________
/// 0x01001880C0060020: / 0000 0001000000 0000011000 1000000011 0000000000 0110000000 0000100000
///                    |
///                     \ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _
///                                                                                             \
///                                [11]: 3    [10]: 256  [9]: 416   [8]: 4     [7]: 768          |
///                                __________ __________ __________ __________ __________ ______/
/// 0x0200D0068004C020:   00000010 0000000011 0100000000 0110100000 0000000100 1100000000 100000
/// ```
///
/// ```rust
/// use brine_chunk::packed_vec::PackedIntVec;
///
/// let words = vec![0x01001880C0060020, 0x0200D0068004C020];
/// let length = 12;
/// let bits_per_entry = 10;
///
/// let vec = PackedIntVec::from_parts(words, length, bits_per_entry).unwrap();
///
/// let entries: Vec<u32> = vec.iter().collect();
/// assert_eq!(entries, [32, 384, 0, 515, 24, 64, 512, 768, 4, 416, 256, 3]);
/// ```
#[derive(Clone)]
pub struct PackedIntVec {
    words: Vec<u64>,
    length: usize,
    bits_per_entry: u8,
}

impl PackedIntVec {
    /// Initializes a packed vector from a list of u64 words, a length, and the
    /// number of bits per entry.
    ///
    /// Returns `None` if `length` and/or `bits_per_entry` are invalid.
    #[inline]
    pub fn from_parts(
        words: impl IntoIterator<Item = u64>,
        length: usize,
        bits_per_entry: u8,
    ) -> Option<Self> {
        let words: Vec<_> = words.into_iter().collect();

        if bits_per_entry == 0 || bits_per_entry > 32 {
            return None;
        }
        if length * bits_per_entry as usize > words.len() * 64 {
            return None;
        }

        Some(Self {
            words,
            length,
            bits_per_entry,
        })
    }

    /// Returns the packed word vector along with the current length and the
    /// number of bits per entry.
    #[inline]
    pub fn into_parts(self) -> (Vec<u64>, usize, u8) {
        let Self {
            words,
            length,
            bits_per_entry,
        } = self;
        (words, length, bits_per_entry)
    }

    /// Returns the number of entries stored in the packed vector.
    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    /// Iterates through the entries of the packed vector.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            vec: self,
            index: 0,
        }
    }

    /// Iterates through the raw words in the packed vector.
    #[inline]
    pub fn words(&self) -> impl Iterator<Item = u64> + '_ {
        self.words.iter().copied()
    }

    /// Returns the entry at the given index or `None` if out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<u32> {
        if index >= self.length {
            return None;
        }

        Some(self.unpack_integer_at(self.entry_index_to_bit_index(index)))
    }

    /* TODO
    /// Updates the entry at the given index.
    ///
    /// Value will be truncated to fit into the number of bits per entry.
    ///
    /// Returns the previous value, or `None` if index is out of bounds.
    #[inline]
    pub fn set(&mut self, index: usize, value: u32) -> Option<u32> {
        if index >= self.length {
            return None;
        }

        let index = self.entry_index_to_bit_index(index);
        let prev = self.unpack_integer_at(index);
        self.pack_integer_at(index, value);

        Some(prev)
    }
    */

    #[inline]
    fn entry_index_to_bit_index(&self, index: usize) -> BitIndex {
        let bit_index = index * self.bits_per_entry as usize;

        let word_index = bit_index / 64;
        let bit_offset = bit_index % 64;

        BitIndex {
            word_index,
            bit_offset: bit_offset as u8,
        }
    }

    #[inline]
    fn unpack_integer_at(&self, bit_index: BitIndex) -> u32 {
        // For example, consider bits_per_entry=10, and two example bit indexes:
        // A) bit_index={0, 48}
        // B) bit_index={0, 60}

        let word = self.words[bit_index.word_index];

        // *) 0b0000000000000000_0000000000000000_0000000000000000_0000001111111111
        //                                                               ^^^^^^^^^^
        let bitmask = u64::MAX >> (64 - self.bits_per_entry);

        // A) 0b0000000000000000_0000000000000000_0000000000000000_1101001001110001
        //                                                         ^^^^^^^^^^^^^^^^
        // B) 0b0000000000000000_0000000000000000_0000000000000000_0000000000001101
        //                                                                     ^^^^
        let word_shifted = word >> bit_index.bit_offset;

        // A) 0b0000000000000000_0000000000000000_0000000000000000_0000001001110001
        //                                                               ^^^^^^^^^^
        // B) 0b0000000000000000_0000000000000000_0000000000000000_0000000000001101
        //                                                               ^^^^^^^^^^
        let word_masked = word_shifted & bitmask;

        // Return here if the entry does not spill over into the next word.
        if bit_index.bit_offset + self.bits_per_entry <= 64 {
            // A) No spillover
            return word_masked as u32;
        }

        // B) Bits gotten so far = 4
        let bits_gotten = 64 - bit_index.bit_offset;

        // B) 0b0000000000000000_0000000000000000_0000000000000000_0000000000111111
        //                                                                   ^^^^^^
        let remaining_bitmask = bitmask >> bits_gotten;

        // Need to get that many least significant bits from the next word.
        let next_word = self.words[bit_index.word_index + 1];

        // B) 0b0000000000000000_0000000000000000_0000000000000000_0000000000100101
        //                                                                   ^^^^^^
        let next_word_masked = next_word & remaining_bitmask;

        // B) 0b0000000000000000_0000000000000000_0000000000000000_0000001001010000
        //                                                               ^^^^^^
        let next_word_masked_and_shifted = next_word_masked << bits_gotten;

        // B) 0b0000000000000000_0000000000000000_0000000000000000_0000001001011101
        //                                                               ^^^^^^^^^^
        (next_word_masked_and_shifted | word_masked) as u32
    }
}

impl PartialEq for PackedIntVec {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl Eq for PackedIntVec {}

impl fmt::Debug for PackedIntVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Packed")
            .field(&self.iter().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct BitIndex {
    word_index: usize,
    bit_offset: u8,
}

/// [`PackedIntVec`] iterator.
pub struct Iter<'a> {
    vec: &'a PackedIntVec,
    index: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.vec.get(self.index);

        if next.is_some() {
            self.index += 1;
        }

        next
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_vec_eq(vec: &PackedIntVec, expected: impl AsRef<[u32]>) {
        let entries: Vec<u32> = vec.iter().collect();
        assert_eq!(&entries[..], expected.as_ref());
    }

    #[test]
    fn invalid_construction() {
        let words = vec![0xFEDCBA9876543210];

        assert_eq!(PackedIntVec::from_parts(words.clone(), 0, 0), None);
        assert_eq!(PackedIntVec::from_parts(words.clone(), 1, 0), None);
        assert_eq!(PackedIntVec::from_parts(words.clone(), 1, 33), None);
        assert_eq!(PackedIntVec::from_parts(words, 5, 13), None);
    }

    #[test]
    fn even_divisors() {
        let words = vec![0xFEDCBA9876543210];

        let vec = PackedIntVec::from_parts(words.clone(), 2, 32).unwrap();
        assert_vec_eq(&vec, [0x76543210, 0xFEDCBA98]);

        let vec = PackedIntVec::from_parts(words.clone(), 4, 16).unwrap();
        assert_vec_eq(&vec, [0x3210, 0x7654, 0xBA98, 0xFEDC]);

        let vec = PackedIntVec::from_parts(words.clone(), 8, 8).unwrap();
        assert_vec_eq(&vec, [0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE]);

        let vec = PackedIntVec::from_parts(words, 16, 4).unwrap();
        assert_vec_eq(
            &vec,
            [
                0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF,
            ],
        );

        let words =
            vec![0b1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010_1010];

        let vec = PackedIntVec::from_parts(words.clone(), 32, 2).unwrap();
        assert_vec_eq(
            &vec,
            [
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2,
            ],
        );

        let vec = PackedIntVec::from_parts(words, 64, 1).unwrap();
        assert_vec_eq(
            &vec,
            [
                0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
                0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
                0, 1, 0, 1, 0, 1, 0, 1,
            ],
        )
    }

    #[test]
    fn test_equality_with_different_bits_outside_of_range() {
        let vec1 = PackedIntVec::from_parts(vec![0xFFF0000000000000], 2, 24).unwrap();
        let vec2 = PackedIntVec::from_parts(vec![0x0000000000000000], 2, 24).unwrap();
        assert_eq!(vec1, vec2);
    }
}
