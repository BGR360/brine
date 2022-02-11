# brine_voxel

A library for Minecraft-like voxel games.

## Features and Capabilities

### Mesh Generation

#### Arbitrary chunk shapes

Geometry can be generated for cubic and non-cubic voxel regions. This means that
the library can be used to, for instance, generate meshes for inventory items
using their textures.

#### Complex voxel geometry

The term "voxel" may give the impression that this library is suited only
for worlds that consist entirely of 1x1 cubes. But in fact, this library
supports worlds with arbitrary[^1] geometry in each voxel. This makes it
suitable for Minecraft worlds, where not all blocks are simple 1x1 cubes.

So, when using this library and reading its documentation, you should
consider the definition of "voxel" to be a cubic **location** in a
3-dimensional blocky world.

[^1]: Voxels can't have *truly* arbitrary geometry; the geometry must
      consist only of quads (i.e., no arbitrary triangle meshes).

#### Ambient occlusion (planned)

Minecraft-style ambient occlusion.

### Collision Detection (planned)

TODO

### World Storage (planned)

TODO