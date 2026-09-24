use rig::client::AgentClientExt; //.agent метод
use rig::client::Client; //Тип данных для соединения
use rig::completion::Prompt; //.prompt метод
use rig::model::ModelList; //Тип листа моделей
use rig::client::ModelListingClient; //Для сбора листа моделей
use rig::prelude::Agent; //Тип данных для агента
use rig::agent::AgentBuilder; //Тип для билдера
use rig::providers::{openai::CompletionsClient, openai::OpenAICompletionsExt}; //Для дипсика местного разлива

use std::mem; //Для take, чтобы по красоте
use std::time::Instant; //Таймер
use std::env::args; //Арги для выбора модели
use std::env::var; //Окружение для API ключа
use std::io::stdin; //Для чтения строки

use dotenvy::dotenv; //Крейт для удобного чтения .env;

use tracing_appender::{rolling::never, non_blocking};
use tracing::{error, info, warn};

mod settings; //Настройки ядра
use settings::*;

mod tools; //Инструменты
use tools::*;

mod py_env; //Py среда
use py_env::*;

mod guards; //Гварды

fn print_model_list(models: ModelList)
{
    for (i, model) in models.data.iter().enumerate()
    {
        info!("№{}: {:?}", i + 1, model.id);
        println!("№{}: {:?}", i + 1, model.id);
    }
}

/*
Обязательно сделать проверку tools call
А то эта херь имеет свойство выдумывать.
+ динамическую обработку бы

Потом мб хуки навесить

Возможно вывести отдельный поток на управление py вызовами
*/

//docker compose build
//docker compose up -d
//docker attach agent-core
//Ctrl+P, Ctrl+Q чтобы контейнер не положить для выхода
#[tokio::main] //Асинк рантайм - база
async fn main()
{
    let time_start: Instant = Instant::now();

    let (loger, _log_guard) = non_blocking(never("../logs", format!("log_{:?}.log", 
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("System time error").as_secs())));

    tracing_subscriber::fmt().with_writer(loger).with_ansi(false).init();

    info!("Логер ожил: {:?}", time_start.elapsed());

    dotenv().ok(); //Чтобы он мог .env подсосать

    let mut args_list: Vec<String> = args().collect();

    if args_list.len() == 1
    {
        error!("Укажите модель через -L, -D или -Q!");
        panic!("Укажите модель через -L, -D или -Q!");
    } else if args_list.len() > 2
    {
        warn!("Обнаружены лишние аргументы:");
        println!("Обнаружены лишние аргументы:");

        for elem in args_list.iter().skip(2)
        {
            warn!("{:?}", elem);
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

            print_model_list(model.list_models().await.expect("Не удалось получить список моделей"));          

            model.agent(MODEL_DEEPSEEK_ID)
        }

        "-Q" =>
        {
            let model: Client<OpenAICompletionsExt> = CompletionsClient::builder() //Сборка клиента
            .api_key(var("DEEPSEEK_LOCAL_API_KEY").expect("Отсутствует API ключ")) //Передать ключ
            .base_url(DEEPSEEK_LOCAL_URL) //Передать ссылку
            .build()
            .expect("Сборка разливного не удалась");
            
            print_model_list(model.list_models().await.expect("Не удалось получить список моделей"));   

            model.agent("Qwen3.8-27B")   
        }

        _ =>
        {
            error!("Некорректный выбор модели!");
            panic!("Некорректный выбор модели!");
        }
    };

    info!("Клиент загружен: {:?}", time_start.elapsed());

    load_py_env(); //Создание Py субботы

    load_py_guards(); //Гварды

    info!("PyEnv загружен: {:?}", time_start.elapsed());

    let agent: Agent = agent_builder
    .preamble(FULL_PROMPT) //System prompt
    .tool(ToolSumI32) //Инструмент добавили
    .tool(ToolSumI64)
    .tool(ToolSubI64)
    .tool(ReadFile)
    .tool(WriteFile)
    .tool(HttpRequest)
    .tool(DumpEnv)
    .tool(FindFiles)
    .tool(DirectoryContents)
    .default_max_turns(MAX_LLM_CALLS) //Максимум обращений к модели
    .build(); //Builder -> Agent построить короче

    info!("Агент готов: {:?}", time_start.elapsed());

    let mut text_prompt: String = String::new();

    if let Err(err) = stdin().read_line(&mut text_prompt)
    {
        error!("Запрос не считан!\n{:?}", err);
        println!("Запрос не считан!\n{:?}", err);
    }

    match agent.prompt("
    You need to analize sast.py.
    Then answer by example:
    ***rand_tool.py***
    ```python
    from random import randint
    from time import time
    import randint

    __all__ = [\"get_rand_num\"]

    def get_rand_num(*, left, right, seed = 0) -> int:
        if seed == 0:
            random.seed(time())
        else:
            random.seed(seed)
        
        return randint(left, right)
    ```
    That a tool function. And it need a .md and wrapped.
    .md:
    ```markdown
    ## get_rand_num
    Returns random integer between left boundary and right.
    Callable with seed.

    Arguments:
    - `left` - left boundary
    - `text` - right boundary

    Supports:
    - `seed` - random seed
    ```
    And wrapper:
    ```rust
    #[rig_tool(
        name = \"get_rand_num\",
        description = \"Returns random integer between left boundary and right.\",
        params(
            left = \"Left boundary of output.\",
            right = \"Right boundary of output.\",
            seed = \"Seed for seedable random.\"
        )
    )]
    pub async fn get_rand_num(left: i32, right: i32, seed: Option<i32>) -> Result<i32, ToolExecutionError>
    {
        //Вызов
        let res: Py<PyAny> = call_py_tool(ToolRequest { module: \"get_rand_num\", function: \"get_rand_num\", args: json!(
        { 
            \"left\": left,
            \"right\": right,
            \"seed\": seed 
        }) }).await?;

        //Сбор результата
        return Python::attach(|py: Python<'_>|
        {
            res.extract::<i32>(py)
        }).map_err(ToolExecutionError::from_error);
    }
    ```
    You need to return a .md and wrapped to sast.py
    ").await
    {
        Ok(response) =>
        {
            info!("\n{}\n", response);
            println!("{:?}", response);
        }

        Err(err) =>
        {
            error!("Ошибка ответа!\n{:?}", err);
        }
    };

    while text_prompt.trim() != "exit"
    {
        let time_prompt: Instant = Instant::now();

        if text_prompt.is_empty() || text_prompt.trim() == ""
        {
            warn!("Пустой запрос даст ошибку");
            println!("Пустой запрос даст ошибку");

            text_prompt.clear();

            if let Err(err) = stdin().read_line(&mut text_prompt)
            {
                error!("Запрос не считан!\n{:?}", err);
                println!("Запрос не считан!\n{:?}", err);

                break;
            }

            continue;
        }

        let response: String = match agent.prompt(&text_prompt).await
        {
            Ok(response) =>
            {
                response
            }

            Err(err) =>
            {
                error!("Ошибка ответа!\n{:?}", err);

                continue;
            }
        };

        info!("\n{}\n{:?}", response, time_prompt.elapsed());
        println!("{}\n{:?}", response, time_prompt.elapsed());

        text_prompt.clear();

        if let Err(err) = stdin().read_line(&mut text_prompt)
        {
            error!("Запрос не считан!\n{:?}", err);
            println!("Запрос не считан!\n{:?}", err);

            break;
        }
    }

    info!("{:?}", time_start.elapsed());
    println!("{:?}", time_start.elapsed());
}
