use std::any::{Any, TypeId};

use crate::Model;
use crate::{macros::InputOrOutput, OutputNode};
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
            let operation_result = self.nodes_elements[node].node.operation(Vec::new());
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
                        "Invalid type for input, expected {:?}, got {:?}. From node: {}, to node: {}",
                        needed_type, actual_type, input.from.node, input.to.node
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

    pub fn get_nodes(&self) -> Vec<&dyn NodeAny> {
        self.nodes_elements
            .iter()
            .map(|x| x.node.as_ref())
            .collect()
    }

    pub fn get_node(&self, index: usize) -> &dyn NodeAny {
        self.nodes_elements[index].node.as_ref()
    }
}

pub trait NodeAny {
    fn operation(&self, input: Vec<&dyn std::any::Any>) -> Vec<Box<dyn std::any::Any>>;
    fn needed_types_input(&self) -> Vec<std::any::TypeId>;
    fn needed_types_output(&self) -> Vec<std::any::TypeId>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

struct DynNode<N, I, O>
where
    N: Node<I, O>,
    I: InputOrOutput<T = I> + 'static,
    O: InputOrOutput<T = O> + 'static,
{
    node: N,
    _phantom: std::marker::PhantomData<(I, O)>,
    name: String,
    description: String,
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

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

impl<N, I, O> From<N> for DynNode<N, I, O>
where
    N: Node<I, O>,
    I: InputOrOutput<T = I> + 'static,
    O: InputOrOutput<T = O> + 'static,
{
    fn from(node: N) -> Self {
        let mut name = std::any::type_name::<N>().to_string();
        if N::name() != "" {
            name = N::name().to_string();
        }

        Self {
            node,
            _phantom: std::marker::PhantomData,
            name,
            description: N::description(),
        }
    }
}

pub trait Node<I, O>
where
    I: InputOrOutput<T = I> + 'static + Sized,
    O: InputOrOutput<T = O> + 'static + Sized,
{
    fn operation(&self, input: I) -> O;
    fn name() -> String {
        "".to_string()
    }
    fn description() -> String {
        "".to_string()
    }
}
