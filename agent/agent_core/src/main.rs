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
use std::sync::OnceLock; //Хранение вывода
use std::thread; //Теперь консоль будет жить здесь
use std::sync::mpsc; //Связь tx-rx меж потоками
use std::io::{stdout, Stdout}; //Для вывода с crossterm

use crossterm::{execute, event::{EnableMouseCapture, DisableMouseCapture}, 
terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}}; //А это и есть вывод
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind}; //Для захвата и обработки мыши

use ratatui::{backend::CrosstermBackend, text::{Line}, widgets::Paragraph, Terminal}; //Новый вывод
use ratatui::layout::{Constraint, Layout, Rect}; //Рамочки и координаты символов

use tui_textarea::TextArea; //Кусок вввода

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
    id: usize,
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

fn spawn_console_thread(prompt_tx: tokio::sync::mpsc::UnboundedSender<String>) -> thread::JoinHandle<()>
{
    fn render_blocks(block: &ConsoleOut, lines: &mut Vec<Line<'static>>, compressed_rows: &mut [Option<u16>])
    {
        match block
        {
            ConsoleOut::Text(text) =>
            {
                for line in text.lines()
                {
                    lines.push(Line::from(line.to_string()));
                }
            }

            ConsoleOut::Compressed(block) =>
            {
                let row: u16 = lines.len() as u16;

                compressed_rows[block.id] = Some(row);

                if block.compressed
                {
                    lines.push(Line::from(format!("{}:> Скрыто {} элементов", block.id, block.out.len())));
                } else {
                    lines.push(Line::from(format!("{}:V Раскрыто {} элементов", block.id, block.out.len())));

                    for next_block in block.out.iter()
                    {
                        render_blocks(next_block, lines, compressed_rows);
                    }
                }
            }
        }
        
    }

    fn render(terminal: &mut Terminal<CrosstermBackend<Stdout>>, blocks: &[ConsoleOut], compressed_rows: &mut [Option<u16>], input: &TextArea<'static>, scroll: u16) -> (Rect, usize)
    {
        let mut lines: Vec<Line<'static>> = Vec::with_capacity(16);

        for row in compressed_rows.iter_mut()
        {
            *row = None;
        }

        for block in blocks.iter()
        {
            render_blocks(block, &mut lines, compressed_rows);
        }

        let mut current_output_area: Rect = Rect::default();

        terminal.draw(|frame|
        {
            let [output_area, input_area] = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(frame.area());

            current_output_area = output_area;

            let paragraph = Paragraph::new(lines.clone()).scroll((scroll, 0));

            frame.render_widget(paragraph, output_area);
            frame.render_widget(input, input_area);
        }).expect("Ошибка отрисовки Ratatui");

        return (current_output_area, lines.len());
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

    let console_thread_writer: thread::JoinHandle<()> = thread::spawn(move ||
    {
        let mut state: Vec<ConsoleOut> = Vec::with_capacity(128); //Вектор условно строк
        let mut compressed_index: Vec<Vec<usize>> = Vec::with_capacity(16); //Вектора индексов по глубине
        let mut compressed_rows: Vec<Option<u16>> = Vec::with_capacity(64); //Координаты < и V для свёрток

        let mut current_compressed: Option<Vec<usize>> = None;

        let mut input: TextArea<'static> = TextArea::default();
        let mut running: bool = true;

        enable_raw_mode().expect("Не удалось включить raw mode");
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture).expect("Не удалось настроить терминал");
        let backend: CrosstermBackend<Stdout> = CrosstermBackend::new(stdout()); //Связь с cross

        //Консолька
        let mut terminal: Terminal<CrosstermBackend<Stdout>> = Terminal::new(backend).expect("Не удалось создать терминал Ratatui");

        let mut scroll: u16 = 0;
        let mut output_area: Rect = Rect::default();

        let mut fall_down: bool = true;
        let mut lines_count: usize = 0;
        let mut max_scroll: u16 = 0;

        while running
        {
            while let Ok(out) = rx.try_recv()
            {
                match out
                {
                    Out::Shutdown =>
                    {
                        running = false;
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

                                        block.out.push(ConsoleOut::Compressed( CompressedOut { id: compressed_index.len(), compressed: true, out: Vec::new() }));
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

                                state.push(ConsoleOut::Compressed(CompressedOut { id: compressed_index.len(), compressed: true, out: Vec::new() }));
                                
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
                        let path: &Vec<usize> = match compressed_index.get(id)
                        {
                            Some(path) => path,

                            None =>
                            {
                                warn!("id блока не существует");
                                CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                                Out::Text(format!("id блока не существует")))
                                .inspect_err(|err| error!("\nОшибка связи с консолью - {}", err))
                                .expect("Ошибка связи с консолью");

                                continue;
                            }
                        };

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

                //scroll = u16::MAX;
            }

            if !running
            {
                break;
            }

            if event::poll(Duration::from_millis(20)).expect("Ошибка чтения консоли")
            {
                match event::read().expect("Ошибка чтения события")
                {
                    Event::Key(key) if key.kind == KeyEventKind::Press =>
                    {
                        match key.code
                        {
                            KeyCode::Enter =>
                            {
                                let prompt: String = input.lines().join("\n");

                                prompt_tx.send(prompt)
                                    .expect("Main больше не принимает ввод");

                                input = TextArea::default();
                            }

                            _ =>
                            {
                                input.input(key);
                            }
                        }
                    }

                    Event::Mouse(mouse) =>
                    {
                        match mouse.kind
                        {
                            MouseEventKind::ScrollUp =>
                            {
                                fall_down = false;

                                scroll = scroll.saturating_sub(3);
                            }

                            MouseEventKind::ScrollDown =>
                            {
                                scroll = scroll.saturating_add(3);

                                if scroll >= max_scroll
                                {
                                    scroll = max_scroll;
                                    fall_down = true;
                                }
                            }

                            MouseEventKind::Down(MouseButton::Left) =>
                            {
                                if mouse.row >= output_area.y && mouse.row < output_area.y + output_area.height
                                {
                                    let clicked_row: u16 = scroll + (mouse.row - output_area.y);

                                    if let Some(id) = compressed_rows.iter().position(|row| *row == Some(clicked_row))
                                    {
                                        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг")
                                        .send(Out::Toggle(id))
                                        .expect("Ошибка связи с консолью");
                                    }
                                }
                            }

                            _ => {}
                        }
                    }

                    _ => {}
                }
            }

            compressed_rows.clear();
            compressed_rows.resize(compressed_index.len(), None);

            max_scroll = lines_count.saturating_sub(output_area.height as usize) as u16;

            let (new_output_area, lines_count_new) = render(&mut terminal, &state, &mut compressed_rows, &input, scroll);

            lines_count = lines_count_new;

            output_area = new_output_area;

            if fall_down
            {
                scroll = max_scroll;
            }
        }

        disable_raw_mode().expect("Не удалось отключить raw mode");
        execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen).expect("Не удалось восстановить терминал");
        terminal.show_cursor().expect("Не удалось вернуть курсор");

        let _terminal_guard: TerminalGuard = TerminalGuard;
    });

    return console_thread_writer;
}

struct TerminalGuard;

impl Drop for TerminalGuard
{
    fn drop(&mut self)
    {
        restore_terminal();
    }
}

fn restore_terminal()
{
    let _ = disable_raw_mode();

    let _ = execute!(stdout(), DisableMouseCapture, LeaveAlternateScreen);
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
        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
        Out::Text(format!("№{}: {:?}", i + 1, model.id)))
        .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time_start.elapsed()))
        .expect("Ошибка связи с консолью");
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
        
