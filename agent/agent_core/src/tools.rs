use rig::{rig_tool}; //fn -> tool
use rig::tool::ToolExecutionError; //Ошибка для тулза
use tokio::process::Command; //Вызов внешних процессов
use std::process::Stdio; //Для общения с вызовами
use tokio::io::AsyncWriteExt; //Для записи в stdin процесса
use tokio::process::Child; //Запуск процесса как пиздюка
use serde::{Serialize, Deserialize}; //Для сборки разборки struct<->json
use serde_json::{Value, json}; //Json собранный
use std::process::Output; //Тип ответа

use crate::py_env::{get_py_env, PyFileModule}; //Py воскресенье для тузлов
use std::collections::HashMap; //Они кста тут живут  

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

/*
//Читает файл через py скрипт. 
//На будущее возвращать в LLM короткую версию ошибки
#[rig_tool(description = "Read file.")]
pub async fn read_file(filename: String) -> Result<String, ToolExecutionError>
{
    let verdict: GuardResponse = tools_guard(String::from("read_file"), json!({ "filename": filename })).await;

    if !verdict.allowed
    {
        return Err(ToolExecutionError::permission_denied(verdict.reason));
    }

    match Command::new("python") //Вызов пыхтуна
    .arg("../tools/tools_py/read_all_file.py") //Файл
    .arg(&filename) //Аргумент
    .output() //Сбор того, что тот выведет
    .await //Асинк, че сказать
    {
        Ok(out) => //Прочитал
        {
            if !out.status.success() //Успех?
            {
                return Err(ToolExecutionError::other(String::from_utf8(out.stderr).unwrap())); //Не успех
            }

            return Ok(String::from_utf8(out.stdout).unwrap()); //Успех
        }

        Err(err) => //Ошибка вызова
        {
            return Err(ToolExecutionError::from_error(err));
        }
    }    
}
*/

//Тест-зона
use pyo3::prelude::*;
use pyo3::types::PyFunction;

#[rig_tool(description = "Write file.")]
pub async fn write_file(filename: String, text: String) -> Result<(), ToolExecutionError>
{
    let verdict: GuardResponse = tools_guard(String::from("write_file"), json!({ "filename": filename, "text": text })).await;

    if !verdict.allowed
    {
        return Err(ToolExecutionError::permission_denied(verdict.reason));
    }

    let py_env: &HashMap<String, PyFileModule> = get_py_env().await;

    let func = py_env.get("files").unwrap().funcs.get("write_file").unwrap();

    return Python::attach(|py: Python<'_>| -> PyResult<()>
    {
        func.call1(py, (filename, text))?;

        Ok(())
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(description = "Read file.")]
pub async fn read_file(filename: String) -> Result<String, ToolExecutionError>
{
    let verdict: GuardResponse = tools_guard(String::from("read_file"), json!({ "filename": filename })).await;

    if !verdict.allowed
    {
        return Err(ToolExecutionError::permission_denied(verdict.reason));
    }

    let py_env: &HashMap<String, PyFileModule> = get_py_env().await;

    let func: &Py<PyFunction> = py_env.get("files").unwrap().funcs.get("read_file").unwrap();

    return Python::attach(|py: Python<'_>| -> PyResult<String>
    {
        let func: &Bound<'_, PyFunction> = func.bind(py);
        let result: Bound<'_, PyAny> = func.call1((filename,))?;

        result.extract()
    })
    .map_err(ToolExecutionError::from_error);
}

/*
use pyo3::prelude::*;
use pyo3::types::PyModule;
use pyo3::ffi::c_str;

//Пишет текст в файл через ну как бы .py скрипт
#[rig_tool(description = "Write file.")]
pub async fn write_file(filename: String, text: String) -> Result<(), ToolExecutionError>
{
    Python::attach(|py: Python<'_>| -> PyResult<()> //По факту замыкание с возвращаемым типом
    {
        let module: Bound<'_, PyModule> = PyModule::from_code(py, //Сбор файла из кода
        c_str!(include_str!("../../tools/tools_py/write_all_file.py")), //Код
        c"write_file.py", c"write_file")?; //Имя файла и имя модуля

        let function: Bound<'_, PyAny> = module.getattr("write_file")?; //Определение функции из модуля

        function.call1((filename, text))?; //Вызов функции

        Ok(()) //Py отработал
    }).map_err(ToolExecutionError::from_error) //Если не отработал, то каждый PyErr от ? обернётся в TEE и отправится модельке
}
*/