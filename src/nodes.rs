use crate::macros::Input;
use crate::Model;
use three_d::{CpuMesh, Matrix4, Vector3};

pub trait Node<I: Input, O> {
    fn operation(&self, input: I) -> O;
}

pub struct SphereNode {}

impl Node<((),), Model> for SphereNode {
    fn operation(&self, _: ((),)) -> Model {
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
    }
}

pub struct TransformNode {}

impl Node<(Model, Vector3<f32>), Model> for TransformNode {
    fn operation(&self, info: (Model, Vector3<f32>)) -> Model {
        let mut model = info.0;
        let vector3 = info.1;

        let transform = Matrix4::from_translation(vector3);
        model.set_transform(transform);

        model
    }
}
