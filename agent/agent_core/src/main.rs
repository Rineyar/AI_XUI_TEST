use rig::client::AgentClientExt; //.agent метод
use rig::client::Client; //Тип данных для соединения
use rig::completion::Prompt; //.prompt метод
use rig::prelude::Agent; //Тип данных для агента
use rig::agent::AgentBuilder; //Тип для билдера
use rig::providers::{openai::CompletionsClient, openai::OpenAICompletionsExt}; //Для дипсика местного разлива
use rig::agent::PromptResponse; //Скрытый тип подробного ответа
use rig::http_client::HeaderMap; //Для Vega id session
use rig::agent::WithBuilderTools; //Для последовательной сборки агента

use std::mem; //Для take, чтобы по красоте
use std::time::Instant; //Таймер
use std::env::args; //Арги для выбора модели
use std::env::var; //Окружение для API ключа
use std::thread; //Сбор результата с потока
use std::fs::read_to_string;

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

//Консоль
#[cfg(not(feature = "tui"))]
#[path = "io_light.rs"]
mod io;
#[cfg(feature = "tui")]
mod io;
use io::*;

//Для DinD
#[cfg(feature = "docker")]
mod container;
#[cfg(feature = "docker")]
use container::*;

/*
Обязательно сделать проверку tools call
А то эта херь имеет свойство выдумывать.
+ динамическую обработку бы

Потом мб хуки навесить

Возможно вывести отдельный поток на управление py вызовами

Сделать проверку того, что в лог пихается. (прямо сейчас он запихал в лог весь бинарник, т.к. не смог его прочитать)
Что-то похожее уже возникало раньше...

Заняться кешированием запросов и памятью
*/

/*
================== СБОРКА ==================

# Полная сборка (TUI + Docker + SAST + DAST)
cargo build --release
cross build --release --target x86_64-unknown-linux-gnu

# Облегчённая (только базовые инструменты + io_light)
cargo build --release --no-default-features
cross build --release --no-default-features --target x86_64-unknown-linux-gnu

# Облегчённая + SAST
cargo build --release --no-default-features --features sast
cross build --release --no-default-features --features sast --target x86_64-unknown-linux-gnu

# Облегчённая + DAST
cargo build --release --no-default-features --features dast
cross build --release --no-default-features --features dast --target x86_64-unknown-linux-gnu

# Облегчённая + SAST + DAST
cargo build --release --no-default-features --features "sast dast"
cross build --release --no-default-features --features "sast dast" --target x86_64-unknown-linux-gnu

# TUI + SAST + DAST (без Docker)
cargo build --release --no-default-features --features "tui sast dast"
cross build --release --no-default-features --features "tui sast dast" --target x86_64-unknown-linux-gnu

# Docker + SAST + DAST (без TUI)
cargo build --release --no-default-features --features "docker sast dast"
cross build --release --no-default-features --features "docker sast dast" --target x86_64-unknown-linux-gnu


================== ЗАПУСК ==================

# Из каталога agent_core/
cargo run --release -- -S

# Из каталога agent/ (облегчённый)
cargo run --manifest-path ./agent_core/Cargo.toml --release --no-default-features -- -S

# Из каталога agent/ (SAST + DAST, без Docker и TUI)
cargo run --manifest-path ./agent_core/Cargo.toml --release --no-default-features --features "sast dast" -- -S

# Из каталога agent/ (полная сборка)
cargo run --manifest-path ./agent_core/Cargo.toml --release -- -S


================== DOCKER ==================

docker compose build
docker compose up -d
docker attach agent-core

# Ctrl+P, Ctrl+Q — отсоединиться, не останавливая контейнер

docker compose run --rm agent -S
docker compose run --rm agent -L
docker compose run --rm agent -O

docker compose run --name agent-auto agent -S
docker cp agent-auto:/agent/workspace/tests/test_1/answer.md .\agent\logs\answer.md ~пример
docker rm agent-auto
*/

#[tokio::main] //Асинк рантайм - база
async fn main()
{
    let time_start: Instant = Instant::now();

    //Чтобы при вылете терминал отпустил
    #[cfg(feature = "tui")]
    let default_panic = std::panic::take_hook();
    #[cfg(feature = "tui")]
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

    //Создать поток для консоли
    let console_thread: thread::JoinHandle<()> = spawn_console_thread(prompt_tx);

    dotenv().ok(); //Чтобы он мог .env подсосать

    #[cfg(feature = "docker")]
    init_sandbox_manager(&time_start).await; //Изолятор

    let mut args_list: Vec<String> = args().collect();

    if args_list.len() == 1
    {
        error!("Укажите модель через -S, -L, -Q или -O!");
        panic!("Укажите модель через -S, -L, -Q или -O!");
    } else if args_list.len() > 2
    {
        warn!("Обнаружены лишние аргументы:");
        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
        Out::Text(String::from("Обнаружены лишние аргументы:")))
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
            let model: Client<_> = Client::from_url("http://localhost:1234").inspect_err(|err|
            error!("Локальня модель недоступна - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Локальня модель недоступна"); //Получение по ссылке

            model.agent("openai/gpt-oss-20b")
        }

        "-S" =>
        {
            let model: Client<OpenAICompletionsExt> = CompletionsClient::builder() //Сборка клиента
            .api_key(var("MAIN_MODEL_API_KEY").inspect_err(|err|
            error!("Отсутствует API ключ - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Отсутствует API ключ")) //Передать ключ
            .base_url(MODEL_SERVER_URL) //Передать ссылку
            .build().inspect_err(|err|
            error!("Сборка разливного не удалась - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Сборка разливного не удалась");

            print_model_list(&model, &time_start).await;

            model.agent(MODEL_SERVER_ID)
        }

        "-Q" =>
        {
            let model: Client<OpenAICompletionsExt> = CompletionsClient::builder() //Сборка клиента
            .api_key(var("MAIN_MODEL_API_KEY").inspect_err(|err|
            error!("Отсутствует API ключ - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Отсутствует API ключ")) //Передать ключ
            .base_url(MODEL_SERVER_URL) //Передать ссылку
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

        "-V" =>
        {
            let mut headers: HeaderMap = HeaderMap::new();

            headers.insert(
        "X-VEGA-Agent-Session",
        var("VEGA_AGENT_SESSION_ID").inspect_err(|err|
            error!("Отсутствует VEGA_AGENT_SESSION_ID - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Отсутствует VEGA_AGENT_SESSION_ID")
            .parse().inspect_err(|err|
            error!("Некорректный VEGA_AGENT_SESSION_ID - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Некорректный VEGA_AGENT_SESSION_ID"));

            let model: Client<OpenAICompletionsExt> = CompletionsClient::builder() //Сборка клиента
            .api_key(var("OPENAI_API_KEY").inspect_err(|err|
            error!("Отсутствует API ключ - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Отсутствует API ключ")) //Передать ключ
            .base_url(var("OPENAI_BASE_URL").inspect_err(|err|
            error!("Отсутствует ссылка - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Отсутствует ссылка")) //Передать ссылку
            .http_headers(headers)
            .build().inspect_err(|err|
            error!("Сборка VEGA не удалась - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Сборка VEGA не удалась");

            model.agent(var("LOCAL_AGENT_MODEL").inspect_err(|err|
            error!("Отсутствует ID модели - {:?}\t|\t{:?}", err, time_start.elapsed()))
            .expect("Отсутствует ID модели"))
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

    //Навесить базу
    let agent_builder: AgentBuilder = agent_builder
    .preamble(SYSTEM_PROMPT)
    .context(MAIN_SKILLS)
    .context(MAIN_TOOLS);

    //Настройки
    let agent_builder: AgentBuilder = agent_builder
    .default_max_turns(MAX_LLM_CALLS);

    #[cfg(feature = "sast")]
    let agent_builder: AgentBuilder = agent_builder
    .context(SAST_SKILLS)
    .context(SAST_TOOLS);

    #[cfg(feature = "dast")]
    let agent_builder: AgentBuilder = agent_builder
    .context(DAST_SKILLS)
    .context(DAST_TOOLS);

    #[cfg(feature = "docker")]
    let agent_builder: AgentBuilder = agent_builder
    .context(DOCKER_SKILLS)
    .context(DOCKER_TOOLS);

    //Базовые инструменты
    let agent_builder: AgentBuilder<WithBuilderTools> = agent_builder
    .tool(ReadFile)
    .tool(WriteFile)
    .tool(HttpRequest)
    .tool(DumpEnv)
    .tool(FindFiles)
    .tool(DirectoryContents);

    //SAST + DAST
    #[cfg(feature = "sast")]
    let agent_builder: AgentBuilder<WithBuilderTools> = agent_builder
    .tool(RunBandit)
    .tool(RunSemgrep);

    #[cfg(feature = "dast")]
    let agent_builder: AgentBuilder<WithBuilderTools> = agent_builder
    .tool(RunZap)
    .tool(RunNuclei);

    //Контейнер
    #[cfg(feature = "docker")]
    let agent_builder: AgentBuilder<WithBuilderTools> = agent_builder
    .tool(CreateContainer)
    .tool(RemoveContainer)
    .tool(ExecuteCommand)
    .tool(SendPath)
    .tool(SendText)
    .tool(DownloadPath)
    .tool(ReadContainerFile);

    let agent: Agent = agent_builder.build(); //Builder -> Agent построить короче

    info!("Агент готов: {:?}", time_start.elapsed());
    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
    Out::Text(format!("Агент готов: {:?}", time_start.elapsed())))
    .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time_start.elapsed()))
    .expect("Ошибка связи с консолью");

    load_settings(&time_start); //Загрузить настройки

    if *AUTO.get().ok_or("Невероятно").inspect_err(|err| error!("{:?}", err)).expect("Невероятно")
    {
        let task: String = read_to_string("strategy.md")
        .inspect_err(|err| error!("Ошибка чтения задания в AUTO режиме - {:?}", err))
        .expect("Ошибка чтения задания в AUTO режиме");

        let prompt: &str = task.trim();

        info!("Задача:\n{:?}\n{:?}", prompt, time_start.elapsed());

        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
        Out::Text(prompt.to_string()))
        .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time_start.elapsed()))
        .expect("Ошибка связи с консолью");

        let time_prompt: Instant = Instant::now();

        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
        Out::Text(String::from("Запрос передан в обработку...")))
        .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time_start.elapsed()))
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

                info!("{}\n\n{:#?}\n\n{:?}\n", response.output, response.usage, time_prompt.elapsed());

                for call in response.completion_calls.iter()
                {
                    info!("\nLLM call №{}: {:?}\n", call.call_index, call.usage);
                }

                for call in response.completion_calls.iter()
                {
                    info!("\nLLM call №{} raw usage: {:?}\n", call.call_index, call.raw.get("usage"));
                }

                CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                Out::Text(format!("{}\n{:?}\n", response.output, time_prompt.elapsed())))
                .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
                .expect("Ошибка связи с консолью");
            }

            Err(err) =>
            {
                error!("Ошибка ответа!\n{:?}", err);
                CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                Out::Text(format!("Ошибка ответа!\n{:?}", err)))
                .inspect_err(|err| error!("\nОшибка связи с консолью - {:#?}\t|\t{:?}", err, time_start.elapsed()))
                .expect("Ошибка связи с консолью");
            }
        }
    } else {
        loop //Общение с душевнобольным
        {
            let prompt: String = match prompt_rx.recv().await
            {
                Some(prompt) => prompt,

                None => break,
            };

            let prompt: &str = prompt.trim();

            info!("Запрос:\n{:?}\n{:?}", prompt, time_start.elapsed());

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
                Out::Text(String::from("Пустой запрос даст ошибку")))
                .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
                .expect("Ошибка связи с консолью");

                continue;
            }

            let time_prompt: Instant = Instant::now();

            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(String::from("Запрос передан в обработку...")))
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

                    info!("{}\n\n{:#?}\n\n{:?}\n", response.output, response.usage, time_prompt.elapsed());

                    for call in response.completion_calls.iter()
                    {
                        info!("\nLLM call №{}: {:?}\n", call.call_index, call.usage);
                    }

                    for call in response.completion_calls.iter()
                    {
                        info!("\nLLM call №{} raw usage: {:?}\n", call.call_index, call.raw.get("usage"));
                    }

                    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                    Out::Text(format!("{}\n{:?}\n", response.output, time_prompt.elapsed())))
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
        
