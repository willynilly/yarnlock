use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
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

#[pyfunction]
fn yarnlock_parse<'py>(py: Python<'py>, yarnlock: &str) -> PyResult<Bound<'py, PyDict>> {
    let result = PyDict::new_bound(py);
    let mut file_tab_size: Option<usize> = None;
    let mut this_dict: Option<Bound<'py, PyDict>> = None;
    let mut this_subdict: Option<Bound<'py, PyDict>> = None;

    for line in yarnlock
        .lines()
        .into_iter()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let line_tab_size = tab_size(line);
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line_tab_size == 0 {
            // new dependency
            this_dict = Some(parse_dependency(py, &result, line)?);
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
                    )))
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
                    )))
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
) -> PyResult<Bound<'py, PyDict>> {
    match match_colon(line) {
        Some(line) => {
            let this_dict_ = PyDict::new_bound(py);
            let mut this_matches = Vec::new();
            let mut root_set: bool = false;

            for component in line.trim_end_matches(':').split(", ") {
                match trim_string(component).rsplit_once('@') {
                    Some((name, version)) => {
                        if !root_set {
                            result.set_item(name.to_string(), &this_dict_).unwrap();
                            root_set = true;
                        }
                        this_matches.push(version.to_string());
                    }
                    None => {
                        return Err(PyValueError::new_err(format!(
                            "Invalid dependency line: {line}"
                        )))
                    }
                }
            }

            assert!(root_set);
            this_dict_
                .set_item("matches", PyList::new_bound(py, this_matches))
                .unwrap();
            this_dict_
                .set_item("dependencies", PyDict::new_bound(py))
                .unwrap();
            this_dict_
                .set_item("optionalDependencies", PyDict::new_bound(py))
                .unwrap();
            Ok(this_dict_)
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
            *this_subdict = Some(PyDict::new_bound(py));
            this_dict
                .set_item(trim_string(key).to_string(), this_subdict.as_ref())
                .unwrap();
            Ok(())
        }
        None => match line.split_once(' ') {
            Some((key, val)) => {
                this_dict
                    .set_item(trim_string(key).to_string(), trim_string(val).to_string())
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
                .set_item(trim_string(key).to_string(), trim_string(val).to_string())
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
