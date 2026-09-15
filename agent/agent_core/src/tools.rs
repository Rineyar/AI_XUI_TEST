use rig::rig_tool; //fn -> tool
use rig::tool::ToolExecutionError; //Ошибка для тулза
use tokio::process::Command; //Вызов внешних процессов

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

//Читает файл через py скрипт. 
//На будущее возвращать в LLM короткую версию ошибки
#[rig_tool(description = "Read file.")]
pub async fn read_file(filename: String) -> Result<String, ToolExecutionError>
{
    match Command::new("python") //Вызов пыхтуна
    .arg("./test.py") //Файл
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