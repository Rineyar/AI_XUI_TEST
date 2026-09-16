use rig::client::AgentClientExt; //.agent метод
use rig::client::Client; //Тип данных для соединения
use rig::completion::Prompt; //.prompt метод
use rig::prelude::Agent; //Тип данных для агента
use rig::agent::AgentBuilder; //Тип для билдера
use rig::providers::{openai::CompletionsClient, openai::OpenAICompletionsExt}; //Для дипсика местного разлива

use std::mem; //Для take, чтобы по красоте
use std::time::Instant; //Таймер
use std::env::args; //Арги для выбора модели
use std::env::var; //Окружение для API ключа

use dotenvy::dotenv; //Крейт для удобного чтения .env;

mod settings; //Настройки ядра
use settings::*;

mod tools; //Инструменты
use tools::*;

mod py_env; //Py среда
use py_env::*;

/*
Обязательно сделать проверку tools call
А то эта херь имеет свойство выдумывать.
+ динамическую обработку бы
*/

//Сборка по докер cross +stable build --release --target x86_64-unknown-linux-gnu
//Если не может подсосать файлы, то $env:AGENT_ROOT = (Resolve-Path "..").Path

//После docker compose build --no-cache
//docker compose up --force-recreate
#[tokio::main] //Асинк рантайм - база
async fn main()
{
    let time_start: Instant = Instant::now();

    dotenv().ok(); //Чтобы он мон .env подсосать

    let mut args_list: Vec<String> = args().collect();

    if args_list.len() == 1
    {
        panic!("Укажите модель через -L или -D!");
    } else if args_list.len() > 2
    {
        println!("Обнаружены лишние аргументы:");

        for elem in args_list.iter().skip(2)
        {
            println!("{:?}", elem);
        }
    }

    let model_select: String = mem::take(&mut args_list[1]);

    drop(args_list);

    let agent_builder: AgentBuilder = match model_select.as_str()
    {
        "-L" =>
        {
            let model: Client<_> = Client::from_url(MODEL_LOCAL_URL).expect("Локальня модель недоступна"); //Получение по ссылке

            model.agent(MODEL_LOCAL_ID)
        }

        "-D" =>
        {
            let model: Client<OpenAICompletionsExt> = CompletionsClient::builder() //Сборка клиента
            .api_key(var("DEEPSEEK_LOCAL_API_KEY").expect("Отсутствует API ключ")) //Передать ключ
            .base_url(DEEPSEEK_LOCAL_URL) //Передать ссылку
            .build()
            .expect("Сборка разливного не удалась");

            model.agent(MODEL_DEEPSEEK_ID)
        }

        _ =>
        {
            panic!("Некорректный выбор модели!");
        }
    };

    println!("Client loaded - {:?}", time_start.elapsed());

    load_py_env().await; //Создание Py субботы

    println!("PyEnv loaded - {:?}", time_start.elapsed());

    let agent: Agent = agent_builder
    .preamble(FULL_PROMPT) //System prompt
    .tool(ToolSumI32) //Инструмент добавили
    .tool(ToolSumI64)
    .tool(ToolSubI64)
    .tool(ReadFile)
    .tool(WriteFile)
    .tool(HttpRequest)
    .default_max_turns(MAX_LLM_CALLS) //Максимум обращений к модели
    .build(); //Builder -> Agent построить короче

    println!("Agent builded - {:?}", time_start.elapsed());

    let response: String = agent
    .prompt("
    Do GET request to https://deepcode.ci.nsu.ru/api/models and write result to get.txt
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