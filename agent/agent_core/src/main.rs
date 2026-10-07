use rig::client::AgentClientExt; //.agent метод
use rig::client::Client; //Тип данных для соединения
use rig::completion::Prompt; //.prompt метод
use rig::prelude::Agent; //Тип данных для агента
use rig::agent::AgentBuilder; //Тип для билдера
use rig::providers::{openai::CompletionsClient, openai::OpenAICompletionsExt}; //Для дипсика местного разлива
use rig::agent::PromptResponse; //Скрытый тип подробного ответа

use std::mem; //Для take, чтобы по красоте
use std::time::Instant; //Таймер
use std::env::args; //Арги для выбора модели
use std::env::var; //Окружение для API ключа
use std::thread; //Сбор результата с потока

use dotenvy::dotenv; //Крейт для удобного чтения .env;

use tracing_appender::{rolling::never, non_blocking}; //Логи
use tracing::{error, info, warn}; //Макросы логирования

mod settings; //Настройки ядра
use settings::*;

mod tools; //Инструменты
use tools::*;

mod py_env; //Py среда
use py_env::*;

mod guards; //Гварды

mod io; //Консоль
use io::*;

/*
Обязательно сделать проверку tools call
А то эта херь имеет свойство выдумывать.
+ динамическую обработку бы

Потом мб хуки навесить

Возможно вывести отдельный поток на управление py вызовами

Сделать проверку того, что в лог пихается. (прямо сейчас он запихал в лог весь бинарник, т.к. не смог его прочитать)
Что-то похожее уже возникало раньше...
*/

//docker compose build
//docker compose up -d
//docker attach agent-core
//Ctrl+P, Ctrl+Q чтобы контейнер не положить для выхода
//docker compose run --rm agent -L тест на локалке
//docker compose run --rm agent -O
#[tokio::main] //Асинк рантайм - база
async fn main()
{
    let time_start: Instant = Instant::now();

    //Чтобы при вылете терминал отпустил
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info|
    {
        restore_terminal();
        default_panic(info);
    }));

    //Логер
    let (loger, _log_guard) = non_blocking(never("./logs", format!("log_{:?}.log", 
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("Времени нет").as_secs())));

    tracing_subscriber::fmt().with_writer(loger).with_ansi(false).init();

    info!("Логер ожил: {:?}", time_start.elapsed());

    let (prompt_tx, mut prompt_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    //Создать потоки
    let console_thread: thread::JoinHandle<()> = spawn_console_thread(prompt_tx);

    dotenv().ok(); //Чтобы он мог .env подсосать

    let mut args_list: Vec<String> = args().collect();

    if args_list.len() == 1
    {
        error!("Укажите модель через -L, -D, -Q или -O!");
        panic!("Укажите модель через -L, -D, -Q или -O!");
    } else if args_list.len() > 2
    {
        warn!("Обнаружены лишние аргументы:");
        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
        Out::Text(format!("Обнаружены лишние аргументы:")))
        .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
        .expect("Ошибка связи с консолью");

        for elem in args_list.iter().skip(2)
        {
            warn!("{:?}", elem);
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(format!("{:?}", elem)))
            .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Ошибка связи с консолью");
        }
    }

    let model_select: String = mem::take(&mut args_list[1]);

    drop(args_list);

    //Основа агента
    let agent_builder: AgentBuilder = match model_select.as_str()
    {
        "-L" =>
        {
            let model: Client<_> = Client::from_url(MODEL_LOCAL_URL).inspect_err(|err|
            error!("Локальня модель недоступна - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Локальня модель недоступна"); //Получение по ссылке

            model.agent(MODEL_LOCAL_ID)
        }

        "-D" =>
        {
            let model: Client<OpenAICompletionsExt> = CompletionsClient::builder() //Сборка клиента
            .api_key(var("DEEPSEEK_LOCAL_API_KEY").inspect_err(|err|
            error!("Отсутствует API ключ - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Отсутствует API ключ")) //Передать ключ
            .base_url(DEEPSEEK_LOCAL_URL) //Передать ссылку
            .build().inspect_err(|err|
            error!("Сборка разливного не удалась - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Сборка разливного не удалась");

            print_model_list(&model, &time_start).await;

            model.agent(MODEL_DEEPSEEK_ID)
        }

        "-Q" =>
        {
            let model: Client<OpenAICompletionsExt> = CompletionsClient::builder() //Сборка клиента
            .api_key(var("DEEPSEEK_LOCAL_API_KEY").inspect_err(|err|
            error!("Отсутствует API ключ - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Отсутствует API ключ")) //Передать ключ
            .base_url(DEEPSEEK_LOCAL_URL) //Передать ссылку
            .build().inspect_err(|err|
            error!("Сборка разливного не удалась - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Сборка разливного не удалась");

            print_model_list(&model, &time_start).await;

            model.agent("Qwen3.8-27B")   
        }

        "-O" =>
        {
            let model: Client<OpenAICompletionsExt> = CompletionsClient::builder()
            .api_key(var("OPENROUTER_API_KEY").inspect_err(|err|
            error!("Отсутствует OPENROUTER_API_KEY - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Отсутствует OPENROUTER_API_KEY"))
            .base_url("https://openrouter.ai/api/v1")
            .build().inspect_err(|err|
            error!("Не удалось подключиться к OpenRouter - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Не удалось подключиться к OpenRouter");

            // Популярная стабильная бесплатная модель (можно поменять на "qwen/qwen-2.5-coder-32b-instruct:free")
            model.agent("openrouter/auto")
        }

        _ =>
        {
            error!("Некорректный выбор модели!");
            panic!("Некорректный выбор модели!");
        }
    };

    info!("Клиент загружен: {:?}", time_start.elapsed());
    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
    Out::Text(format!("Клиент загружен: {:?}", time_start.elapsed())))
    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
    .expect("Ошибка связи с консолью");

    load_py_env(); //Создание Py субботы

    load_py_guards(); //Гварды

    info!("PyEnv загружен: {:?}", time_start.elapsed());
    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
    Out::Text(format!("PyEnv загружен: {:?}", time_start.elapsed())))
    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
    .expect("Ошибка связи с консолью");

    let agent: Agent = agent_builder
    .preamble(SYSTEM_PROMPT) //System prompt
    .context(TOOLS)
    .context(MAIN_SKILLS)
    .context(SAST_SKILLS)
    .context(DAST_SKILLS)
    /* Не требуются более
    .tool(ToolSumI32) //Инструмент добавили
    .tool(ToolSumI64)
    .tool(ToolSubI64)
    */
    .tool(RunZap)
    .tool(RunNuclei)
    .tool(ReadFile)
    .tool(WriteFile)
    .tool(HttpRequest)
    //.tool(DumpEnv)
    //.tool(FindFiles)
    .tool(DirectoryContents)
    .tool(RunBandit)
    .tool(RunSemgrep)
    .default_max_turns(MAX_LLM_CALLS) //Максимум обращений к модели
    .build(); //Builder -> Agent построить короче

    info!("Агент готов: {:?}", time_start.elapsed());
    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
    Out::Text(format!("Агент готов: {:?}", time_start.elapsed())))
    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
    .expect("Ошибка связи с консолью");

    loop //Общение с душевнобольным
    {
        let prompt: String = match prompt_rx.recv().await
        {
            Some(prompt) => prompt,

            None => break,
        };

        let prompt: &str = prompt.trim();

        info!("Запрос - {}\t|\t{:?}", prompt, time_start.elapsed());

        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
        Out::Text(prompt.to_string()))
        .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
        .expect("Ошибка связи с консолью");

        if prompt == "exit"
        {
            break;
        } else if prompt == ":_"
        {
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг")
                .send(Out::Clear)
                .expect("Ошибка связи с консолью");

            continue;
        } else if prompt.is_empty()
        {
            warn!("Пустой запрос даст ошибку");
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(format!("Пустой запрос даст ошибку")))
            .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Ошибка связи с консолью");

            continue;
        }

        let time_prompt: Instant = Instant::now();

        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
        Out::Text(format!("Запрос передан в обработку...")))
        .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
        .expect("Ошибка связи с консолью");

        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг")
        .send(Out::CompressedStart)
        .expect("Ошибка связи с консолью");

        let response: Result<PromptResponse, _> =  agent.prompt(prompt).extended_details().await;

        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг")
        .send(Out::CompressedEnd)
        .expect("Ошибка связи с консолью");

        match response
        {
            Ok(response) =>
            {
                let response: PromptResponse = response;

                info!("\n{}\n{:#?}\n{:?}", response.output, response.usage, time_prompt.elapsed());
                CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                Out::Text(format!("{}\n{:?}", response.output, time_prompt.elapsed())))
                .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
                .expect("Ошибка связи с консолью");
            }

            Err(err) =>
            {
                error!("Ошибка ответа!\n{:?}", err);
                CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                Out::Text(format!("Ошибка ответа!\n{:?}", err)))
                .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
                .expect("Ошибка связи с консолью");

                continue;
            }
        }
    }

    //Закрыть поток консоли
    CONSOLE_OUT_TX.get().expect("Невозможное возможно").send(Out::Shutdown)
    .inspect_err(|err| error!("Ошибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
    .expect("Ошибка связи с консолью");

    //Подтянуть её поток
    console_thread.join()
    .inspect_err(|err| error!("Ошибка присоединения потока консоли - {:?}\t|\t{:?}", err, time_start.elapsed()))
    .expect("Ошибка присоединения потока консоли");

    info!("{:?}", time_start.elapsed());
    println!("{:?}", time_start.elapsed());
}
        
