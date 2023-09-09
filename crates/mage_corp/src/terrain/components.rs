// use planiscope::shape::ShapeLike;
// use serde::Serialize;

// #[derive(Debug, Clone, Serialize)]
// pub struct HillyLand {}

// impl ShapeLike for HillyLand {
//   fn compile_solid(
//     &self,
//     ctx: &mut fidget::Context,
//     settings: &planiscope::comp::CompilationSettings,
//   ) -> fidget::context::Node {
//     let x = ctx.x();
//     let y = ctx.y();
//     let z = ctx.z();

//     let one = ctx.constant(1.0);
//     let starting_plane = ctx.sub(y, one).unwrap();

//     let variation = {
//       let period_scale = ctx.constant(10.0);
//       let magnitude_scale = ctx.constant(2.0);
//       let x_sin_input = ctx.div(x, period_scale).unwrap();
//       let z_sin_input = ctx.div(z, period_scale).unwrap();
//       let x_sin = ctx.sin(x_sin_input).unwrap();
//       let z_sin = ctx.sin(z_sin_input).unwrap();
//       let sin_sum = ctx.add(x_sin, z_sin).unwrap();
//       let sin_scaled = ctx.mul(sin_sum, magnitude_scale).unwrap();
//       sin_scaled
//     };

//     ctx.add(starting_plane, variation).unwrap()
//     // starting_plane
//   }


// starting_plane = "y - 1.0"
// variation = "(sin(x / 10.0) + sin(z / 10.0)) * 2.0"
// result = "(y - 1.0) + ((sin(x / 10.0) + sin(z / 10.0)) * 2.0)"


//   fn compile_color(
//     &self,
//     ctx: &mut fidget::Context,
//     settings: &planiscope::comp::CompilationSettings,
//   ) -> fidget::context::Node {
//     self.compile_solid(ctx, settings)
//   }
// }
