use serde::{Deserialize}; //Для сборки разборки struct<->json
use serde_pyobject::{to_pyobject, from_pyobject}; //serde_json <-> py_dict

use std::collections::HashMap; //Тип для py_env

use crate::py_env::{get_py_guards, PyFileModule}; //Взять гварды и тип к ним
use crate::tools::ToolRequest; //Тип для запроса

//Для PyEnv
use pyo3::prelude::*;
use pyo3::types::{PyFunction};
use pyo3::exceptions::PyValueError;

/*
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Access
{
    Deny = 0,
    Allow = 1,
}
*/

#[derive(Deserialize, Debug, Clone)]
pub struct GuardResponse
{
    pub allowed: bool,
    pub reason: String,
}

//Гвард проверяющий инструменты
pub async fn tools_guard(request: ToolRequest) -> (GuardResponse, ToolRequest)
{
    //Спавн в блокирующий поток, чтобы не уйти в дедлок
    return tokio::task::spawn_blocking(move || -> (GuardResponse, ToolRequest)
    {
        let guards: &HashMap<String, PyFileModule> = get_py_guards(); //Функции гвардов

        //Выборочная
        let guard: &Py<PyFunction> = match guards.get("tools_guard").expect("Гвард не найден").funcs.get("guard_select")
        {
            Some(guard) => guard,

            None =>
            {
                return (GuardResponse { allowed: false, reason: String::from("Guard not covered this call") }, request);
            }
        };

        match Python::attach(|py: Python<'_>| -> PyResult<GuardResponse>
        {
            let args: Bound<'_, PyAny> = to_pyobject(py, &request)?; //Арги

            let ret: Py<PyAny> = guard.call1(py, (args,))?; //Вызов

            let verdict: PyResult<GuardResponse> = from_pyobject(ret.into_bound(py)).map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()));

            return verdict;
        })
        {
        Ok(verdict) => 
        {
            return (verdict, request);
        }

        Err(err) => 
        {
            return (GuardResponse { allowed: false, reason: err.to_string() }, request);
        }
    }; }).await.expect("Guard thread joining error");
}
