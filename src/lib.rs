use pyo3::exceptions::PyValueError;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::types::{PyDict, PyList};

fn tab_size(s: &str) -> usize {
    s.chars().take_while(|c| c.is_whitespace()).count()
}

fn match_colon(s: &str) -> Option<&str> {
    if s.ends_with(':') {
        Some(&s[..s.len() - 1])
    } else {
        None
    }
}

fn trim_string(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

struct InternKeys<'py> {
    matches: &'py Bound<'py, PyString>,
    dependencies: &'py Bound<'py, PyString>,
    optional_dependencies: &'py Bound<'py, PyString>,
}

#[pyfunction]
fn yarnlock_parse<'py>(py: Python<'py>, yarnlock: &str) -> PyResult<Bound<'py, PyDict>> {
    // Intern common keys once
    let keys = InternKeys {
        matches: intern!(py, "matches"),
        dependencies: intern!(py, "dependencies"),
        optional_dependencies: intern!(py, "optionalDependencies"),
    };

    let result = PyDict::new(py);
    let mut file_tab_size: Option<usize> = None;
    let mut this_dict: Option<Bound<'py, PyDict>> = None;
    let mut this_subdict: Option<Bound<'py, PyDict>> = None;

    for line in yarnlock
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let line_tab_size = tab_size(line);
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line_tab_size == 0 {
            // new dependency
            this_dict = Some(parse_dependency(py, &result, line, &keys)?);
            continue;
        }

        if file_tab_size.is_none() {
            file_tab_size = Some(line_tab_size);
        }
        let file_tab_size = file_tab_size.unwrap();

        if line_tab_size == file_tab_size {
            match this_dict.as_ref() {
                Some(this_dict) => {
                    parse_property(py, this_dict, line, &mut this_subdict)?;
                }
                None => {
                    return Err(PyValueError::new_err(format!(
                        "Attempted to set property '{line}' before defining dependency"
                    )));
                }
            }
        } else if line_tab_size == file_tab_size * 2 {
            match this_subdict.as_ref() {
                Some(this_subdict) => {
                    parse_sub_property(this_subdict, line)?;
                }
                None => {
                    return Err(PyValueError::new_err(format!(
                        "Attempted to set sub-property '{line}' before defining property"
                    )));
                }
            }
        }
    }

    Ok(result)
}

fn parse_dependency<'py>(
    py: Python<'py>,
    result: &Bound<'py, PyDict>,
    line: &str,
    keys: &InternKeys<'py>,
) -> PyResult<Bound<'py, PyDict>> {
    match match_colon(line) {
        Some(line) => {
            let this_dict = PyDict::new(py);
            let mut this_matches = Vec::new();
            let mut root_set: bool = false;

            let full_key = line.trim_end_matches(':');

            // Set an item with the full dependency key
            result
                .set_item(
                    if !full_key.contains(',') {
                        trim_string(full_key)
                    } else {
                        full_key
                    },
                    &this_dict,
                )
                .unwrap();

            for component in full_key.split(", ") {
                match trim_string(component).rsplit_once('@') {
                    Some((name, version)) => {
                        if !root_set {
                            // Set an item with just the package name
                            result.set_item(name, &this_dict).unwrap();
                            root_set = true;
                        }
                        this_matches.push(version);
                    }
                    None => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid dependency line: {line}"
                        )));
                    }
                }
            }

            assert!(root_set);

            this_dict
                .set_item(keys.matches, PyList::new(py, this_matches).unwrap())
                .unwrap();
            this_dict
                .set_item(keys.dependencies, PyDict::new(py))
                .unwrap();
            this_dict
                .set_item(keys.optional_dependencies, PyDict::new(py))
                .unwrap();

            Ok(this_dict)
        }
        None => Err(PyValueError::new_err(format!(
            "Invalid dependency line: {line}"
        ))),
    }
}

fn parse_property<'py>(
    py: Python<'py>,
    this_dict: &Bound<'py, PyDict>,
    line: &str,
    this_subdict: &mut Option<Bound<'py, PyDict>>,
) -> PyResult<()> {
    match match_colon(line) {
        Some(key) => {
            *this_subdict = Some(PyDict::new(py));
            this_dict
                .set_item(trim_string(key), this_subdict.as_ref())
                .unwrap();
            Ok(())
        }
        None => match line.split_once(' ') {
            Some((key, val)) => {
                this_dict
                    .set_item(trim_string(key), trim_string(val))
                    .unwrap();
                Ok(())
            }
            None => Err(PyValueError::new_err(format!(
                "Invalid property line: {line}"
            ))),
        },
    }
}

fn parse_sub_property<'py>(this_subdict: &Bound<'py, PyDict>, line: &str) -> PyResult<()> {
    match line.split_once(' ') {
        Some((key, val)) => {
            this_subdict
                .set_item(trim_string(key), trim_string(val))
                .unwrap();
            Ok(())
        }
        None => Err(PyValueError::new_err(format!(
            "Invalid sub-property line: {line}"
        ))),
    }
}

#[pymodule]
#[pyo3(name = "_lib")]
fn lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(yarnlock_parse, m)?)?;
    Ok(())
}
