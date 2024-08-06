use three_d::Mesh;

struct Node<I, O> {
    operation: fn(I) -> O,
}

impl<I, O> Node<I, O> {
    fn operation(&self, input: I) -> O {
        (self.operation)(input)
    }
}

use three_d::*;
fn main() {
    let sphere_node = Node {
        operation: |()| -> Model {
            let mut model = Model::new();
            let sphere = CpuMesh::sphere(32);
            for vertex in sphere.positions.into_f32().iter() {
                model.add_vertex(vertex.x, vertex.y, vertex.z);
            }
            let mut a = None;
            let mut b = None;
            let mut c = None;
            let indices = sphere.indices.into_u32().expect("Indices are not u32");
            for index in indices.iter() {
                if a.is_none() {
                    a = Some(*index);
                } else if b.is_none() {
                    b = Some(*index);
                } else if c.is_none() {
                    c = Some(*index);
                    model.add_index(a.unwrap(), b.unwrap(), c.unwrap());
                    a = None;
                    b = None;
                    c = None;
                }
            }
            model
        },
    };

    let translate_node = Node {
        operation: |info: (Model, Vector3<f32>)| -> Model {
            let mut model = info.0;
            let vector3 = info.1;

            for vertex in model.vertices.iter_mut() {
                *vertex += vector3;
            }

            model
        },
    };

    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    let context = window.gl();
    let mesh = translate_node
        .operation((sphere_node.operation(()), Vector3::new(0.0, 0.0, 0.0)))
        .into_gm(&context);

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(5.0, 2.0, 2.5),
        vec3(0.0, 0.0, -0.5),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let light0 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, -0.5, -0.5));
    let light1 = DirectionalLight::new(&context, 1.0, Srgba::WHITE, &vec3(0.0, 0.5, 0.5));

    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
            .render(&camera, (&mesh).into_iter(), &[&light0, &light1]);

        FrameOutput::default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let node = Node {
            operation: |x: i32| x + 1,
        };
        assert_eq!((node.operation)(1), 2);
    }
}

struct Model {
    vertices: Vec<Vector3<f32>>,
    indices: Vec<u32>,
    normals: Vec<Vector3<f32>>,
    normals_calculated: bool,
}

impl Model {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            normals_calculated: false,
        }
    }

    fn add_vertex(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.push(Vector3::new(x, y, z));
        self.normals_calculated = false;
    }

    fn add_index(&mut self, a: u32, b: u32, c: u32) {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
        self.normals_calculated = false;
    }

    fn into_gm(&mut self, context: &Context) -> Gm<Mesh, PhysicalMaterial> {
        if !self.normals_calculated {
            self.auto_generate_normals();
        }

        let mesh = Mesh::new(
            context,
            &CpuMesh {
                positions: Positions::F32(self.vertices.clone()),
                indices: Indices::U32(self.indices.clone()),
                normals: Some(self.normals.clone()),
                uvs: None,
                colors: None,
                tangents: None,
            },
        );

        let gm = Gm::new(
            mesh,
            PhysicalMaterial::new_opaque(
                context,
                &CpuMaterial {
                    albedo: Srgba::new(128, 128, 128, 255),
                    ..Default::default()
                },
            ),
        );

        gm
    }

    fn auto_generate_normals(&mut self) {
        let mut normals = vec![Vector3::new(0.0, 0.0, 0.0); self.vertices.len()];
        for i in 0..self.indices.len() / 3 {
            let a = self.indices[i * 3] as usize;
            let b = self.indices[i * 3 + 1] as usize;
            let c = self.indices[i * 3 + 2] as usize;
            let normal = (self.vertices[b] - self.vertices[a])
                .cross(self.vertices[c] - self.vertices[a])
                .normalize();
            normals[a] += normal;
            normals[b] += normal;
            normals[c] += normal;
        }
        self.normals = normals;
        self.normals_calculated = true;
    }
}
