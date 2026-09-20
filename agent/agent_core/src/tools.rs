use rig::{rig_tool}; //fn -> tool
use rig::tool::ToolExecutionError;//Ошибка для тулза
use serde::Serialize; //Сбор в json

use serde_pyobject::to_pyobject; 
use std::time::Instant;

use serde_json::{Value, json}; //Json собранный

use crate::py_env::{get_py_env, PyFileModule}; //Py воскресенье для тузлов
use std::collections::HashMap; //Они кста тут живут  

//Для PyEnv
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFunction};

use crate::guards::{GuardResponse, tools_guard}; //Гварды

#[derive(Serialize, Clone, Debug)]
pub struct ToolRequest
{
    pub module: &'static str,
    pub function: &'static str,
    pub args: Value
}

//Макрос для обёртки функции в инструмент
#[rig_tool(description = "Add two signed 32-bit integers. Both operands AND their mathematical sum must fit in signed 32-bit range.")]
pub async fn tool_sum_i32(a: i32, b: i32) -> Result<i32, ToolExecutionError> 
{
    match a.checked_add(b)
    {
        Some(res) =>
        {
            return Ok(res);
        }

        None =>
        {
            return Err(ToolExecutionError::invalid_args("Signed i32 overflow: a + b must be between -2147483648 and 2147483647. Choose different operands."));
        }
    }
}

#[rig_tool(description = "Add two signed 64-bit integers. Both operands AND their mathematical sum must fit in signed 64-bit range.")]
pub async fn tool_sum_i64(a: i64, b: i64) -> Result<i64, ToolExecutionError> 
{
    match a.checked_add(b)
    {
        Some(res) =>
        {
            return Ok(res);
        }

        None =>
        {
            return Err(ToolExecutionError::invalid_args("Signed i64 overflow"));
        }
    }
}

#[rig_tool(description = "Sub two signed 64-bit integers. Both operands AND their mathematical subtract must fit in signed 64-bit range.")]
pub async fn tool_sub_i64(a: i64, b: i64) -> Result<i64, ToolExecutionError> 
{
    match a.checked_sub(b)
    {
        Some(res) =>
        {
            return Ok(res);
        }

        None =>
        {
            return Err(ToolExecutionError::invalid_args("Signed i64 overflow"));
        }
    }
}

//Обёртка вызовов
async fn call_py_tool(request: ToolRequest) -> Result<Py<PyAny>, ToolExecutionError>
{
    let time: Instant = Instant::now();
    print!("Tool {:?} called with args: {:?}", request.function, request.args);

    let verdict: GuardResponse = tools_guard(&request).await; //Вызов гварда

    if !verdict.allowed //Можно?
    {
        println!(" | guard blocked | time - {:?}", time.elapsed());

        return Err(ToolExecutionError::permission_denied(verdict.reason)); //Нельзя
    }

    let py_env: &HashMap<String, PyFileModule> = get_py_env().await; //Получить вторник

    let func: &Py<PyFunction> = match py_env.get(request.module) //Функция
    {
        Some(module) => //Из модуля
        {
            match module.funcs.get(request.function) //Там лежит
            {
                Some(func) => { func }

                None =>
                {
                    println!(" | missing tool | time - {:?}", time.elapsed());

                    return Err(ToolExecutionError::not_found(format!("Tool {:?} in module {:?} is missing", request.function, request.module)));
                }
            }
        }

        None => 
        {
            println!(" | missing module | time - {:?}", time.elapsed());

            return Err(ToolExecutionError::not_found(format!("Module {:?} with tool {:?} is missing", request.module, request.function)));
        }
    };

    return Python::attach(|py| -> PyResult<Py<PyAny>>
    {
        let kwargs: Bound<'_, PyDict> = to_pyobject(py, &request.args)?.cast_into()?;

        let ret: Result<Py<PyAny>, PyErr> = if kwargs.is_empty()
        {
            func.call0(py)
        } else {
            func.call(py, (), Some(&kwargs))
        };

        println!(" | called | time - {:?}", time.elapsed());

        return ret;
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(description = "Write file.")]
pub async fn write_file(filename: String, text: String) -> Result<(), ToolExecutionError>
{
    //Вызов
    call_py_tool(ToolRequest { module: "files", function: "write_file", args: json!({ "filename": filename, "text": text }) }).await?;

    //Сбора нет
    return Ok(());
}

#[rig_tool(description = "Read file.")]
pub async fn read_file(filename: String) -> Result<String, ToolExecutionError>
{
    //Вызов
    let res: Py<PyAny> = call_py_tool(ToolRequest { module: "files", function: "read_file", args: json!({ "filename": filename }) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(description = "HTTP request.")]
pub async fn http_request(url: String, req_type: String, post_data: Option<HashMap<String, String>>, get_params: Option<HashMap<String, String>>) -> Result<String, ToolExecutionError>
{
    //Вызов
    let res: Py<PyAny> = call_py_tool(ToolRequest { module: "http_request", function: "make_request", args: json!(
    {
        "url": url,
        "req_type": req_type,
        "post_data": post_data,
        "get_params": get_params
    }) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>| -> PyResult<String>
    {
        res.extract::<String>(py)
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(description = "Dump env.")]
pub async fn dump_env() -> Result<String, ToolExecutionError>
{
    //Вызов
    let res: Py<PyAny> = call_py_tool( ToolRequest { module: "dump_env", function: "dump_env", args: json!({ }) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(description = "Find files")]
pub async fn find_files(pattern: String, path: String) -> Result<String, ToolExecutionError>
{
    //Вызов
    let res: Py<PyAny> = call_py_tool( ToolRequest { module: "find_file", function: "find_files", args: json!(
    { 
        "pattern": pattern,
        "path": path
    }) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(description = "Directory contents.")]
pub async fn directory_contents(path: String) -> Result<String, ToolExecutionError>
{
    //Вызов
    let res: Py<PyAny> = call_py_tool( ToolRequest { module: "directory_contents", function: "find_fildirectory_contentses",
    args: json!(
    { 
        "path": path
    }) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}
