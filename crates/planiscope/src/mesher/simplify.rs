use super::FullMesh;

impl FullMesh {
  fn calculate_face_normal(&self, triangle: usize) -> glam::Vec3A {
    let triangle = self.triangles[triangle];
    let a = self.vertices[triangle[0] as usize];
    let b = self.vertices[triangle[1] as usize];
    let c = self.vertices[triangle[2] as usize];
    let ab = b - a;
    let ac = c - a;
    ab.cross(ac).normalize()
  }

  fn are_coplanar(&self, triangle_a: usize, triangle_b: usize) -> bool {
    let normal_a = self.calculate_face_normal(triangle_a);
    let normal_b = self.calculate_face_normal(triangle_b);
    let dot = normal_a.dot(normal_b);
    dot > 0.9999 || dot < -0.9999
  }

  fn calculate_vertex_to_triangle_map(&self) -> Vec<Vec<usize>> {
    let mut vertex_to_triangle_map =
      vec![Vec::with_capacity(5); self.vertices.len()];
    for (triangle_index, triangle) in self.triangles.iter().enumerate() {
      for vertex in triangle.to_array() {
        vertex_to_triangle_map[vertex as usize].push(triangle_index);
      }
    }
    vertex_to_triangle_map
  }

  fn find_coplanar_groups(
    &self,
    vertex_to_triangle_map: &[Vec<usize>],
  ) -> Vec<Vec<usize>> {
    let mut coplanar_groups = Vec::new();
    let mut visited_triangles = vec![false; self.triangles.len()];

    for (triangle_index, _) in self.triangles.iter().enumerate() {
      if visited_triangles[triangle_index] {
        continue;
      }
      let mut coplanar_group = Vec::new();
      let mut queue = vec![triangle_index];
      while let Some(triangle_index) = queue.pop() {
        if visited_triangles[triangle_index] {
          continue;
        }
        visited_triangles[triangle_index] = true;
        coplanar_group.push(triangle_index);
        for vertex in self.triangles[triangle_index].to_array() {
          for neighbor in &vertex_to_triangle_map[vertex as usize] {
            if !visited_triangles[*neighbor]
              && self.are_coplanar(triangle_index, *neighbor)
            {
              queue.push(*neighbor);
            }
          }
        }
      }
      coplanar_groups.push(coplanar_group);
    }
    coplanar_groups
  }

  // projects points into common plane
  fn project_points_into_polygon(&self, coplanar_group: &[usize]) -> Polygon {
    let plane_normal = self.calculate_face_normal(coplanar_group[0]);

    let mut projected_vertices = Vec::new();
    let mut projected_to_original = Vec::new();

    // iterate through the triangles in the group
    let triangles = coplanar_group
      .iter()
      .map(|triangle| {
        // find the triangle in the original array and iterate through it
        let new_triangle = self.triangles[*triangle]
          .to_array()
          .iter()
          .map(|i| {
            // find the vertex in the original array and project it
            let vertex = self.vertices[*i as usize];
            let projected_vertex = project_into_plane(vertex, plane_normal);

            // add the projected vertex if it's not already in the list
            let index = projected_to_original
              .iter()
              .position(|v| v == i)
              .unwrap_or_else(|| {
                projected_vertices.push(projected_vertex);
                projected_to_original.push(*i);
                projected_vertices.len() - 1
              });
            index as u32
          })
          .collect::<Vec<_>>();
        // reassamble the triangle with indexes from the projected vertex list
        glam::UVec3::from_slice(new_triangle.as_slice())
      })
      .collect::<Vec<_>>();

    Polygon {
      projected_vertices,
      triangles,
      projected_to_original,
      original_triangles: coplanar_group.to_vec(),
    }
  }

  fn retriangulate_from_polygons(&mut self, polygons: &[Polygon]) {
    // remove old triangles
    let mut tris_to_remove = polygons
      .iter()
      .flat_map(|p| p.original_triangles.iter().copied())
      .collect::<Vec<_>>();
    tris_to_remove.sort();

    tris_to_remove.into_iter().rev().for_each(|i| {
      self.triangles.swap_remove(i);
    });

    // add new triangles
    self.triangles.extend(polygons.iter().flat_map(|polygon| {
      polygon.triangles.iter().map(|t| {
        glam::UVec3::from_slice(
          t.to_array()
            .iter()
            .map(|i| polygon.projected_to_original[*i as usize])
            .collect::<Vec<_>>()
            .as_slice(),
        )
      })
    }));
  }

  pub fn simplify(&mut self) {
    let vertex_to_triangle_map = self.calculate_vertex_to_triangle_map();
    let coplanar_groups = self.find_coplanar_groups(&vertex_to_triangle_map);
    let polygons = coplanar_groups
      .iter()
      .map(|g| {
        let mut polygon = self.project_points_into_polygon(g);
        polygon.retriangulate();
        polygon
      })
      .collect::<Vec<_>>();
    self.retriangulate_from_polygons(&polygons);
  }
}

#[inline]
fn project_into_plane(
  vertex: glam::Vec3A,
  plane_normal: glam::Vec3A,
) -> glam::Vec2 {
  vertex.project_onto(plane_normal).truncate()
}

#[derive(Debug)]
struct Polygon {
  projected_vertices:    Vec<glam::Vec2>,
  triangles:             Vec<glam::UVec3>,
  projected_to_original: Vec<u32>,
  original_triangles:    Vec<usize>,
}

impl Polygon {
  fn retriangulate(&mut self) {
    let border_point_indexes = self
      .ordered_boundary_edges()
      .into_iter()
      .map(|edge| edge.0)
      .collect::<Vec<_>>();
    let points = border_point_indexes
      .iter()
      .map(|i| delaunator::Point {
        x: self.projected_vertices[*i].x as f64,
        y: self.projected_vertices[*i].y as f64,
      })
      .collect::<Vec<_>>();
    let triangulation = delaunator::triangulate(&points);
    if triangulation.triangles.len() > self.triangles.len() {
      // skipping bc of no improvement
      return;
    }

    self.triangles = triangulation
      .triangles
      .iter()
      .map_windows(|a: &[&usize; 3]| {
        glam::UVec3::new(*a[0] as u32, *a[1] as u32, *a[2] as u32)
      })
      .collect::<Vec<_>>();
  }
}
