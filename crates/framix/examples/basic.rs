use framix::floorplan::Root;

fn main() {
  let root = Root::generate(20, 15);
  println!("{}", root.render());
  // dbg!(root);
}
