use crate::Model;
use crate::{macros::InputOrOutput, Node};
use three_d::{CpuMesh, Matrix4, Vector3};

// just acts as a tag to get the output of the graph
pub struct OutputNode {}

impl Node<(Model,), (Model,)> for OutputNode {
    fn operation(&self, model: (Model,)) -> (Model,) {
        model
    }
}

pub struct TransformNode {}

impl Node<(Model, Vector3<f32>), (Model,)> for TransformNode {
    fn operation(&self, info: (Model, Vector3<f32>)) -> (Model,) {
        let mut model = info.0;
        let vector3 = info.1;

        let transform = Matrix4::from_translation(vector3);
        model.set_transform(transform);

        (model,)
    }
}

pub struct ValueNode<T> {
    value: T,
}

impl<T> ValueNode<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> Node<((),), (T,)> for ValueNode<T>
where
    T: Clone + 'static,
{
    fn operation(&self, _: ((),)) -> (T,) {
        (self.value.clone(),)
    }
}

pub struct SphereNode {}

impl Node<((),), (Model,)> for SphereNode {
    fn operation(&self, _: ((),)) -> (Model,) {
        let mut model = crate::Model::new();

        let sphere = CpuMesh::sphere(4);
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
        (model,)
    }
}

pub struct InstatiateOnPointsNode {}

impl Node<(Model, Model), (Vec<Model>,)> for InstatiateOnPointsNode {
    fn operation(&self, info: (Model, Model)) -> (Vec<Model>,) {
        let model = info.0;
        let points = info.1;

        println!("spawning: {} instances", points.vertices.iter().count());

        let mut models = Vec::new();
        for vertex in points.vertices.iter() {
            let mut m = model.clone();
            m.set_transform(Matrix4::from_translation(*vertex));
            models.push(m);
        }

        (models,)
    }
}

pub struct ScaleInstanceNode {}

impl Node<(Vec<Model>, f32), (Vec<Model>,)> for ScaleInstanceNode {
    fn operation(&self, info: (Vec<Model>, f32)) -> (Vec<Model>,) {
        let models = info.0;
        let scale = info.1;

        let mut new_models = Vec::new();
        for mut model in models {
            model.set_transform(model.transform() * Matrix4::from_scale(scale));
            new_models.push(model);
        }

        (new_models,)
    }
}

pub struct MergeNode {}

impl Node<(Vec<crate::Model>,), (crate::Model,)> for MergeNode {
    fn operation(&self, info: (Vec<crate::Model>,)) -> (crate::Model,) {
        let mut model = crate::Model::new();

        for m in info.0 {
            model.merge(&m);
        }

        (model,)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use three_d::Vector3;

    #[test]
    fn test_transform_node() {
        let transform_node = TransformNode {};
        let mut model = crate::Model::new();
        model.add_vertex(0.0, 0.0, 0.0);
        model.add_vertex(1.0, 1.0, 1.0);
        model.add_index(0, 1, 2);
        let vector3 = Vector3::new(1.0, 1.0, 1.0);
        let model = transform_node.operation((model, vector3));
        assert_eq!(model.0.transform, Matrix4::from_translation(vector3));
    }
}
