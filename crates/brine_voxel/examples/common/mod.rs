use std::fmt;

pub const CHUNK_SIDE: u8 = 4;
pub const CHUNK_SIDE_USIZE: usize = CHUNK_SIDE as usize;
pub const CHUNK_VOXELS: usize = CHUNK_SIDE_USIZE * CHUNK_SIDE_USIZE * CHUNK_SIDE_USIZE;

mod mesh_viewer;

pub use mesh_viewer::MeshViewerPlugin;

pub struct IntChunk([u32; CHUNK_VOXELS]);

impl IntChunk {
    pub fn all(value: u32) -> Self {
        Self([value; CHUNK_VOXELS])
    }

    pub fn random(max: u32) -> Self {
        let mut ints = [0; CHUNK_VOXELS];
        for i in ints.iter_mut() {
            *i = fastrand::u32(..max + 1);
        }
        Self(ints)
    }

    #[inline(always)]
    pub fn get(&self, x: u8, y: u8, z: u8) -> Option<u32> {
        if x >= CHUNK_SIDE || y >= CHUNK_SIDE || z >= CHUNK_SIDE {
            None
        } else {
            let index: usize = ((x as usize) * CHUNK_SIDE_USIZE * CHUNK_SIDE_USIZE)
                + ((y as usize) * CHUNK_SIDE_USIZE)
                + (z as usize);

            Some(self.0[index])
        }
    }
}

/*
      0   1   0   1
    0   0   1   1
  0   0   0   0
1   0   0   1

      1   1   1   1
    1   0   1   0
  0   0   1   1
0   1   0   0

    +y
    |
    |
    |_______ +x
   /
  /
+z
 */
impl fmt::Display for IntChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const INDENT: u8 = 2;
        const SPACING: u8 = 3;

        let value_width = (*self.0.iter().max().unwrap() / 10 + 1) as u8;
        let spacing = SPACING - value_width + 1;

        for y in (0..CHUNK_SIDE).rev() {
            for z in 0..CHUNK_SIDE {
                let num_indents = CHUNK_SIDE - z;
                for _ in 0..(num_indents * INDENT) {
                    write!(f, " ")?;
                }

                for x in 0..CHUNK_SIDE {
                    write!(f, "{}", self.get(x, y, z).unwrap())?;

                    if x != CHUNK_SIDE - 1 {
                        for _ in 0..spacing {
                            write!(f, " ")?;
                        }
                    }
                }
                writeln!(f)?;
            }
            writeln!(f)?;
        }

        write!(
            f,
            r#"
    +y
    |
    |
    |_______ +x
   /
  /
+z
            "#
        )
    }
}
