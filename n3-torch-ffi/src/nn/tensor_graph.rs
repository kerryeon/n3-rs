use std::ops::{Deref, DerefMut};

use pyo3::prelude::*;
use pyo3::types::PyList;

use crate::machine::Torch;

pub struct TensorGraph(PyObject);

impl TensorGraph {
    pub fn new(py: Python, nodes: Py<PyList>) -> PyResult<Self> {
        Ok(Self {
            0: Torch(py).nn("ModuleList")?.call1((nodes,))?.into_py(py),
        })
    }

    pub fn parameters(&self, py: Python) -> PyResult<PyObject> {
        self.0.call_method0(py, "parameters")
    }
}

impl Deref for TensorGraph {
    type Target = PyObject;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TensorGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use pyo3::types::IntoPyDict;

    use super::*;
    use crate::machine::GenericMachine;

    #[test]
    fn test_linear() -> Result<(), ()> {
        fn linear(py: Python, input_channels: usize, output_channels: usize) -> PyResult<PyObject> {
            Ok(Torch(py)
                .nn("Linear")?
                .call(
                    (),
                    Some(
                        [
                            ("in_features", input_channels),
                            ("out_features", output_channels),
                        ]
                        .into_py_dict(py),
                    ),
                )?
                .into_py(py))
        }

        Python::with_gil(|py| {
            let mut machine = GenericMachine::new(py).into();

            // get a sample tensor graph
            let tensor_graph = TensorGraph::new(
                py,
                vec![
                    linear(py, 16, 32)?,
                    linear(py, 32, 64)?,
                    linear(py, 64, 10)?,
                ],
            )?;

            // get a sample 3x16 tensor
            let mut output = machine.torch.this("zeros")?.call1((3, 16))?;

            // propagate (16 -> 32 -> 64 -> 10)
            let mut nodes = tensor_graph.as_ref(py).iter()?;
            while let Some(node) = nodes.next() {
                let node = node?;
                output = node.call_method1("forward", (output,))?;
            }

            // test output shape
            assert_eq!(output.getattr("shape")?.extract::<(_, _)>()?, (3, 10));

            machine.terminate()
        })
        .map_err(|e: PyErr| Python::with_gil(|py| e.print_and_set_sys_last_vars(py)))
    }
}
