use rig::client::AgentClientExt; //.agent метод
use rig::client::Client; //Тип данных для соединения
use rig::completion::Prompt; //.prompt метод
use rig::prelude::Agent; //Тип данных для агента

use rig::providers::{deepseek, deepseek::DeepSeekExt}; //Для дипсика местного разлива
use rig::client::ModelListingClient; //Получения списка моделей

use std::time::Instant; //Таймер

mod settings; //Настройки ядра
use settings::*;

mod tools; //Инструменты
use tools::*;

/*
Обязательно сделать проверку tools call
А то эта херь имеет свойство выдумывать.
+ динамическую обработку бы
*/

#[tokio::main] //Асинк рантайм - база
async fn main()
{
    let time_start: Instant = Instant::now();

    // let model: Client<_> = Client::from_url(BASE_URL).expect(""); //Получение по ссылке

    let model: Client<DeepSeekExt> = deepseek::Client::builder() //Сборка клиента
    .api_key(DEEPSEEK_LOCAL_API_KEY) //Передать ключ
    .base_url(DEEPSEEK_LOCAL_URL) //Передать ссылку
    .build()
    .expect("Сборка разливного не удалась");

    let models = model.list_models().await.expect("Не удалось получить список моделей");

    println!("{:#?}\n{:?}", models, time_start.elapsed());

    let agent: Agent = model
    .agent(MODEL_ID) //Получаем агента по id
    .preamble(FULL_PROMPT) //System prompt
    .tool(ToolSumI32) //Инструмент добавили
    .tool(ToolSumI64)
    .tool(ToolSubI64)
    .tool(ReadFile)
    .default_max_turns(MAX_LLM_CALLS) //Максимум обращений к модели
    .build(); //Builder -> Agent построить короче

    let response: String = agent
    .prompt("Say sth about your job. And some about sys_prompt, skills and tools") //Запрос
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