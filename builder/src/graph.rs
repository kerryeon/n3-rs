use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use n3_parser::ast;

use crate::context::CloneSafe;
use crate::error::{GraphError, Result};
use crate::seed::Seed;
use crate::variable::*;

pub type RefGraph = Rc<RefCell<Graph>>;

#[derive(Debug)]
pub struct Graph {
    pub id: u64,
    shortcuts: Table,
    variables: Table,
}

pub type Table = BTreeMap<String, ast::RefVariable>;
pub type Values = BTreeMap<String, Option<ast::Value>>;

impl Graph {
    pub fn with_id(id: u64) -> Self {
        Self {
            id,
            shortcuts: Table::new(),
            variables: Table::new(),
        }
    }

    pub fn new(seed: &Seed) -> Self {
        Self::with_id(seed.generate())
    }

    pub fn with_one_var(seed: &Seed, name: &str, value: Option<ast::Value>) -> Self {
        let mut graph = Self::new(&seed);

        let mut value = ast::Variable::with_name_value(name.to_string(), value);
        value.id = Some(graph.id);
        value.id_old = Some(graph.id);

        graph.add(value.into()).unwrap();
        graph
    }

    pub fn try_with_variables<I>(id: u64, variables: I) -> Result<Self>
    where
        I: IntoIterator<Item = (String, ast::NodeLet)>,
    {
        let mut graph = Graph {
            id,
            shortcuts: Table::new(),
            variables: variables
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        ast::Variable {
                            id: Some(id),
                            id_old: Some(id),
                            name: v.name,
                            shortcut: v.shortcut,
                            ty: Some(v.ty),
                            value: v.value,
                        }
                        .into(),
                    )
                })
                .collect(),
        };
        graph.build()?;
        Ok(graph)
    }

    pub fn add(&mut self, variable: ast::RefVariable) -> Result<()> {
        let mut var_ref = variable.borrow_mut();
        let name = var_ref.name.clone();

        if self.variables.contains_key(&name) {
            return GraphError::DuplicatedVariable { name }.into();
        }

        var_ref.id = Some(self.id);
        var_ref.id_old = Some(self.id);
        drop(var_ref);

        self.variables.insert(name, variable);
        Ok(())
    }

    pub fn apply(&self, variables: Values, shortcut: bool) -> Result<()> {
        let self_variables = if shortcut {
            &self.shortcuts
        } else {
            &self.variables
        };

        for (name, v) in variables.into_iter() {
            if let Some(var) = self_variables.get(&name) {
                var.borrow_mut().value = v;
            } else {
                return GraphError::NoSuchVariable {
                    name,
                    candidates: self_variables.keys().cloned().collect(),
                }
                .into();
            }
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&ast::RefVariable> {
        self.variables.get(name).ok_or_else(|| {
            GraphError::NoSuchVariable {
                name: name.to_string(),
                candidates: self.variables.keys().cloned().collect(),
            }
            .into()
        })
    }

    fn build(&mut self) -> Result<()> {
        let shortcuts_map = self
            .variables
            .iter()
            .filter_map(|(k, v)| {
                v.borrow()
                    .shortcut
                    .as_ref()
                    .and_then(|s| Some((s.clone(), k.clone())))
            })
            .collect();

        let variables = self
            .variables
            .iter()
            .map(|(k, v)| {
                let mut names = Vec::new();
                let value = v.replace_to(&mut names, &self.variables, &shortcuts_map)?;
                Ok((k.clone(), value))
            })
            .collect::<Result<_>>()?;

        self.variables = variables;
        self.shortcuts = self
            .variables
            .values()
            .map(|var| {
                let borrowed = var.borrow();
                let name = borrowed
                    .shortcut
                    .as_ref()
                    .or_else(|| Some(&borrowed.name))
                    .cloned()
                    .unwrap();
                (name, var.clone())
            })
            .collect();

        Ok(())
    }

    pub fn hint(&self, out: &ast::Out, shape: &ast::Shape) -> Result<ast::Shape> {
        let dims = shape
            .0
            .iter()
            .enumerate()
            .map(|(dim, v)| v.hint(&self.shortcuts, out, dim, true))
            .collect::<Result<_>>()?;
        Ok(ast::Shape(dims))
    }

    pub fn replace_to(&self, variable: Option<ast::Value>) -> Result<Option<ast::Value>> {
        if let Some(variable) = variable {
            match variable {
                ast::Value::Variable(var) => {
                    let var_borrow = var.borrow();
                    if let Some(var) = self.shortcuts.get(&var_borrow.name) {
                        Ok(Some(ast::Value::Variable(var.clone())))
                    } else {
                        drop(var_borrow);
                        Ok(Some(ast::Value::Variable(var)))
                    }
                }
                ast::Value::Expr(mut expr) => {
                    expr.lhs = self.replace_to(Some(expr.lhs))?.unwrap();
                    expr.rhs = self.replace_to(expr.rhs)?;
                    Ok(Some(ast::Value::Expr(expr)))
                }
                _ => Ok(Some(variable)),
            }
        } else {
            Ok(None)
        }
    }

    pub fn unload_dims(&mut self) -> Values {
        self.variables
            .iter_mut()
            .filter(|(_, v)| v.borrow().ty == Some(ast::LetType::Dim))
            .map(|(k, v)| (k.clone(), v.borrow_mut().value.take()))
            .collect()
    }

    pub fn load_dims_weakly(&mut self, values: Values) {
        for (name, value) in values {
            let var = &self.variables[&name];
            let mut var_ref = var.borrow_mut();

            if var_ref.value.is_none() {
                var_ref.value = value;
            }
        }
    }

    pub fn variables(&self) -> &Table {
        &self.variables
    }

    pub fn into_variables(self) -> Table {
        self.variables
    }
}

impl From<Graph> for RefGraph {
    fn from(graph: Graph) -> Self {
        Rc::new(RefCell::new(graph))
    }
}

impl Estimable for Graph {
    fn is_estimable(&self) -> bool {
        self.variables.is_estimable()
    }
}

impl CloneSafe for Graph {
    fn clone_safe(&self, seed: &Seed, variables: &mut Vec<ast::RefVariable>) -> Self {
        let id = seed.generate();

        // Step 1. get the copies
        let mut self_variables: Table = self
            .variables
            .iter()
            .map(|(k, v)| (k.clone(), v.detach(id)))
            .collect();
        let self_shortcuts = self_variables
            .values()
            .filter_map(|v| {
                v.borrow()
                    .shortcut
                    .as_ref()
                    .and_then(|s| Some((s.clone(), v.clone())))
            })
            .collect();
        for var in self_variables.values_mut() {
            variables.push(var.clone());
            // Step 2. replace the olds into the news
            let new_var = var.borrow().value.clone_value(variables);
            var.borrow_mut().value = new_var;
        }
        // Step 3. store
        Graph {
            id,
            shortcuts: self_shortcuts,
            variables: self_variables,
        }
    }
}
