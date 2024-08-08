use crate::macros::InputOrOutput;
use crate::Model;
use three_d::{CpuMesh, Matrix4, Vector3};

struct NodeSocket {
    node: usize,
    socket: usize,
}

pub struct NodeGraph {
    nodes: Vec<Box<dyn NodeAny>>,
    connections: Vec<(NodeSocket, NodeSocket)>,
}

impl NodeGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_node<N, I, O>(&mut self, node: N)
    where
        I: InputOrOutput<T = I> + 'static,
        O: InputOrOutput<T = O> + 'static,
        N: Node<I, O> + 'static,
    {
        self.nodes.push(Box::new(DynNode::<N, I, O>::from(node)));
    }
}

pub trait NodeAny {
    fn operation(&self, input: Vec<&dyn std::any::Any>) -> Box<dyn std::any::Any>;
    fn needed_types_input(&self) -> Vec<std::any::TypeId>;
    fn needed_types_output(&self) -> Vec<std::any::TypeId>;
}

struct DynNode<N, I, O>
where
    N: Node<I, O>,
    I: InputOrOutput<T = I> + 'static,
    O: InputOrOutput<T = O> + 'static,
{
    node: N,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<N, I, O> NodeAny for DynNode<N, I, O>
where
    N: Node<I, O>,
    I: InputOrOutput<T = I> + 'static,
    O: InputOrOutput<T = O> + 'static,
{
    fn operation(&self, input: Vec<&dyn std::any::Any>) -> Box<dyn std::any::Any> {
        let input = I::convert(input);
        Box::new(self.node.operation(input))
    }

    fn needed_types_input(&self) -> Vec<std::any::TypeId> {
        I::needed_types()
    }

    fn needed_types_output(&self) -> Vec<std::any::TypeId> {
        O::needed_types()
    }
}

impl<N, I, O> From<N> for DynNode<N, I, O>
where
    N: Node<I, O>,
    I: InputOrOutput<T = I> + 'static,
    O: InputOrOutput<T = O> + 'static,
{
    fn from(node: N) -> Self {
        Self {
            node,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub trait Node<I, O>
where
    I: InputOrOutput<T = I> + 'static + Sized,
    O: InputOrOutput<T = O> + 'static + Sized,
{
    fn operation(&self, input: I) -> O;
}

pub struct SphereNode {}

impl Node<((),), (Model,)> for SphereNode {
    fn operation(&self, _: ((),)) -> (Model,) {
        let mut model = crate::Model::new();

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
        (model,)
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
