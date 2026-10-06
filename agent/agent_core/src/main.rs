use rig::client::AgentClientExt; //.agent метод
use rig::client::Client; //Тип данных для соединения
use rig::completion::Prompt; //.prompt метод
use rig::model::ModelList; //Тип листа моделей
use rig::client::ModelListingClient; //Для сбора листа моделей
use rig::prelude::Agent; //Тип данных для агента
use rig::agent::AgentBuilder; //Тип для билдера
use rig::providers::{openai::CompletionsClient, openai::OpenAICompletionsExt}; //Для дипсика местного разлива
use rig::agent::PromptResponse; //Скрытый тип подробного ответа

use std::mem; //Для take, чтобы по красоте
use std::time::Instant; //Таймер
use std::env::args; //Арги для выбора модели
use std::env::var; //Окружение для API ключа
use std::io::{stdin, BufRead}; //Для нового чтения строки
use std::borrow::Cow; //Для обрезка строки
use std::sync::OnceLock; //Хранение вывода
use std::thread; //Теперь консоль будет жить здесь
use std::sync::mpsc; //Связь tx-rx меж потоками
use std::io::{stdout, Stdout, Write}; //Для вывода с crossterm

use crossterm::{cursor::{MoveUp, MoveToColumn}, queue, style::Print, terminal::{Clear, ClearType}}; //А это и есть вывод

use dotenvy::dotenv; //Крейт для удобного чтения .env;

use tracing_appender::{rolling::never, non_blocking}; //Логи
use tracing::{error, info, warn}; //Макросы логирования

use tokio::time::{Duration, timeout}; //Для ограничения времени на операцию

mod settings; //Настройки ядра
use settings::*;

mod tools; //Инструменты
use tools::*;

mod py_env; //Py среда
use py_env::*;

mod guards; //Гварды

static CONSOLE_OUT_TX: OnceLock<mpsc::Sender<Out>> = OnceLock::new(); //Отправщик для консоли

enum ConsoleOut
{
    Text(String),
    Compressed(CompressedOut),
}

struct CompressedOut
{
    compressed: bool,
    out: Vec<ConsoleOut>,
}

enum Out
{
    CompressedStart,
    Text(String),
    CompressedEnd,
    Toggle(usize),
    Clear,
    Shutdown
}

fn spawn_console_thread() -> thread::JoinHandle<()>
{
    fn render(stdout: &mut Stdout, blocks: &[ConsoleOut], rendered_lines: &mut usize)
    {
        let mut new_lines: usize = 0;

        if *rendered_lines > 0
        {
            queue!(stdout, MoveUp(*rendered_lines as u16), MoveToColumn(0), Clear(ClearType::FromCursorDown)).expect("Ошибка чистки вывода");
        }

        for block in blocks.iter()
        {
            match block
            {
                ConsoleOut::Text(text) =>
                {
                    queue!(stdout, Print(text), Print('\n')).expect("Ошибка очереди вывода");
                    new_lines += 1;
                }

                ConsoleOut::Compressed(block) =>
                {
                    if block.compressed
                    {
                        queue!(stdout, Print("Скрыто "), Print(block.out.len()), Print(" элементов"), Print('\n')).expect("Ошибка очереди вывода");
                        new_lines += 1;
                    } else {
                        for line in block.out.iter()
                        {
                            //queue!(stdout, Print(line), Print('\n')).expect("Ошибка очереди вывода");
                            new_lines += 1;
                        }
                    }
                }
            }
        }

        stdout.flush().expect("Ошибка вывода");

        *rendered_lines = new_lines;
    }

    fn get_state_by_path_mut<'a>(path: &[usize], state: &'a mut[ConsoleOut]) -> &'a mut ConsoleOut
    {
        let mut current_state: &mut ConsoleOut = &mut state[path[0]];

        for &current_idx in path.iter().skip(1)
        {
            current_state = match current_state
            {
                ConsoleOut::Compressed(block) =>
                {
                    &mut block.out[current_idx]
                }

                ConsoleOut::Text(_) =>
                {
                    unreachable!("Путь проходит через Text");
                }
            };
        }

        return current_state;
    }

    let (tx, rx) = mpsc::channel(); //Связь с консолью

    CONSOLE_OUT_TX.set(tx).expect("Невозможное случилось"); //static поставить

    let console_thread: thread::JoinHandle<()> = thread::spawn(move ||
    {
        let mut state: Vec<ConsoleOut> = Vec::with_capacity(128); //Вектор условно строк
        let mut compressed_index: Vec<Vec<usize>> = (0..8).map(|_| Vec::with_capacity(4)).collect(); //Вектора индексов по глубине

        let mut stdout: Stdout = stdout(); //Просто без скобок

        let mut current_compressed: Option<Vec<usize>> = None;

        let mut rendered_lines: usize = 0; //Сколько линий есть

        while let Ok(event) = rx.recv()
        {
            match event
            {
                Out::Shutdown =>
                {
                    break;
                }

                Out::CompressedStart =>
                {
                    match &mut current_compressed
                    {
                        Some(path) =>
                        {
                            let current_state: &mut ConsoleOut = get_state_by_path_mut(path, &mut state);

                            let new_idx: usize;

                            match current_state
                            {
                                ConsoleOut::Compressed(block) =>
                                {
                                    new_idx = block.out.len();

                                    block.out.push(ConsoleOut::Compressed( CompressedOut { compressed: true, out: Vec::new() }));
                                }

                                ConsoleOut::Text(_) =>
                                {
                                    unreachable!("current_compressed указывает на Text");
                                }
                            }

                            path.push(new_idx);
                            compressed_index.push(path.clone());
                        }

                        None =>
                        {
                            let new_idx: usize = state.len();

                            state.push(ConsoleOut::Compressed(CompressedOut { compressed: true, out: Vec::new() }));
                            
                            let path: Vec<usize> = vec![new_idx];

                            compressed_index.push(path.clone());
                            current_compressed = Some(path);
                        }
                    }
                }

                Out::Text(msg) =>
                {
                    match &current_compressed
                    {
                        Some(path) =>
                        {
                            let current_state: &mut ConsoleOut = get_state_by_path_mut(path, &mut state);

                            match current_state
                            {
                                ConsoleOut::Text(_) =>
                                {
                                    unreachable!("current_compressed указывает на Text");
                                }

                                ConsoleOut::Compressed(block) =>
                                {
                                    block.out.push(ConsoleOut::Text(msg));
                                }
                            }
                        }

                        None =>
                        {
                            state.push(ConsoleOut::Text(msg));
                        }
                    }
                }

                Out::CompressedEnd =>
                {
                    if let Some(path) = &mut current_compressed
                    {
                        path.pop().expect("Пустой путь внутри current_compressed");

                        if path.is_empty()
                        {
                            current_compressed = None;
                        }
                    } else {
                        unreachable!("Невозможно вызвать End без открытого блока");
                    }
                }

                Out::Toggle(id) =>
                {
                    let path: &Vec<usize> = &compressed_index[id];

                    let current_state: &mut ConsoleOut = get_state_by_path_mut(path, &mut state);

                    match current_state
                    {
                        ConsoleOut::Compressed(block) =>
                        {
                            block.compressed = !block.compressed;
                        }

                        ConsoleOut::Text(_) =>
                        {
                            unreachable!("Путь Toggle указывает на Text");
                        }
                    }
                }

                Out::Clear =>
                {
                    state.clear();
                    compressed_index.clear();
                    current_compressed = None;
                }
            }

            render(&mut stdout, &state, &mut rendered_lines);
        }
    });

    return console_thread;
}

async fn print_model_list(model: &Client<OpenAICompletionsExt>, time_start: &Instant)
{
    let models: ModelList = timeout(Duration::from_secs(5), model.list_models())
    .await.inspect_err(|err|
    error!("Превышено время ожидания списка моделей - {:?}\t|\t{:?}", err, time_start.elapsed()))
    .expect("Превышено время ожидания списка моделей").inspect_err(|err|
    error!("Не удалось получить список моделей - {:?}\t|\t{:?}", err, time_start.elapsed()))
    .expect("Не удалось получить список моделей");

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

    let console_thread: thread::JoinHandle<()> = spawn_console_thread(); //Создать поток

    //Логер
    let (loger, _log_guard) = non_blocking(never("./logs", format!("log_{:?}.log", 
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("Времени нет").as_secs())));

    tracing_subscriber::fmt().with_writer(loger).with_ansi(false).init();

    info!("Логер ожил: {:?}", time_start.elapsed());
    println!("Логер ожил: {:?}", time_start.elapsed());

    dotenv().ok(); //Чтобы он мог .env подсосать

    let mut args_list: Vec<String> = args().collect();

    if args_list.len() == 1
    {
        error!("Укажите модель через -L, -D, -Q или -O!");
        panic!("Укажите модель через -L, -D, -Q или -O!");
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
    println!("Клиент загружен: {:?}", time_start.elapsed());

    load_py_env(); //Создание Py субботы

    load_py_guards(); //Гварды

    info!("PyEnv загружен: {:?}", time_start.elapsed());
    println!("PyEnv загружен: {:?}", time_start.elapsed());

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
    .tool(FindFiles)
    .tool(DirectoryContents)
    .tool(RunBandit)
    .tool(RunSemgrep)
    .default_max_turns(MAX_LLM_CALLS) //Максимум обращений к модели
    .build(); //Builder -> Agent построить короче

    info!("Агент готов: {:?}", time_start.elapsed());
    println!("Агент готов: {:?}", time_start.elapsed());

    loop //Общение с душевнобольным
    {
        let mut buffer: Vec<u8> = Vec::new();

        match stdin().lock().read_until(b'\n', &mut buffer)
        {
            Ok(bytes) =>
            {
                if bytes == 0
                {
                    break;
                }
            }

            Err(err) =>
            {
                error!("Запрос не считан!\n{:?}", err);
                break;
            }
        };

        let prompt: Cow<'_, str> = String::from_utf8_lossy(&buffer);
        let prompt: &str = prompt.trim();

        if prompt == "exit"
        {
            break;
        }

        if prompt.is_empty()
        {
            warn!("Пустой запрос даст ошибку");
            println!("Пустой запрос даст ошибку");

            continue;
        }

        let time_prompt: Instant = Instant::now();

        println!("Запрос передан в обработку...");

        match agent.prompt(prompt).extended_details().await
        {
            Ok(response) =>
            {
                let response: PromptResponse = response;

                info!("\n{}\n{:#?}\n{:?}", response.output, response.usage, time_prompt.elapsed());
                println!("{}\n{:?}", response.output, time_prompt.elapsed());
            }

            Err(err) =>
            {
                error!("Ошибка ответа!\n{:?}", err);
                println!("Ошибка ответа!\n{:?}", err);

                continue;
            }
        }
    }

    //Закрыть поток консоли
    CONSOLE_OUT_TX.get().unwrap(/*SAFETY точно инит есть*/).send(Out::Shutdown)
    .inspect_err(|err| 
    { error!("Ошибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed());
    println!("Ошибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()); })
    .expect("Ошибка связи с консолью");

    //Подтянуть её поток
    console_thread.join().inspect_err(|err|
    { error!("Ошибка присоединения потока консоли - {:?}\t|\t{:?}", err, time_start.elapsed());
    println!("Ошибка присоединения потока консоли - {:?}\t|\t{:?}", err, time_start.elapsed())} )
    .expect("Ошибка присоединения потока консили");

    info!("{:?}", time_start.elapsed());
    println!("{:?}", time_start.elapsed());
}
        