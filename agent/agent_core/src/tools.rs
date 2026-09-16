use rig::{rig_tool}; //fn -> tool
use rig::tool::ToolExecutionError; //Ошибка для тулза
use tokio::process::Command; //Вызов внешних процессов
use std::process::Stdio;
use std::time::Instant; //Для общения с вызовами
use tokio::io::AsyncWriteExt; //Для записи в stdin процесса
use tokio::process::Child; //Запуск процесса как пиздюка
use serde::{Serialize, Deserialize}; //Для сборки разборки struct<->json
use serde_json::{Value, json}; //Json собранный
use std::process::Output; //Тип ответа

use crate::py_env::{get_py_env, PyFileModule}; //Py воскресенье для тузлов
use std::collections::HashMap; //Они кста тут живут  

//Для PyEnv
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFunction};
use pyo3::call::PyCallArgs;

#[derive(Serialize, Debug, Clone)]
struct GuardRequest //Заспрос в гвард
{
    tool: String,
    args: Value,
}

/*
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Access
{
    Deny = 0,
    Allow = 1,
}
*/

#[derive(Deserialize, Debug, Clone)]
struct GuardResponse
{
    allowed: bool,
    reason: String,
}

/*
#[derive(Serialize, Debug, Clone)]
struct ToolRequest
{
    name: String,
    args: Value,
}

#[derive(Deserialize, Debug, Clone)]
struct ToolResult
{
    success: bool,
    output: Option<Value>,
    error: Option<String>,
}
*/

//Пока ничего не значащий гвард
async fn tools_guard(toolname: String, args: Value) -> GuardResponse
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
async fn call_py_tool<A>(module_name: &str, func_name: &str, guard_args: Value, args: Option<A>, kwargs: Option<Py<PyDict>>) -> Result<Py<PyAny>, ToolExecutionError>
where //Тип аргумента
    for<'py> A: PyCallArgs<'py>,
{
    let time: Instant = Instant::now();
    print!("Tool {:?} called", func_name);

    let verdict: GuardResponse = tools_guard(func_name.to_owned(), guard_args).await; //Вызов гварда

    if !verdict.allowed //Можно?
    {
        println!(" | guard blocked | time - {:?}", time.elapsed());

        return Err(ToolExecutionError::permission_denied(verdict.reason)); //Нельзя
    }

    let py_env: &HashMap<String, PyFileModule> = get_py_env().await; //Получить вторник

    let func: &Py<PyFunction> = match py_env.get(module_name) //Функция
    {
        Some(module) => //Из модуля
        {
            match module.funcs.get(func_name) //Там лежит
            {
                Some(func) => { func }

                None =>
                {
                    println!(" | missing tool | time - {:?}", time.elapsed());

                    return Err(ToolExecutionError::not_found(format!("Tool {:?} in module {:?} is missing", func_name, module_name)));
                }
            }
        }

        None => 
        {
            println!(" | missing module | time - {:?}", time.elapsed());

            return Err(ToolExecutionError::not_found(format!("Module {:?} with tool {:?} is missing", module_name, func_name)));
        }
    };

    //Проверка кол-ва аргументов
    match args
    {
        Some(args) =>
        {
            match kwargs
            {
                None =>
                {
                    
                    return Python::attach(|py: Python<'_>| -> PyResult<Py<PyAny>> 
                    {
                        let ret: Result<Py<PyAny>, PyErr> = func.call1(py, args); //Вызов с args
                        
                        println!(" | called | time - {:?}", time.elapsed());

                        return ret;
                    }).map_err(ToolExecutionError::from_error); //Возврат её ошибок
                }

                Some(kwargs) =>
                {
                    return Python::attach(|py: Python<'_>| -> PyResult<Py<PyAny>> 
                    {
                        let ret: Result<Py<PyAny>, PyErr> = func.call(py, args, Some(kwargs.bind(py))); //Вызов с args + kwargs

                        println!(" | called | time - {:?}", time.elapsed());

                        return ret;
                    }).map_err(ToolExecutionError::from_error); //Возврат её ошибок                    
                }
            }
        }

        None =>
        {
            return Python::attach(|py: Python<'_>| -> PyResult<Py<PyAny>> 
            {
                let ret: Result<Py<PyAny>, PyErr> = func.call0(py); //Вызов без args

                println!(" | called | time - {:?}", time.elapsed());

                return ret;
            }).map_err(ToolExecutionError::from_error); //Возврат её ошибок            
        }
    }

}

#[rig_tool(description = "Write file.")]
pub async fn write_file(filename: String, text: String) -> Result<(), ToolExecutionError>
{
    //Вызов
    call_py_tool("files", "write_file", json!({ "filename": &filename, "text": &text }), Some((filename,text)), None).await?;

    //Сбора нет
    return Ok(());
}

#[rig_tool(description = "Read file.")]
pub async fn read_file(filename: String) -> Result<String, ToolExecutionError>
{
    //Вызов
    let res: Py<PyAny> = call_py_tool("files", "read_file", json!({ "filename": &filename }), Some((filename,)), None).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(description = "HTTP request.")]
pub async fn http_request(url: String, req_type: String, post_data: Option<HashMap<String, String>>, get_params: Option<HashMap<String, String>>) -> Result<String, ToolExecutionError>
{
    let guard_args: Value = json!({ "url": &url, "req_type": &req_type, "post_data": &post_data, "get_params": &get_params }); //json гварду

    let kwargs: Py<PyDict> = Python::attach(|py: Python<'_>| -> PyResult<Py<PyDict>> //Сбор kwargs
    {
        let kwargs: Bound<'_, PyDict> = PyDict::new(py);

        if let Some(data) = post_data
        {
            kwargs.set_item("post_data", data)?;
        }

        if let Some(params) = get_params
        {
            kwargs.set_item("get_params", params)?;
        }

        Ok(kwargs.unbind())
    }).map_err(ToolExecutionError::from_error)?;

    //Вызов
    let res: Py<PyAny> = call_py_tool("http_request", "make_request", guard_args, Some((url, req_type)), Some(kwargs)).await?;

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
    let res: Py<PyAny> = call_py_tool("dump_env", "dump_env", json!({ }), None::<()>, None).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}
