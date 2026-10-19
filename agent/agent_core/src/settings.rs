use std::sync::OnceLock; //Для переменных

use std::env::var; //Окружение
use std::time::Instant; //Таймер

use crate::io::{CONSOLE_OUT_TX, Out};

use tracing::{error, info}; //Макросы логирования

pub static SYSTEM_PROMPT: &str = include_str!("../../system_prompt.md");

//system_prompt. Собирается из файлов на этапе компиляции
/*
pub static FULL_PROMPT: &str = concat!(
    "SYSTEM_PROMPT:\n",
    include_str!("../../system_prompt.md"),
    "MAIN_SKILLS:\n",
    include_str!("../../skills/main_skills.md"),
    "SAST_SKILLS:\n",
    include_str!("../../skills/sast/sast_skills.md"),
    // "DAST_SKILLS:\n",
    // include_str!("../../skills/dast/dast_skills.md"),
    "TOOLS:\n",
    include_str!("../../tools/tools.md")
);
*/

//Скиллы
pub static MAIN_SKILLS: &str = include_str!("../../skills/main_skills.md");
#[cfg(feature = "sast")]
pub static SAST_SKILLS: &str = include_str!("../../skills/sast_skills.md");
#[cfg(feature = "dast")]
pub static DAST_SKILLS: &str = include_str!("../../skills/dast_skills.md");
#[cfg(feature = "docker")]
pub static DOCKER_SKILLS: &str = include_str!("../../skills/docker_skills.md");

//Инструменты
pub static MAIN_TOOLS: &str = include_str!("../../tools/main_tools.md");
#[cfg(feature = "sast")]
pub static SAST_TOOLS: &str = include_str!("../../tools/sast_tools.md");
#[cfg(feature = "dast")]
pub static DAST_TOOLS: &str = include_str!("../../tools/dast_tools.md");
#[cfg(feature = "docker")]
pub static DOCKER_TOOLS: &str = include_str!("../../tools/docker_tools.md");
#[cfg(feature = "unsafe_tools")]
pub static UNSAFE_TOOLS: &str = include_str!("../../tools/unsafe_tools.md");

//Максимум вывовов модели
pub const MAX_LLM_CALLS: usize = 200;

//Ссылка на сервер с моделью
pub static MODEL_SERVER_URL: &str = "https://deepcode.ci.nsu.ru/api";

//id модели
pub static MODEL_SERVER_ID: &str = "qwen3.8-flash-next";

pub static AUTO: OnceLock<bool> = OnceLock::new();
pub static UNSAFE: OnceLock<bool> = OnceLock::new();

pub fn load_settings(time: &Instant)
{
    match var("AUTO")
    {
        Ok(var) =>
        {
            info!("Параметр запуска обнаружен");
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(String::from("Параметр запуска обнаружен")))
            .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
            .expect("Ошибка связи с консолью");

            if var == "1"
            {
                info!("Переход в автоматический режим...");
                CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                Out::Text(String::from("Переход в автоматический режим...")))
                .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
                .expect("Ошибка связи с консолью");

                AUTO.set(true).inspect_err(|err| error!("Неверотяно - {:?}", err)).expect("Неверотяно");
            } else {
                info!("Переход в ручной режим...");
                CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                Out::Text(String::from("Переход в ручной режим...")))
                .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
                .expect("Ошибка связи с консолью");

                AUTO.set(false).inspect_err(|err| error!("Неверотяно - {:?}", err)).expect("Неверотяно");
            }
        }

        Err(_) =>
        {
            info!("Параметр запуска не обнаружен");
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(String::from("Параметр запуска не обнаружен")))
            .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
            .expect("Ошибка связи с консолью");

            info!("Переход в ручной режим...");
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(String::from("Переход в ручной режим...")))
            .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
            .expect("Ошибка связи с консолью");

            AUTO.set(false).inspect_err(|err| error!("Неверотяно - {:?}", err)).expect("Неверотяно");
        }
    }

    match var("UNSAFE")
    {
        Ok(var) =>
        {
            info!("Параметр запуска обнаружен");
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(String::from("Параметр запуска обнаружен")))
            .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
            .expect("Ошибка связи с консолью");

            if var == "1"
            {
                info!("Определена свободная среда...");
                CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                Out::Text(String::from("Определена свободная среда...")))
                .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
                .expect("Ошибка связи с консолью");

                UNSAFE.set(true).inspect_err(|err| error!("Неверотяно - {:?}", err)).expect("Неверотяно");
            } else {
                info!("Определена безопасная среда...");
                CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                Out::Text(String::from("Определена безопасная среда...")))
                .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
                .expect("Ошибка связи с консолью");

                UNSAFE.set(false).inspect_err(|err| error!("Неверотяно - {:?}", err)).expect("Неверотяно");
            }
        }

        Err(_) =>
        {
            info!("Параметр запуска не обнаружен");
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(String::from("Параметр запуска не обнаружен")))
            .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
            .expect("Ошибка связи с консолью");

            info!("Определена безопасная среда...");
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(String::from("Определена безопасная среда...")))
            .inspect_err(|err| error!("\nОшибка связи с консолью - {:?}\t|\t{:?}", err, time.elapsed()))
            .expect("Ошибка связи с консолью");

            UNSAFE.set(false).inspect_err(|err| error!("Неверотяно - {:?}", err)).expect("Неверотяно");
        }
    }

}
