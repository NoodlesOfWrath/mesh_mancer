use three_d::*;
mod nodes;
use nodes::*;
pub mod macros;

fn main() {
    let sphere_node = SphereNode {};

    let transform_node = TransformNode {};

    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((2560, 1440)),
        ..Default::default()
    })
    .unwrap();

    let context = window.gl();

    let mut node_graph = NodeGraph::new();

    let sphere_node_index = node_graph.add_node(sphere_node);
    let transform_node_index = node_graph.add_node(transform_node);
    node_graph.connect(
        NodeSocket::new(sphere_node_index, 0),
        NodeSocket::new(transform_node_index, 0),
    );
    let output_node_index = node_graph.add_node(OutputNode {});
    node_graph.connect(
        NodeSocket::new(transform_node_index, 0),
        NodeSocket::new(output_node_index, 0),
    );
    let mesh = node_graph.get_output().into_gm(&context);

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

#[derive(Clone)]
pub struct Model {
    vertices: Vec<Vector3<f32>>,
    indices: Vec<u32>,
    normals: Vec<Vector3<f32>>,
    transform: Matrix4<f32>,
    normals_calculated: bool,
}

impl Model {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            transform: Matrix4::identity(),
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

        let mut gm = Gm::new(
            mesh,
            PhysicalMaterial::new_opaque(
                context,
                &CpuMaterial {
                    albedo: Srgba::new(128, 128, 128, 255),
                    ..Default::default()
                },
            ),
        );

        gm.set_transformation(self.transform);

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

    fn set_transform(&mut self, transform: Matrix4<f32>) {
        self.transform = transform;
    }
}
