use super::{Mesh, MeshingView};

pub trait Mesher {
    fn generate_mesh<V>(&mut self, view: V) -> Mesh
    where
        V: MeshingView;
}
