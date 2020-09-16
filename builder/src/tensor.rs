use crate::ast;
use crate::code::Code;
use crate::context::{Build, CloneSafe, Context};
use crate::error::{Result, TensorNodeError};
use crate::execs::ExecIR;
use crate::externs::ExternIR;
use crate::graph::{RefGraph, Table};
use crate::nodes::{ASTBuild, NodeIR, NodeRoot};
use crate::seed::Seed;

#[derive(Debug)]
pub struct TensorGraph(Vec<TensorNode>);

#[derive(Debug)]
pub enum TensorNode {
    Node(NodeIR),
    Extern(ExternIR),
    Exec(ExecIR),
}

#[derive(Debug)]
pub struct IRData {
    pub id: u64,
    pub name: String,
    pub graph: RefGraph,
    pub input: Option<ast::Outs>,
    pub output: Option<ast::Outs>,
}

impl Into<TensorNode> for NodeIR {
    fn into(self) -> TensorNode {
        TensorNode::Node(self)
    }
}

impl Into<TensorGraph> for Vec<TensorNode> {
    fn into(self) -> TensorGraph {
        TensorGraph(self)
    }
}

impl Into<TensorNode> for ExternIR {
    fn into(self) -> TensorNode {
        TensorNode::Extern(self)
    }
}

impl Into<TensorNode> for ExecIR {
    fn into(self) -> TensorNode {
        TensorNode::Exec(self)
    }
}

impl TensorGraph {
    pub fn get_input_shapes(&self) -> Option<&ast::Shapes> {
        let input_node = &self.0[0];
        if input_node.is_input() {
            input_node.get_output_shapes()
        } else {
            input_node.get_input_shapes()
        }
    }

    pub fn get_output_shapes(&self) -> Option<&ast::Shapes> {
        for node in self.0.iter().rev() {
            if let Some(shapes) = node.get_output_shapes() {
                // filter dynamic size
                if shapes.0.len() == 1 {
                    if let Some(None) = shapes.0.get("x") {
                        continue;
                    }
                }
                return Some(shapes);
            }
        }
        self.0.last().unwrap().get_output_shapes()
    }

    pub fn build(self, root: &NodeRoot) -> Result<Vec<Code>> {
        self.0.into_iter().map(|x| x.build_to_code(root)).collect()
    }
}

impl TensorNode {
    pub fn is_input(&self) -> bool {
        match self {
            Self::Node(node) => node.data.id == 0,
            Self::Extern(_) => false,
            Self::Exec(_) => false,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Node(node) => &node.data.name,
            Self::Extern(node) => &node.data.name,
            Self::Exec(node) => &node.data.name,
        }
    }

    fn ty(&self) -> ast::FinalNodeType {
        match self {
            Self::Node(_) | Self::Extern(_) => ast::FinalNodeType::Default,
            Self::Exec(_) => ast::FinalNodeType::Exec,
        }
    }

    pub fn get_id(&self) -> u64 {
        match self {
            Self::Node(node) => node.data.id,
            Self::Extern(node) => node.data.id,
            Self::Exec(node) => node.data.id,
        }
    }

    pub fn set_id(&mut self, id: u64) {
        match self {
            Self::Node(node) => node.data.id = id,
            Self::Extern(node) => node.data.id = id,
            Self::Exec(node) => node.data.id = id,
        }
    }

    pub fn get_input_shapes(&self) -> Option<&ast::Shapes> {
        match self {
            Self::Node(node) => node.get_input_shapes(),
            Self::Extern(node) => node.get_input_shapes(),
            Self::Exec(_) => exec_node_cannot_have_shapes(),
        }
    }

    pub fn get_output_shapes(&self) -> Option<&ast::Shapes> {
        match self {
            Self::Node(node) => node.get_output_shapes(),
            Self::Extern(node) => node.get_output_shapes(),
            Self::Exec(_) => exec_node_cannot_have_shapes(),
        }
    }

    pub fn build_to_code(self, root: &NodeRoot) -> Result<Code> {
        match self {
            Self::Node(node) => Ok(node.build(root)?.into()),
            Self::Extern(node) => Ok(node.build(root)?.into()),
            Self::Exec(node) => Ok(node.build(root)?.into()),
        }
    }

    pub fn apply_variables(&mut self, variables: Table) -> Result<()> {
        todo!()
    }

    pub fn unwrap_node(self) -> Result<NodeIR> {
        match self {
            Self::Node(node) => Ok(node),
            _ => TensorNodeError::MismatchedType {
                expected: ast::FinalNodeType::Default,
                given: self.ty(),
            }
            .into(),
        }
    }
}

fn exec_node_cannot_have_shapes() -> ! {
    unreachable!("The exec node cannot have the shapes");
}

impl CloneSafe for TensorNode {
    fn clone_safe(&self, seed: &Seed, variables: &mut Vec<ast::RefVariable>) -> Self {
        match self {
            Self::Node(node) => node.clone_safe(seed, variables).into(),
            Self::Extern(node) => node.clone_safe(seed, variables).into(),
            Self::Exec(node) => node.clone_safe(seed, variables).into(),
        }
    }
}

impl Build for TensorNode {
    type Output = Self;

    fn build(root: &NodeRoot, name: &str, source: String) -> Result<Self::Output> {
        let file = root.parser.parse_file(&source)?;

        // test name
        if file.node.name != name {
            TensorNodeError::MismatchedName {
                expected: name.to_string(),
                given: file.node.name,
            }
            .into()
        } else {
            let mut ctx = Context::new(root);
            file.build(&mut ctx, Default::default())
        }
    }
}

impl IRData {
    pub fn new(
        name: String,
        graph: RefGraph,
        input: Option<&ast::Shapes>,
        output: Option<&ast::Shapes>,
    ) -> Self {
        Self {
            id: 0,
            name,
            graph,
            input: input.map(|x| x.to_outs(0)),
            output: output.map(|x| x.to_outs(1)),
        }
    }
}

impl CloneSafe for IRData {
    fn clone_safe(&self, seed: &Seed, variables: &mut Vec<ast::RefVariable>) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            graph: self.graph.clone_safe(seed, variables),
            input: self.input.clone(),
            output: self.output.clone(),
        }
    }
}

impl CloneSafe for ast::Shapes {
    fn clone_safe(&self, seed: &Seed, variables: &mut Vec<ast::RefVariable>) -> Self {
        Self(self.0.clone_safe(seed, variables))
    }
}

impl CloneSafe for ast::Shape {
    fn clone_safe(&self, seed: &Seed, variables: &mut Vec<ast::RefVariable>) -> Self {
        Self(self.0.clone_safe(seed, variables))
    }
}

impl CloneSafe for ast::Value {
    fn clone_safe(&self, seed: &Seed, variables: &mut Vec<ast::RefVariable>) -> Self {
        match self {
            Self::Bool(v) => Self::Bool(*v),
            Self::UInt(v) => Self::UInt(*v),
            Self::Int(v) => Self::Int(*v),
            Self::Real(v) => Self::Real(*v),
            Self::Node(v) => Self::Node(v.clone()),
            Self::Dim(v) => Self::Dim(v.clone()),
            Self::Variable(v) => Self::Variable(v.clone_safe(seed, variables)),
            Self::Expr(v) => Self::Expr(v.clone_safe(seed, variables)),
        }
    }
}

impl CloneSafe for ast::Expr {
    fn clone_safe(&self, seed: &Seed, variables: &mut Vec<ast::RefVariable>) -> Self {
        Self {
            op: self.op,
            lhs: self.lhs.clone_safe(seed, variables),
            rhs: self.rhs.clone_safe(seed, variables),
        }
    }
}

impl CloneSafe for ast::RefVariable {
    fn clone_safe(&self, seed: &Seed, variables: &mut Vec<ast::RefVariable>) -> Self {
        todo!()
    }
}
