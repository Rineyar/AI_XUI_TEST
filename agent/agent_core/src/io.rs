use rig::client::{Client, ModelListingClient}; //Тип данных для соединения
use rig::providers::{openai::OpenAICompletionsExt}; //Для дипсика местного разлива
use rig::model::ModelList; //Тип списка моделей

use std::sync::OnceLock; //Хранение вывода
use std::sync::mpsc; //Связь tx-rx меж потоками
use std::io::{stdout, Stdout}; //Для вывода с crossterm
use std::thread; //Теперь консоль будет жить здесь
use std::time::Instant;

use crossterm::{execute, event::{EnableMouseCapture, DisableMouseCapture}, 
terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}}; //А это и есть вывод
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind}; //Для захвата и обработки мыши

use ratatui::{backend::CrosstermBackend, text::{Line}, widgets::Paragraph, Terminal}; //Новый вывод
use ratatui::layout::{Constraint, Layout, Rect}; //Рамочки и координаты символов

use tui_textarea::TextArea; //Кусок вввода

use tokio::time::{Duration, timeout}; //Для ограничения времени на операцию

use tracing::{error, info, warn}; //Макросы логирования

pub static CONSOLE_OUT_TX: OnceLock<mpsc::Sender<Out>> = OnceLock::new(); //Отправщик для консоли

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

pub enum Out
{
    CompressedStart,
    Text(String),
    CompressedEnd,
    Toggle(usize),
    Clear,
    Shutdown
}

pub fn spawn_console_thread(prompt_tx: tokio::sync::mpsc::UnboundedSender<String>) -> thread::JoinHandle<()>
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

pub fn restore_terminal()
{
    let _ = disable_raw_mode();

    let _ = execute!(stdout(), DisableMouseCapture, LeaveAlternateScreen);
}

pub async fn print_model_list(model: &Client<OpenAICompletionsExt>, time_start: &Instant)
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
