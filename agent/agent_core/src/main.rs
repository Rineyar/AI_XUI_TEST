use rig::client::AgentClientExt; //.agent метод
use rig::client::Client; //Тип данных для соединения
use rig::completion::Prompt; //.prompt метод
use rig::rig_tool; //fn -> tool
use rig::tool::ToolExecutionError; //Ошибка для тулза
use rig::providers::llamafile::LlamafileExt; // Тип клиента
use rig::prelude::Agent; //Тип данных для агента

use std::time::Instant; //Таймер

const MAX_LLM_CALLS: usize = 2; //Максимум вывовом модели

#[rig_tool(description = "Return a sum of 2 32bit numbers")] //Макрос для обёртки функции в инструмент
async fn tool_sum(a: i32, b: i32) -> Result<i32, ToolExecutionError> 
{
    return Ok(a.saturating_add(b)); //При переполнении не вылетит, а останется на максимуме
}

#[tokio::main] //Асинк рантайм - база
async fn main()
{
    let time_start: Instant = Instant::now();

    let model: Client<LlamafileExt> = Client::from_url("http://localhost:1234").expect("Не грузит по ссылке"); //Получение по ссылке

    let agent: Agent = model
    .agent("openai/gpt-oss-20b") //Получаем агента по id
    .preamble("Try to tell sth") //System prompt
    .tool(ToolSum) //Инструмент добавили
    .default_max_turns(MAX_LLM_CALLS) //Максимум обращений к модели
    .build(); //Builder -> Agent построить короче

    let response: String = agent
    .prompt("Sum two 32-bit integers of your choice. You must use the tool.") //Запрос
    .await
    .expect("Не отвечает");

    println!("{}\n{:?}", response, time_start.elapsed());

    /* //Потробный ответ от агента
    use rig::agent::PromptResponse;
    let response: PromptResponse = agent
    .prompt("Sum two 32-bit integers of your choice. You must use the tool.")
    .extended_details()
    .await
    .expect("Не отвечает");

    println!("{:?}\n{:?}\n{:?}", response.content, response.usage, time_start.elapsed());
    */

}