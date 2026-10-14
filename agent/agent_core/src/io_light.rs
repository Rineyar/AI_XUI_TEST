use rig::client::{Client, ModelListingClient}; //Тип данных для соединения
use rig::providers::{openai::OpenAICompletionsExt}; //Для дипсика местного разлива
use rig::model::ModelList; //Тип списка моделей

use std::sync::OnceLock; //Хранение вывода
use std::sync::mpsc; //Связь tx-rx меж потоками
use std::io::{Stdin, stdin}; //База консоли
use std::thread; //Теперь консоль будет жить здесь
use std::time::Instant;

use tokio::time::{Duration, timeout}; //Для ограничения времени на операцию

use tracing::{error, info}; //Макросы логирования

pub static CONSOLE_OUT_TX: OnceLock<mpsc::Sender<Out>> = OnceLock::new(); //Отправщик для консоли

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
    let (tx, rx) = mpsc::channel(); //Связь с консолью

    CONSOLE_OUT_TX.set(tx).expect("Невозможное случилось"); //static поставить

    thread::spawn(move ||
    {
        let stdin: Stdin = stdin();

        loop
        {
            let mut prompt: String = String::new();

            match stdin.read_line(&mut prompt)
            {
                Ok(0) => { break; }

                Ok(_) =>
                {
                    if prompt_tx.send(prompt).is_err() 
                    {
                        error!("\nОшибка связи с консолью");

                        break;
                    }
                }

                Err(_) => { break; }
            }
        }
    });

    let console_thread_writer: thread::JoinHandle<()> = thread::spawn(move ||
    {
        while let Ok(out) = rx.recv()
        {
            match out
            {
                Out::Text(msg) =>
                {
                    println!("{}", msg);
                }

                Out::Shutdown => { break; }

                _ => { }
            }
        }
    });

    return console_thread_writer;
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
        .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time_start.elapsed()))
        .expect("Ошибка связи с консолью");
    }
}
