use std::collections::{HashMap, HashSet};

use three_d::*;
mod node;
use node::*;
pub mod macros;
mod nodes;
use nodes::*;

fn main() {
    let sphere_node = SphereNode {};

    let transform_node = TransformNode {};

    let window = Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((800, 450)),
        ..Default::default()
    })
    .unwrap();

    let context = window.gl();

    let mut node_graph = NodeGraph::new();

    let sphere_node_index = node_graph.add_node(sphere_node);
    let instatiate_node_index = node_graph.add_node(InstatiateOnPointsNode {});
    let scale_node_index = node_graph.add_node(ScaleInstanceNode {});
    let merge_node_index = node_graph.add_node(MergeNode {});
    let scale_value_node_index = node_graph.add_node(ValueNode::new(0.1 as f32));
    let output_node_index = node_graph.add_node(OutputNode {});

    node_graph.connect(
        NodeSocket::new(sphere_node_index, 0),
        NodeSocket::new(instatiate_node_index, 0),
    );

    node_graph.connect(
        NodeSocket::new(sphere_node_index, 0),
        NodeSocket::new(instatiate_node_index, 1),
    );

    node_graph.connect(
        NodeSocket::new(instatiate_node_index, 0),
        NodeSocket::new(scale_node_index, 0),
    );

    node_graph.connect(
        NodeSocket::new(scale_value_node_index, 0),
        NodeSocket::new(scale_node_index, 1),
    );

    node_graph.connect(
        NodeSocket::new(scale_node_index, 0),
        NodeSocket::new(merge_node_index, 0),
    );

    node_graph.connect(
        NodeSocket::new(merge_node_index, 0),
        NodeSocket::new(output_node_index, 0),
    );

    //let mesh = node_graph.get_output().into_gms(&context);
    let mesh = node_graph.get_output().into_gm_single(&context);
    //println!("mesh: {}", mesh.len());

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

    fn seperate_parts(&self) -> Vec<Model> {
        let mut models = Vec::new();
        let mut sets: Vec<(HashSet<u32>, Vec<u32>)> = vec![];

        for index in self.indices.chunks(3) {
            let mut found = false;
            'set_loop: for set in sets.iter_mut() {
                for i in index.iter() {
                    if set.0.contains(i) {
                        set.0.insert(index[0]);
                        set.0.insert(index[1]);
                        set.0.insert(index[2]);
                        set.1.extend(index[0..3].to_vec());
                        found = true;
                        break 'set_loop;
                    }
                }
            }
            if !found {
                let mut set = HashSet::new();
                set.extend(index[0..3].iter());
                sets.push((set, index[0..3].to_vec()));
            }
        }

        let mut index_to_remove = None;
        // merge the sets
        'outer_loop: loop {
            if let Some(index) = index_to_remove {
                sets.remove(index);
            }
            let set_clone = sets.clone();
            for (mutable_set_index, set) in sets.iter_mut().enumerate() {
                for (i, other_set) in set_clone.iter().enumerate() {
                    if mutable_set_index == i {
                        continue;
                    }
                    if set.0.intersection(&other_set.0).count() > 0 {
                        set.0.extend(other_set.0.iter());
                        set.1.extend(other_set.1.iter());
                        index_to_remove = Some(i);
                        // restart the search as the sets have changed
                        continue 'outer_loop;
                    }
                }
            }
            break;
        }

        let mut global_to_local_indices = vec![];
        for set in sets.iter() {
            let mut global_to_local_index = HashMap::new();
            let sorted = set.0.iter().collect::<Vec<&u32>>();

            for (i, index) in sorted.iter().enumerate() {
                global_to_local_index.insert(*index, i as u32);
            }
            global_to_local_indices.push(global_to_local_index);
        }

        for (i, set) in sets.iter().enumerate() {
            let global_to_local_index = &global_to_local_indices[i];

            let mut model = Model::new();
            for index in set.1.chunks(3) {
                model.add_index(
                    global_to_local_index[&index[0]],
                    global_to_local_index[&index[1]],
                    global_to_local_index[&index[2]],
                );
            }
            for index in set.0.iter() {
                model.add_vertex(
                    self.vertices[*index as usize].x,
                    self.vertices[*index as usize].y,
                    self.vertices[*index as usize].z,
                );
            }

            model.set_transform(self.transform);

            models.push(model);
        }

        models
    }

    fn into_gm_single(&mut self, context: &Context) -> Gm<Mesh, PhysicalMaterial> {
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

    fn into_gms(&mut self, context: &Context) -> Vec<Gm<Mesh, PhysicalMaterial>> {
        let mut gms = Vec::new();

        for mut model in self.seperate_parts() {
            gms.push(model.into_gm_single(context));
        }

        gms
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

    fn transform(&self) -> Matrix4<f32> {
        self.transform
    }

    fn merge(&mut self, other: &Model) {
        let offset = self.vertices.len() as u32;
        for vertex in other.vertices.iter() {
            let mut point = Point3 {
                x: vertex.x,
                y: vertex.y,
                z: vertex.z,
            };

            point = other.transform.transform_point(point);

            self.vertices.push(Vector3::new(point.x, point.y, point.z));
        }
        self.indices
            .extend(other.indices.iter().map(|i| i + offset));
        self.normals.extend(other.normals.iter());
    }
}
