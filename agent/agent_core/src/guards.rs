use serde::{Deserialize}; //Для сборки разборки struct<->json

use std::collections::HashMap; //Тип для py_env

use crate::py_env::{get_py_guards, PyFileModule}; //Взять гварды и тип к ним
use crate::tools::ToolRequest; //Тип для запроса

use serde_pyobject::{to_pyobject, from_pyobject};

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
pub async fn tools_guard(request: &ToolRequest) -> GuardResponse
{
    let guards: &HashMap<String, PyFileModule> = get_py_guards().await;

    let guard: &Py<PyFunction> = match guards.get("tools_guard").expect("Гвард не найден").funcs.get("guard_select")
    {
        Some(guard) => guard,

        None =>
        {
            return GuardResponse { allowed: false, reason: String::from("Guard not covered this call") };
        }
    };

    return match Python::attach(|py: Python<'_>| -> PyResult<GuardResponse>
    {
        let args: Bound<'_, PyAny> = to_pyobject(py, &request)?;

        let ret: Py<PyAny> = guard.call1(py, (args,))?;

        let verdict: GuardResponse = from_pyobject(ret.into_bound(py)).map_err(|e| PyErr::new::<PyValueError, _>(e.to_string()))?;

        return Ok(verdict);
    })
    {
        Ok(verdict) => 
        {
            verdict
        }

        Err(err) => 
        {
            GuardResponse { allowed: false, reason: err.to_string() }
        }
    };
}

/*
pub async fn tools_guard(toolname: String, args: Value) -> GuardResponse
{
    let mut child: Child = Command::new("python")
    .arg("../guards.py")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn() //Создать как пиздюка
    .expect("Guard не запустился");

    let request: GuardRequest = GuardRequest { tool: toolname, args}; //Запрос в гвард

    //Забираем поток, чтобы после {} блока он уничтожился и Py пересрал т.е. перестал ждать и получил EOF. 
    if let Some(mut stdin) = child.stdin.take() //Забрать поток у пиздюка
    {
        let json: String = serde_json::to_string(&request).unwrap(); //Создать Request в строке

        stdin.write_all(json.as_bytes()).await.unwrap(); //Отправить туда
    }

    let output: Output = child.wait_with_output().await.unwrap(); //Сбор урожая

    return serde_json::from_slice(&output.stdout).unwrap(); //Ответ из json в структуру
}
*/