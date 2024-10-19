use chrono::{DateTime, Utc};
use pyo3::prelude::*;

use super::{AirMeasurement, LightMeasurement, WaterLevelMeasurement};

#[pymethods]
impl AirMeasurement {
    #[new]
    #[pyo3(signature = (measure_time, label, humidity=None, pressure=None, resistance=None, temperature=None))]
    fn py_new(
        measure_time: DateTime<Utc>,
        label: String,
        humidity: Option<f64>,
        pressure: Option<f64>,
        resistance: Option<f64>,
        temperature: Option<f64>,
    ) -> Self {
        Self {
            measure_time: measure_time.timestamp(),
            label,
            humidity,
            pressure,
            resistance,
            temperature,
        }
    }
}

#[pymethods]
impl LightMeasurement {
    #[new]
    #[pyo3(signature = (measure_time, label, illuminance=None))]
    fn py_new(measure_time: DateTime<Utc>, label: String, illuminance: Option<f64>) -> Self {
        Self {
            measure_time: measure_time.timestamp(),
            label,
            illuminance,
        }
    }
}

#[pymethods]
impl WaterLevelMeasurement {
    #[new]
    #[pyo3(signature = (measure_time, label, distance=None))]
    fn py_new(measure_time: DateTime<Utc>, label: String, distance: Option<u32>) -> Self {
        Self {
            measure_time: measure_time.timestamp(),
            label,
            distance,
        }
    }
}

#[pymodule]
pub fn grow_pylib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<AirMeasurement>()?;
    m.add_class::<LightMeasurement>()?;
    m.add_class::<WaterLevelMeasurement>()?;
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn foo() {
//         pyo3::append_to_inittab!(grow_pylib);
//         pyo3::prepare_freethreaded_python();
//         let py_app = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../test.py"));
//         Python::with_gil(|py| {
//             // PyModule::from_code_bound(py, py_foo, "utils.foo", "utils.foo")?;
//             let app: Py<PyAny> = PyModule::from_code_bound(py, py_app, "", "").unwrap()
//                 .getattr("measure").unwrap()
//                 .into();
//             let res = app.call0(py).unwrap();
//             let measurement: PyRef<'_, WaterLevelMeasurement> = res.extract(py).unwrap();
//             println!("{measurement:?}");
//         });
//         // println!("py: {}", from_python);
//         // //"export" our API module to the python runtime
//         // pyo3::append_to_inittab!(grow_pylib);
//         // //spawn runtime
//         // pyo3::prepare_freethreaded_python();
//         // Python::with_gil(|py| -> Result<(), Box<dyn std::error::Error>> {
//         //     let py_app = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/test.py"));
//         //     let from_python = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
//         //         // PyModule::from_code_bound(py, py_foo, "utils.foo", "utils.foo")?;
//         //         let app: Py<PyAny> = PyModule::from_code_bound(py, py_app, "", "")?
//         //             .getattr("measure")?
//         //             .into();
//         //         app.call0(py)
//         //     });
//         //     //add the current directory to import path of Python (do not use this in production!)
//         //     let syspath: Bound<PyList> = py.import_bound("sys")?.getattr("path")?.extract()?;
//         //     syspath.insert(0, Path::new("./"))?;
//         //     println!("Import path is: {:?}", syspath);
//         //
//         //     // Now we can load our python_plugin/gadget_init_plugin.py file.
//         //     // It can in turn import other stuff as it deems appropriate
//         //     let plugin = PyModule::import_bound(py, "test")?;
//         //     // and call start function there, which will return a python reference to Gadget.
//         //     // Gadget here is a "pyclass" object reference
//         //     let measurement = plugin.getattr("measure")?.call0()?;
//         //
//         //     //now we extract (i.e. mutably borrow) the rust struct from python object
//         //     // {
//         //     //     //this scope will have mutable access to the gadget instance, which will be dropped on
//         //     //     //scope exit so Python can access it again.
//         //     //     let mut gadget_rs: PyRefMut<'_, plugin_api::Gadget> = gadget.extract()?;
//         //     //     // we can now modify it as if it was a native rust struct
//         //     //     gadget_rs.prop = 42;
//         //     //     //which includes access to rust-only fields that are not visible to python
//         //     //     println!("rust-only vec contains {:?}", gadget_rs.rustonly);
//         //     //     gadget_rs.rustonly.clear();
//         //     // }
//         //
//         //     //any modifications we make to rust object are reflected on Python object as well
//         //     // let res: usize = gadget.getattr("prop")?.extract()?;
//         //     println!("{measurement:?}");
//         //     Ok(())
//         // })
//         // .unwrap();
//     }
// }
