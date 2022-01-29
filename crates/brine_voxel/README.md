# brine_voxel

A library for rendering Minecraft worlds.

TODO: Update README.

Currently all that is implemented is two different [chunk builders] (["visible
faces"] and ["naive blocks"]) that generate meshes from chunk data. The former
is implemented using the [`block-mesh`] crate.

[chunk builders]: src/chunk_builder/
["visible faces"]: src/chunk_builder/visible_faces.rs
["naive blocks"]: src/chunk_builder/naive_blocks.rs
[`block-mesh`]: https://github.com/bonsairobo/block-mesh-rs

The [`chunktool`](../../src/bin/chunktool/) utility aims to aid with the
development of this crate.