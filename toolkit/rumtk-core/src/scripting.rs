/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2025  Luis M. Santos, M.D.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

pub mod python_utils {
    use std::ffi::{CString, OsStr};
    use std::fmt::Debug;
    use std::fs::read_to_string;
    use std::path::Path;

    use crate::core::RUMResult;
    use crate::strings::RUMString;
    use compact_str::format_compact;
    use pyo3::prelude::*;
    use pyo3::types::{PyList, PyTuple};

    pub type RUMPyArgs = Py<PyTuple>;
    pub type RUMPyList = Py<PyList>;
    pub type RUMPyResult = Vec<RUMString>;
    pub type RUMPyModule = Py<PyModule>;
    pub type RUMPyTuple = Py<PyTuple>;
    pub type RUMPyFunction = Py<PyAny>;
    pub type RUMPyAny = Py<PyAny>;
    pub type RUMPython<'py> = Python<'py>;

    fn string_to_cstring(data: &str) -> RUMResult<CString> {
        match CString::new(data) {
            Ok(code) => Ok(code),
            Err(e) => Err(format_compact!(
                "Could not cast Python code string to a C string!"
            )),
        }
    }

    fn ostring_to_cstring(data: &OsStr) -> RUMResult<CString> {
        let data_str = match data.to_str() {
            Some(s) => s,
            None => return Err(format_compact!("Could not cast OsStr to a str!")),
        };
        match CString::new(data_str) {
            Ok(code) => Ok(code),
            Err(e) => Err(format_compact!(
                "Could not cast Python code string to a C string because {:#?}!",
                e
            )),
        }
    }

    pub fn py_list_to_tuple(py: RUMPython, py_list: &RUMPyList) -> RUMResult<RUMPyTuple> {
        match PyTuple::new(py, py_list.bind(py).iter()) {
            Ok(py_args) => Ok(py_args.into()),
            Err(e) => Err(format_compact!(
                "Failed to convert arguments from PyList to PyTuple! Reason: {:?}",
                e
            )),
        }
    }

    ///
    /// Convert a vector of `T` to a Python List of `T`.
    ///
    /// ## Example
    ///
    /// ```
    ///     use compact_str::format_compact;
    ///     use pyo3::Python;
    ///     use crate::rumtk_core::scripting::python_utils::{py_buildargs, py_extract_string_vector, py_list_to_tuple};
    ///
    ///     let expect: Vec<&str> = vec!["a", "1", "2"];
    ///
    ///     Python::attach( |py| {
    ///             let py_args = py_buildargs(py, &expect).unwrap();
    ///             let py_obj = py_list_to_tuple(py, &py_args).unwrap();
    ///             let result = py_extract_string_vector(&py_obj).unwrap();
    ///             assert_eq!(&result, &expect, "{}", format_compact!("Python list does not match the input list!\nGot: {:?}\nExpected: {:?}", &result, &expect));
    ///         }
    ///     )
    /// ```
    ///
    pub fn py_buildargs<'a, 'py, T>(py: RUMPython<'py>, args: &Vec<T>) -> RUMResult<RUMPyList>
    where
        T: FromPyObject<'a, 'py> + IntoPyObject<'py> + Debug + Clone,
    {
        match PyList::new(py, args.clone()) {
            Ok(py_args) => Ok(py_args.into()),
            Err(e) => Err(
                format_compact!(
                    "Failed to convert arguments into a Python Object for transfer to Interpreter! Arguments: {:?} Reason: {:?}",
                    &args,
                    e.to_string()
                )
            )
        }
    }

    ///
    /// Create empty Python List, which can be used for creating a collection of arguments to pass
    /// to script.
    ///
    /// ## Example
    ///
    /// ```
    ///     use compact_str::format_compact;
    ///     use pyo3::Python;
    ///     use pyo3::types::{PyListMethods, PyAnyMethods};
    ///     use rumtk_core::scripting::python_utils::{py_new_args, py_push_arg, RUMPyArgs, RUMPyList};
    ///     use crate::rumtk_core::scripting::python_utils::{py_buildargs, py_extract_string_vector};
    ///
    ///
    ///     Python::attach( |py| {
    ///             let example_arg_1 = 1;
    ///             let example_arg_2 = "Hello";
    ///             let py_args: RUMPyList = py_new_args(py);
    ///             py_push_arg(py, &py_args, example_arg_1.clone()).unwrap();
    ///             py_push_arg(py, &py_args, example_arg_2.clone()).unwrap();
    ///             let arg_1: usize = py_args.bind(py).get_item(0).unwrap().extract().unwrap();
    ///             assert_eq!(&example_arg_1, &arg_1, "{}", format_compact!("Python list does not match the input list!\nGot: {:?}\nExpected: {:?}", &arg_1, &example_arg_1));
    ///         }
    ///     )
    /// ```
    ///
    pub fn py_new_args(py: RUMPython) -> RUMPyList {
        PyList::empty(py).unbind()
    }

    ///
    /// Push argument of type `T` into instance of Python List. We can then use the list to pass
    /// arguments to Python function or method.
    ///
    /// ## Example
    ///
    /// ```
    ///     use compact_str::format_compact;
    ///     use pyo3::Python;
    ///     use pyo3::types::{PyListMethods, PyAnyMethods};
    ///     use rumtk_core::scripting::python_utils::{py_new_args, py_push_arg, RUMPyArgs, RUMPyList};
    ///     use crate::rumtk_core::scripting::python_utils::{py_buildargs, py_extract_string_vector};
    ///
    ///
    ///     Python::attach( |py| {
    ///             let example_arg_1 = 1;
    ///             let example_arg_2 = "Hello";
    ///             let py_args: RUMPyList = py_new_args(py);
    ///             py_push_arg(py, &py_args, example_arg_1.clone()).unwrap();
    ///             py_push_arg(py, &py_args, example_arg_2.clone()).unwrap();
    ///             let arg_1: usize = py_args.bind(py).get_item(0).unwrap().extract().unwrap();
    ///             assert_eq!(&example_arg_1, &arg_1, "{}", format_compact!("Python list does not match the input list!\nGot: {:?}\nExpected: {:?}", &arg_1, &example_arg_1));
    ///         }
    ///     )
    /// ```
    ///
    pub fn py_push_arg<'a, 'py, T>(py: RUMPython<'py>, py_args: &RUMPyList, arg: T) -> RUMResult<()>
    where
        T: FromPyObject<'a, 'py> + IntoPyObject<'py> + Debug + Clone,
    {
        match py_args.bind(py).append(arg.clone()) {
            Ok(_) => Ok(()),
            Err(e) => Err(
                format_compact!(
                    "Failed to convert argument into a Python Object for transfer to Interpreter! Argument: {:?} Reason: {:?}",
                    &arg,
                    e.to_string()
                )
            )
        }
    }

    fn string_vector_to_rumstring_vector(list: &Vec<String>) -> RUMPyResult {
        let mut rumstring_vector = Vec::<RUMString>::with_capacity(list.len());

        for itm in list {
            rumstring_vector.push(RUMString::from(itm));
        }

        rumstring_vector
    }

    pub fn py_extract_string_vector(pyargs: &RUMPyArgs) -> RUMResult<RUMPyResult> {
        Python::attach(|py| -> RUMResult<RUMPyResult> {
            let py_list: Vec<String> = match pyargs.extract(py) {
                Ok(list) => list,
                Err(e) => {
                    return Err(format_compact!(
                        "Could not extract list from Python args! Reason => {:?}",
                        e
                    ));
                }
            };
            Ok(string_vector_to_rumstring_vector(&py_list))
        })
    }

    ///
    /// Extract value returned from functions and modules via a `PyAny` object.
    ///
    /// ## Example Usage
    ///
    /// ### Example W/ RustType
    ///
    /// ```
    ///use compact_str::format_compact;
    ///     use pyo3::Python;
    ///     use pyo3::types::{PyListMethods, PyAnyMethods, PyString};
    ///     use rumtk_core::scripting::python_utils::{py_extract_any, py_new_args, py_push_arg, RUMPyArgs, RUMPyList};
    ///     use crate::rumtk_core::scripting::python_utils::{py_buildargs, py_extract_string_vector};
    ///
    ///
    ///     Python::attach(|py| {
    ///             let example_arg_1 = "Hello";
    ///             let py_arg = PyString::new(py, example_arg_1);
    ///             let arg: String = py_arg.extract().unwrap();
    ///             let arg_1: String = py_extract_any(py, &py_arg.as_any().clone().unbind()).unwrap();
    ///             assert_eq!(&example_arg_1, &arg_1, "{}", format_compact!("Python conversion failed!\nGot: {:?}\nExpected: {:?}", &arg_1, &example_arg_1));
    ///         }
    ///     )
    /// ```
    ///
    /// ### Example W/ Custom Type
    ///
    /// ```
    ///use compact_str::format_compact;
    ///     use pyo3::{Python, pyclass, IntoPyObjectExt};
    ///     use pyo3::types::{PyListMethods, PyAnyMethods, PyString};
    ///     use rumtk_core::scripting::python_utils::{py_extract_any, py_new_args, py_push_arg, RUMPyAny, RUMPyArgs, RUMPyList};
    ///     use crate::rumtk_core::scripting::python_utils::{py_buildargs, py_extract_string_vector};
    ///
    ///     #[pyclass]
    ///     #[derive(Clone, Debug, PartialOrd, PartialEq)]
    ///     struct MyWrapper {
    ///         text: String
    ///     }
    ///
    ///     Python::attach(|py| {
    ///             let example_arg_1 = MyWrapper{text: String::from("Hello")};
    ///             let py_arg: RUMPyAny = example_arg_1.clone().into_py_any(py).unwrap();
    ///             let arg_1: MyWrapper = py_extract_any(py, &py_arg).unwrap();
    ///             assert_eq!(&example_arg_1, &arg_1, "{}", format_compact!("Python conversion failed!\nGot: {:?}\nExpected: {:?}", &arg_1, &example_arg_1));
    ///         }
    ///     )
    /// ```
    ///
    pub fn py_extract_any<'py, T>(py: Python<'py>, pyresult: &'py RUMPyAny) -> RUMResult<T>
    where
        T: FromPyObject<'py, 'py> + Clone,
        <T as pyo3::FromPyObject<'py, 'py>>::Error: Debug,
    {
        match pyresult.extract(py) {
            Ok(r) => {
                let val = r;
                Ok(val)
            }
            Err(e) => Err(format_compact!(
                "Could not extract vector from Python result! Reason => {:?}",
                e
            )),
        }
    }

    ///
    /// Load a python module from a given file path!
    ///
    /// ## Example Usage
    ///
    /// ```
    ///     use compact_str::format_compact;
    /// use pyo3::Python;
    ///     use pyo3::types::PyModule;
    ///     use rumtk_core::scripting::python_utils::RUMPyModule;
    ///     use crate::rumtk_core::scripting::python_utils::{py_load};
    ///     use rumtk_core::strings::RUMString;
    ///     use uuid::Uuid;
    ///
    ///     let expected: &str = "print('Hello World!')\ndef test():\n\treturn 'Hello'";
    ///     let fpath: RUMString = format_compact!("/tmp/{}.py", Uuid::new_v4());
    ///     std::fs::write(&fpath, expected.as_bytes()).expect("Failure to write test module.");
    ///
    ///     Python::attach(|py| {
    ///         let py_obj: RUMPyModule = py_load(py, &fpath).expect("Failure to load module!");
    ///     });
    ///     std::fs::remove_file(&fpath).unwrap()
    /// ```
    ///
    pub fn py_load(py: Python, fpath: &str) -> RUMResult<RUMPyModule> {
        let pypath = Path::new(fpath);
        let pycode = match read_to_string(fpath) {
            Ok(code) => string_to_cstring(&code)?,
            Err(e) => {
                return Err(format_compact!(
                    "Unable to read Python file {}. Is it valid?",
                    &fpath
                ));
            }
        };
        let filename = match pypath.file_name() {
            Some(name) => ostring_to_cstring(name)?,
            None => {
                return Err(format_compact!("Invalid Python module path {}!", &fpath));
            }
        };
        let modname = match pypath.file_stem() {
            Some(name) => ostring_to_cstring(name)?,
            None => {
                return Err(format_compact!("Invalid Python module path {}!", &fpath));
            }
        };
        let pymod = match PyModule::from_code(py, pycode.as_c_str(), &filename, &modname) {
            Ok(pymod) => pymod,
            Err(e) => {
                return Err(format_compact!(
                    "Failed to load Python module {} because of {:#?}!",
                    &fpath,
                    e
                ));
            }
        };
        Ok(pymod.into())
    }

    ///
    /// Function for executing a python module's function.
    /// If you set the argument `func_name` to an empty string, `py_exec` will execute the module
    /// itself!!!
    ///
    /// # Examples
    ///
    /// ## Executing Function Within Module
    ///
    /// ```
    ///     use compact_str::format_compact;
    ///     use pyo3::{Python, IntoPyObjectExt};
    ///     use pyo3::types::PyModule;
    ///     use rumtk_core::scripting::python_utils::{RUMPyAny, RUMPyArgs, RUMPyModule, RUMPyList};
    ///     use crate::rumtk_core::scripting::python_utils::{py_load, py_exec, py_buildargs, py_list_to_tuple};
    ///     use uuid::Uuid;
    ///     use rumtk_core::strings::RUMString;
    ///
    ///     let expected: &str = "print('Hello World!')\ndef test():\n\treturn 'Hello'";
    ///     let fpath: RUMString = format_compact!("/tmp/{}.py", Uuid::new_v4());
    ///     std::fs::write(&fpath, expected.as_bytes()).expect("Failure to write test module.");
    ///
    ///     let expect: Vec<&str> = vec![];
    ///
    ///     Python::attach( |py| {
    ///         let py_obj: RUMPyModule = py_load(py, &fpath).expect("Failure to load module!");
    ///         let args: RUMPyList = py_buildargs(py, &expect).unwrap();
    ///
    ///         let result = py_exec(py, &py_obj, "test", &args).expect("Failed to extract result!");
    ///    });
    ///
    ///     std::fs::remove_file(&fpath).unwrap()
    ///```
    ///
    /// ## Executing Module
    ///
    /// ```
    ///     use compact_str::format_compact;
    ///     use pyo3::{Python, IntoPyObjectExt};
    ///     use pyo3::types::PyModule;
    ///     use rumtk_core::scripting::python_utils::{RUMPyAny, RUMPyArgs, RUMPyModule, RUMPyList};
    ///     use crate::rumtk_core::scripting::python_utils::{py_load, py_exec, py_new_args};
    ///     use uuid::Uuid;
    ///     use rumtk_core::strings::RUMString;
    ///
    ///     let expected: &str = "print('Hello World!')\ndef test():\n\treturn 'Hello'";
    ///     let fpath: RUMString = format_compact!("/tmp/{}.py", Uuid::new_v4());
    ///     std::fs::write(&fpath, expected.as_bytes()).expect("Failure to write test module.");
    ///
    ///     let expect: Vec<&str> = vec![];
    ///
    ///     Python::attach( |py| {
    ///         let py_obj: RUMPyModule = py_load(py, &fpath).expect("Failure to load module!");
    ///         let args: RUMPyList = py_new_args(py);
    ///
    ///         let result = py_exec(py, &py_obj, "", &args).expect("Failed to extract result!");
    ///    });
    ///
    ///     std::fs::remove_file(&fpath).unwrap()
    ///```
    ///
    pub fn py_exec(
        py: Python,
        pymod: &RUMPyModule,
        func_name: &str,
        args: &RUMPyList,
    ) -> RUMResult<RUMPyAny> {
        if (!func_name.is_empty()) {
            let pyfunc: RUMPyFunction = match pymod.getattr(py, func_name) {
                Ok(f) => f,
                Err(e) => {
                    return Err(format_compact!(
                        "No function named {} found in module! Error: {:#?}",
                        &func_name,
                        e
                    ));
                }
            };
            match pyfunc.call1(py, py_list_to_tuple(py, args)?) {
                Ok(r) => Ok(r),
                Err(e) => Err(format_compact!(
                    "An error occurred executing Python function {}. Error: {}",
                    &func_name,
                    e
                )),
            }
        } else {
            match pymod.call0(py) {
                Ok(r) => Ok(r),
                Err(e) => Err(format_compact!(
                    "An error occurred executing Python module {}. Error: {}",
                    &func_name,
                    e
                )),
            }
        }
    }
}

pub mod python_macros {
    ///
    /// Load a module text into RAM.
    ///
    /// ## Example
    /// ```
    ///     use std::fs::write;
    ///     use uuid::Uuid;
    ///     use crate::rumtk_core::rumtk_python_exec;
    ///
    ///     let module_fname = format!("{}_module.py", Uuid::new_v4());
    ///     let module_contents = "print(\"Hello World!\")";
    ///     write(&module_fname, module_contents).expect("Failed to write file!");
    ///
    ///     let module_data = rumtk_python_exec!(&module_fname).unwrap();
    ///
    ///     assert_eq!(module_contents, module_data, "Loaded wrong data!")
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_python_exec {
        ( $py:expr, $mod_path:expr) => {{
            use compact_str::format_compact;
            use pyo3::types::PyModule;
            use pyo3::{IntoPyObjectExt, Python};
            use rumtk_core::scripting::python_utils::{
                RUMPyAny, RUMPyArgs, RUMPyList, RUMPyModule,
            };
            use rumtk_core::strings::RUMString;
            use $crate::rumtk_core::scripting::python_utils::{
                py_buildargs, py_exec, py_list_to_tuple, py_load, py_new_args,
            };

            // Load module
            let pymod: RUMPyModule = py_load($py, $mod_path)?;

            // Empty args
            let args = py_new_args(py);

            // Let's execute against arguments
            py_exec($py, pymod, "", &args)
        }};
        ( $py:expr, $mod_path:expr, $func_name:expr ) => {{
            use compact_str::format_compact;
            use pyo3::types::PyModule;
            use pyo3::{IntoPyObjectExt, Python};
            use rumtk_core::scripting::python_utils::{
                RUMPyAny, RUMPyArgs, RUMPyList, RUMPyModule,
            };
            use rumtk_core::strings::RUMString;
            use $crate::rumtk_core::scripting::python_utils::{
                py_buildargs, py_exec, py_list_to_tuple, py_load, py_new_args,
            };

            // Load module
            let pymod: RUMPyModule = py_load($py, $mod_path)?;

            // Empty args
            let args = py_new_args(py);

            // Let's execute against arguments
            py_exec($py, pymod, $func_name, &args)
        }};
        ( $py:expr, $mod_path:expr, $func_name:expr, $args:expr ) => {{
            use compact_str::format_compact;
            use pyo3::types::PyModule;
            use pyo3::{IntoPyObjectExt, Python};
            use rumtk_core::scripting::python_utils::{
                RUMPyAny, RUMPyArgs, RUMPyList, RUMPyModule,
            };
            use rumtk_core::strings::RUMString;
            use $crate::rumtk_core::scripting::python_utils::{
                py_buildargs, py_exec, py_list_to_tuple, py_load,
            };

            // Load module
            let pymod: RUMPyModule = py_load($py, $mod_path)?;

            // Let's execute against arguments
            py_exec($py, pymod, $func_name, $args)
        }};
    }
}
