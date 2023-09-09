use crate::mesher::{
  Composition, FastSurfaceNetsMesher, FullMesh, Mesher, MesherInputs,
};

impl Mesher for FastSurfaceNetsMesher {
  fn build_mesh(
    comp: Composition,
    inputs: MesherInputs,
  ) -> Result<FullMesh, fidget::Error> {
    todo!()
  }
}
