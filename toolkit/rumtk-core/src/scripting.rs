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
    use std::fs::read_to_string;
    use std::path::Path;

    use crate::core::RUMResult;
    use crate::strings::RUMString;
    use compact_str::format_compact;
    use pyo3::prelude::*;
    use pyo3::types::{PyList, PyTuple};

    pub type RUMPyArgs = Py<PyList>;
    pub type RUMPyModule = Py<PyModule>;
    pub type RUMPyTuple = Py<PyTuple>;
    pub type RUMPyFunction = Py<PyAny>;

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

    ///
    /// Convert a vector of strings to a Python List of strings.
    ///
    /// ## Example
    ///
    /// ```
    ///     use compact_str::format_compact;
    ///     use crate::rumtk_core::scripting::python_utils::{py_buildargs, py_extract};
    ///
    ///     let expect: Vec<&str> = vec!["a", "1", "2"];
    ///
    ///     let py_obj = py_buildargs(&expect).unwrap();
    ///     let result = py_extract(&py_obj).unwrap();
    ///     assert_eq!(&result, &expect, "{}", format_compact!("Python list does not match the input list!\nGot: {:?}\nExpected: {:?}", &result, &expect));
    /// ```
    ///
    pub fn py_buildargs(arg_list: &Vec<&str>) -> RUMResult<RUMPyArgs> {
        Python::with_gil(|py| -> RUMResult<RUMPyArgs> {
            match PyList::new(py, arg_list){
                Ok(pylist) => Ok(pylist.into()),
                Err(e) => {
                    Err(format_compact!(
                            "Could not convert argument list {:#?} into a Python args list because of {:#?}!",
                            &arg_list,
                            e
                        ))
                }
            }
        })
    }

    pub fn py_extract(pyargs: &RUMPyArgs) -> RUMResult<Vec<String>> {
        Python::with_gil(|py| -> RUMResult<Vec<String>> {
            let py_list: Vec<String> = match pyargs.extract(py) {
                Ok(list) => list,
                Err(e) => {
                    return Err(format_compact!(
                        "Could not extract list from Python args! Reason => {:?}",
                        e
                    ));
                }
            };
            Ok(py_list)
        })
    }

    fn py_extract_string_tuple<'py>(
        py: &Python<'py>,
        pyresult: &RUMPyTuple,
    ) -> RUMResult<Vec<RUMString>> {
        let pyresult_vec: Vec<String> = match pyresult.extract(*py) {
            Ok(vec) => vec,
            Err(e) => {
                return Err(format_compact!(
                    "Could not extract vector from Python result! Reason => {:?}",
                    e
                ))
            }
        };
        let mut rumstring_vector = Vec::<RUMString>::with_capacity(pyresult_vec.len());

        for itm in pyresult_vec {
            rumstring_vector.push(RUMString::from(itm));
        }

        Ok(rumstring_vector)
    }

    ///
    /// Load a python module from a given file path!
    ///
    /// ## Example Usage
    ///
    /// ```
    ///     use compact_str::format_compact;
    ///     use pyo3::types::PyModule;
    ///     use rumtk_core::scripting::python_utils::RUMPyModule;
    ///     use crate::rumtk_core::scripting::python_utils::{py_load};
    ///
    ///     let expected: &str = "print('Hello World!')";
    ///     let fpath: &str = "/tmp/example.py";
    ///     std::fs::write(&fpath, expected.as_bytes()).expect("Failure to write test module.");
    ///
    ///     let py_obj: RUMPyModule = py_load(&fpath).expect("Failure to load module!");
    ///
    ///     std::fs::remove_file(&fpath).unwrap()
    /// ```
    ///
    pub fn py_load(fpath: &str) -> RUMResult<RUMPyModule> {
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
        Python::with_gil(|py| -> RUMResult<RUMPyModule> {
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
        })
    }

    ///
    ///
    ///
    pub fn py_exec(
        pymod: &RUMPyModule,
        func_name: &str,
        args: &RUMPyArgs,
    ) -> RUMResult<Vec<RUMString>> {
        Python::with_gil(|py| -> RUMResult<Vec<RUMString>> {
            let pyfunc: RUMPyFunction = match pymod.getattr(py, func_name) {
                Ok(f) => f.into(),
                Err(e) => {
                    return Err(format_compact!(
                        "No function named {} found in module! Error: {:#?}",
                        &func_name,
                        e
                    ));
                }
            };
            let result: RUMPyTuple = match pyfunc.call1(py, &args) {
                Ok(r) => r.into(),
                Err(e) => {
                    return Err(format_compact!(
                        "An error occurred executing Python function {}. Error: {}",
                        &func_name,
                        e
                    ))
                }
            };
            py_extract_string_tuple(&py, &result)
        })
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
    ///     use crate::rumtk_core::rumtk_python_load_module;
    ///
    ///     let module_fname = format!("{}_module.py", Uuid::new_v4());
    ///     let module_contents = "print(\"Hello World!\")";
    ///     write(&module_fname, module_contents).expect("Failed to write file!");
    ///
    ///     let module_data = rumtk_python_load_module!(&module_fname).unwrap();
    ///
    ///     assert_eq!(module_contents, module_data, "Loaded wrong data!")
    /// ```
    ///
    #[macro_export]
    macro_rules! rumtk_python_exec {
        ( $mod_path:expr ) => {{
            use compact_str::format_compact;
            use pyo3::{prelude::*, types::IntoPyDict};
            use $crate::scripting::python_utils::{py_exec, py_load};
            let pymod = py_load($mod_path)?;
        }};
        ( $mod_path:expr, $func_name:expr ) => {{
            use compact_str::format_compact;
            use pyo3::{prelude::*, types::IntoPyDict};
            use $crate::scripting::python_utils::{py_buildargs, py_exec, py_load};
            let pymod = py_load($mod_path)?;
            let args = py_buildargs(&vec![])?;
            py_exec(pymod, $func_name, &args)
        }};
        ( $mod_path:expr, $func_name:expr, $($args:expr),+ ) => {{
            use compact_str::format_compact;
            use pyo3::{prelude::*, types::IntoPyDict};
            use $crate::scripting::python_utils::{py_buildargs, py_exec, py_load};
            let pymod = py_load($mod_path)?;
            let args = py_buildargs(&vec![$($arg_items:expr),+])?;
            py_exec(pymod, $func_name, &args)
        }};
    }
}
