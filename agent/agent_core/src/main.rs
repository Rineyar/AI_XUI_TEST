use rig::client::AgentClientExt; //.agent метод
use rig::client::Client; //Тип данных для соединения
use rig::completion::Prompt; //.prompt метод
use rig::rig_tool; //fn -> tool
use rig::tool::ToolExecutionError; //Ошибка для тулза
use rig::providers::llamafile::LlamafileExt; // Тип клиента
use rig::prelude::Agent; //Тип данных для агента

use std::time::Instant; //Таймер

const MAX_LLM_CALLS: usize = 32; //Максимум вывовов модели

#[rig_tool(description = "Add two signed 32-bit integers. Both operands AND their mathematical sum must fit in signed 32-bit range.")] //Макрос для обёртки функции в инструмент
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

#[rig_tool(description = "Add two signed 64-bit integers. Both operands AND their mathematical sum must fit in signed 64-bit range.")] //Макрос для обёртки функции в инструмент
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
            return Err(ToolExecutionError::invalid_args("Signed i64 overflow: a + b must be between -2305843009213693952 and 2305843009213693951. Choose different operands."));
        }
    }
}

#[rig_tool(description = "Sub two signed 64-bit integers. Both operands AND their mathematical subtract must fit in signed 64-bit range.")] //Макрос для обёртки функции в инструмент
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
    .default_max_turns(MAX_LLM_CALLS) //Максимум обращений к модели
    .build(); //Builder -> Agent построить короче

    let response: String = agent
    .prompt("
    Add 2 32‑bit numbers of your choice. The result should also be 32‑bit. 
    Repeat the same with the other numbers. Add these 2 resulting numbers. 
    When choosing the first 4 numbers, make sure the result of their final sums is 64‑bit. 
    Then repeat these operations, and subtract the first number from the second 64‑bit number. 
    Return the result and the operations performed.\n
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