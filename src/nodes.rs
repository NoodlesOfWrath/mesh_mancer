use std::any::{Any, TypeId};

use crate::macros::InputOrOutput;
use crate::Model;
use three_d::{CpuMesh, Matrix4, Vector3};

#[derive(Clone)]
pub struct NodeSocket {
    node: usize,
    socket: usize,
}

impl NodeSocket {
    pub fn new(node: usize, socket: usize) -> Self {
        Self { node, socket }
    }
}

pub struct Connection {
    from: NodeSocket,
    to: NodeSocket,
}

impl Connection {
    pub fn new(from: NodeSocket, to: NodeSocket) -> Self {
        Self { from, to }
    }
}

struct NodeGraphElement {
    node: Box<dyn NodeAny>,
    inputs: Vec<Connection>,
    outputs: Vec<Connection>,
}

pub struct NodeGraph {
    // an index to the output node
    output_node: Option<usize>,
    nodes_elements: Vec<NodeGraphElement>,
}

impl NodeGraph {
    pub fn new() -> Self {
        Self {
            nodes_elements: Vec::new(),
            output_node: None,
        }
    }

    pub fn add_node<N, I, O>(&mut self, node: N) -> usize
    where
        I: InputOrOutput<T = I> + 'static,
        O: InputOrOutput<T = O> + 'static,
        N: Node<I, O> + 'static,
    {
        self.nodes_elements.push(NodeGraphElement {
            node: Box::new(DynNode::<N, I, O>::from(node)),
            inputs: Vec::new(),
            outputs: Vec::new(),
        });
        let index = self.nodes_elements.len() - 1;

        if TypeId::of::<N>() == TypeId::of::<OutputNode>() {
            self.output_node = Some(index);
        }

        index
    }

    pub fn connect(&mut self, from: NodeSocket, to: NodeSocket) {
        self.nodes_elements[to.node].inputs.push(Connection {
            from: from.clone(),
            to: to.clone(),
        });
        self.nodes_elements[from.node]
            .outputs
            .push(Connection { from, to });
    }

    pub fn get_output(&self) -> crate::Model {
        let output_node = self.output_node.expect("Output node not found");
        let output = self.get_output_of_node(output_node);
        let model = output[0]
            .downcast_ref::<Model>()
            .expect("Output is not a model");
        model.clone()
    }

    pub fn get_output_of_node(&self, node: usize) -> Vec<Box<dyn Any>> {
        // if the node has no inputs, then we can just call the operation
        if self.nodes_elements[node]
            .node
            .needed_types_input()
            .is_empty()
        {
            println!("Node: {}", node);
            let operation_result = self.nodes_elements[node].node.operation(Vec::new());
            println!("Operation result: {:?}", operation_result);
            operation_result
        } else {
            // if the node has inputs, then we need to get the output of the inputs
            let mut inputs = Vec::new();
            for input in self.nodes_elements[node].inputs.iter() {
                // check the type validity of the input
                let needed_type =
                    self.nodes_elements[input.to.node].node.needed_types_input()[input.to.socket];

                let actual_type = self.nodes_elements[input.from.node]
                    .node
                    .needed_types_output()[input.from.socket];

                if needed_type != actual_type {
                    panic!(
                        "Invalid type for input, expected {:?}, got {:?}",
                        needed_type, actual_type
                    );
                }

                inputs.push((
                    self.get_output_of_node(input.from.node)
                        .remove(input.from.socket),
                    input.to.socket,
                ));
            }
            // sort the inputs by the socket index
            inputs.sort_by(|a, b| a.1.cmp(&b.1));

            if inputs.len() != self.nodes_elements[node].node.needed_types_input().len() {
                panic!("Not enough inputs for node {}", node);
            }

            // get the references of the inputs as well as discarding the socket index
            let input_refs: Vec<&dyn Any> = inputs.iter().map(|x| x.0.as_ref()).collect();
            self.nodes_elements[node].node.operation(input_refs)
        }
    }
}

pub trait NodeAny {
    fn operation(&self, input: Vec<&dyn std::any::Any>) -> Vec<Box<dyn std::any::Any>>;
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
    fn operation(&self, input: Vec<&dyn std::any::Any>) -> Vec<Box<dyn std::any::Any>> {
        let input = I::convert(input);
        let output = self.node.operation(input);
        O::convert_output(&output)
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

// just acts as a tag to get the output of the graph
pub struct OutputNode {}

impl Node<(Model,), (Model,)> for OutputNode {
    fn operation(&self, model: (Model,)) -> (Model,) {
        model
    }
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
