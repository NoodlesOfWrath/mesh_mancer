use three_d::Mesh;

struct Node<I, O> {
    operation: fn(I) -> O,
}

use three_d::*;
fn main() {
    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();
    let mesh = Mesh::new(&context, &CpuMesh::sphere(32));
    let mesh = Gm::new(
        mesh,
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Srgba::new(128, 128, 128, 255),
                ..Default::default()
            },
        ),
    );

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

struct Point {
    x: f32,
    y: f32,
    z: f32,
}

struct Index {
    a: u32,
    b: u32,
    c: u32,
}

struct Model {
    vertices: Vec<Vector3<f32>>,
    indices: Vec<u32>,
}

impl Model {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_vertex(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.push(Vector3::new(x, y, z));
    }

    fn add_index(&mut self, a: u32, b: u32, c: u32) {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
    }

    fn into_gm(&self, context: &Context) -> Gm<Mesh, PhysicalMaterial> {
        let mesh = Mesh::new(
            context,
            &CpuMesh {
                positions: Positions::F32(self.vertices.clone()),
                indices: Indices::U32(self.indices.clone()),
                normals: None,
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
}
