use rig::client::AgentClientExt; //.agent метод
use rig::client::Client; //Тип данных для соединения
use rig::completion::Prompt; //.prompt метод
use rig::rig_tool; //fn -> tool
use rig::tool::ToolExecutionError; //Ошибка для тулза
use rig::providers::llamafile::LlamafileExt; // Тип клиента
use rig::prelude::Agent; //Тип данных для агента
use tokio::process::Command; //Вызов внешних процессов

use std::time::Instant; //Таймер

const MAX_LLM_CALLS: usize = 32; //Максимум вывовов модели

//Макрос для обёртки функции в инструмент
#[rig_tool(description = "Add two signed 32-bit integers. Both operands AND their mathematical sum must fit in signed 32-bit range.")]
async fn tool_sum_i32(a: i32, b: i32) -> Result<i32, ToolExecutionError> 
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
async fn tool_sum_i64(a: i64, b: i64) -> Result<i64, ToolExecutionError> 
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
async fn tool_sub_i64(a: i64, b: i64) -> Result<i64, ToolExecutionError> 
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
async fn read_file(filename: String) -> Result<String, ToolExecutionError>
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

/*
Обязательно сделать проверку tools call
А то эта херь имеет свойство выдумывать.
+ динамическую обработку бы
*/

#[tokio::main] //Асинк рантайм - база
async fn main()
{
    let time_start: Instant = Instant::now();

    let model: Client<LlamafileExt> = Client::from_url("http://localhost:1234").expect("Не грузит по ссылке"); //Получение по ссылке

    let agent: Agent = model
    .agent("openai/gpt-oss-20b") //Получаем агента по id
    .preamble("API TESTING\nDo prompts\nYOU MUST USE TOOLS") //System prompt
    .tool(ToolSumI32) //Инструмент добавили
    .tool(ToolSumI64)
    .tool(ToolSubI64)
    .tool(ReadFile)
    .default_max_turns(MAX_LLM_CALLS) //Максимум обращений к модели
    .build(); //Builder -> Agent построить короче

    let response: String = agent
    .prompt("
    Read value from 1.txt. Then read value from 2.txt. Take sum of this 2 values. Then read value from 3.txt and return sub of sum and 3rd value.
    ") //Запрос
    .await
    .expect("Не отвечает");

    println!("{}\n{:?}", response, time_start.elapsed());
}

    /* //Подробный ответ от агента
    use rig::agent::PromptResponse;
    let response: PromptResponse = agent
    .prompt("Sum two 32-bit integers of your choice. You must use the tool.")
    .extended_details()
    .await
    .expect("Не отвечает");

    println!("{:?}\n{:?}\n{:?}", response.content, response.usage, time_start.elapsed());
    */