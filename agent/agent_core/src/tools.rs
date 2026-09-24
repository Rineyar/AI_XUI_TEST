use rig::{rig_tool}; //fn -> tool
use rig::tool::ToolExecutionError;//Ошибка для тулза

use serde::Serialize; //Сбор в json
use serde_json::{Value, json, Map}; //Json собранный
use serde_pyobject::to_pyobject; 

use std::time::Instant; //Для таймера
use std::collections::HashMap; //Они кста тут живут  

use tracing::{error, info, warn}; //Логи

use crate::py_env::{get_py_env, PyFileModule}; //Py воскресенье для тузлов
use crate::guards::{GuardResponse, tools_guard}; //Гварды

//Для PyEnv
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFunction};

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

fn insert_option_arg<T>(args: &mut Map<String, Value>, key: &str, value: Option<T>)
where
    T: Serialize
{
    if let Some(value) = value
    {
        args.insert(key.to_string(), json!(value));
    }
}

//Обёртка вызовов
async fn call_py_tool(request: ToolRequest) -> Result<Py<PyAny>, ToolExecutionError>
{
    let time: Instant = Instant::now();

    info!("\nTool {:?} called with args: {:?}\t|\t{:?}", request.function, request.args, time.elapsed());

    let (verdict, request): (GuardResponse, ToolRequest) = tools_guard(request).await; //Вызов гварда

    if !verdict.allowed //Можно?
    {
        warn!("\nVerdict: guard blocked: {:?}\t|\t{:?}", verdict.reason, time.elapsed());

        return Err(ToolExecutionError::permission_denied(verdict.reason)); //Нельзя
    }

    info!("\nVerdict: allow: {:?}\t|\t{:?}", verdict.reason, time.elapsed());

    let py_env: &HashMap<String, PyFileModule> = get_py_env(); //Получить вторник

    let func: &Py<PyFunction> = match py_env.get(request.module) //Функция
    {
        Some(module) => //Из модуля
        {
            match module.funcs.get(request.function) //Там лежит
            {
                Some(func) => { func }

                None =>
                {
                    error!("\nMissing tool - {:?}\t|\t{:?}", request.function, time.elapsed());

                    return Err(ToolExecutionError::not_found(format!("Tool {:?} in module {:?} is missing", request.function, request.module)));
                }
            }
        }

        None => 
        {
            error!("\nMissing module - {:?}\t|\t{:?}", request.module, time.elapsed());

            return Err(ToolExecutionError::not_found(format!("Module {:?} with tool {:?} is missing", request.module, request.function)));
        }
    };

    return tokio::task::spawn_blocking(move || -> Result<Py<PyAny>, ToolExecutionError>
    {
        return Python::attach(|py| -> PyResult<Py<PyAny>>
        {
            let kwargs: Bound<'_, PyDict> = to_pyobject(py, &request.args)?.cast_into()?;

            let ret: Result<Py<PyAny>, PyErr> = if kwargs.is_empty()
            {
                func.call0(py)
            } else {
                func.call(py, (), Some(&kwargs))
            };

            match ret
            {
                Ok(_) => { info!("\nCalled\t|\t{:?}", time.elapsed()); }
                
                Err(_) => { error!("\nError returned - {:?}\t|\t{:?}", ret, time.elapsed()); }
            }

            return ret;
        }).map_err(ToolExecutionError::from_error);
    }).await.expect("Tool thread joining error");
}

#[rig_tool(
    name = "write_file",
    description = "Write text to a file.",
    params(
        filename = "Path to the file relative",
        text = "Text content to write to the file."
    )
)]
pub async fn write_file(filename: String, text: String) -> Result<(), ToolExecutionError>
{
    //Вызов
    call_py_tool(ToolRequest { module: "files", function: "write_file", args: json!({ "filename": filename, "text": text }) }).await?;

    //Сбора нет
    return Ok(());
}

#[rig_tool(
    name = "read_file",
    description = "Read the contents of a text file from the workspace.",
    params(
        filename = "Path to the file relative to the workspace."
    )
)]
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

#[rig_tool(
    name = "http_request",
    description = "Perform an HTTP or HTTPS GET or POST request.",
    params(
        url = "Target HTTP or HTTPS URL.",
        req_type = "Request method: get or post.",
        post_data = "Optional request body for POST requests.",
        get_params = "Optional query parameters for GET requests."
    )
)]
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

#[rig_tool(
    name = "dump_env",
    description = "Return available environment variables."
)]
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

#[rig_tool(
    name = "find_files",
    description = "Recursively find files and directories in the workspace that match a glob pattern.",
    params(
        pattern = "Glob pattern to match file or directory names, for example '*.py' or 'config*'.",
        path = "Directory to search from, relative to the workspace."
    )
)]
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

#[rig_tool(
    name = "directory_contents",
    description = "List files and directories contained directly in a workspace directory.",
    params(
        path = "Path to the directory relative to the workspace."
    )
)]
pub async fn directory_contents(path: String) -> Result<String, ToolExecutionError>
{
    //Вызов
    let res: Py<PyAny> = call_py_tool( ToolRequest { module: "directory_contents", function: "directory_contents",
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
