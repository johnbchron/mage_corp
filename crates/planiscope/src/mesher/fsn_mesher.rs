use crate::mesher::{
  Composition, FastSurfaceNetsMesher, FullMesh, Mesher, MesherInputs,
};

impl Mesher for FastSurfaceNetsMesher {
  fn build_mesh(
    _comp: &Composition,
    _inputs: MesherInputs,
  ) -> Result<FullMesh, fidget::Error> {
    todo!()
  }
}
