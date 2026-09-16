use pyo3::prelude::*; //Для работы с PyO3
use pyo3::types::PyModule; //Тип для модуля

use std::{ffi::CString, str::FromStr}; //Спецстроки
use std::collections::HashMap; //Таблица для хранения структур
use std::sync::OnceLock; //Для получения глобального py_env

use include_dir::{Dir, include_dir}; //Для сборка всей папки и её тип

static PY_ENV: OnceLock<HashMap<String, PyFileModule>> = OnceLock::new(); //Ждёт своего часа

static PY_TOOLS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../tools/tools_py"); //Сбор всей папки

#[derive(Debug)]
pub struct PyFileModule //Структура с модулем и его функциями
{
    pub module: Py<PyModule>,
    pub funcs: HashMap<String, Py<PyAny>>,
}

impl PyFileModule //Имплементация ей функции создания
{
    fn with_module(module: Py<PyModule>) -> Self
    {
        return Self { module, funcs: HashMap::with_capacity(4) };
    }
}

async fn collect_py_modules() -> HashMap<String, PyFileModule> //Сбор модулей
{
    let mut modules: HashMap<String, PyFileModule> = HashMap::with_capacity(16);

    Python::attach(|py: Python<'_>| //Py среда
    {
        for file in PY_TOOLS.files() //Все файлы
        {

            if file.path().extension().unwrap_or_default() != "py" //Скип залётного
            {
                continue;
            }

            //Вытащить код
            let code: CString = CString::from_str(file.contents_utf8().expect("Py файл не в UTF-8. Не порядок")).expect("Ошибка перевода &str (мб валидной) в CString");

            //Пустой файл
            if code.is_empty()
            {
                continue;
            }

            //Имя файла
            let module_name: CString = CString::from_str(file.path().file_stem().expect("Отсутствует имя файла").to_str().expect("Ошибка перевода &OsStr в &str")).expect("Ошибка перевода &str (мб валидной) в CString");

            //Собрать модуль
            let module: Bound<'_, PyModule> = PyModule::from_code(py, &code,
            &CString::from_str(file.path().file_name().expect("Отсутствует имя файла").to_str().expect("Ошибка перевода &OsStr в &str")).expect("Ошибка перевода &str (мб валидной) в CString"),
            &module_name)
            .expect("Ошибка сборки модуля");

            //Добавить модуль в таблицу по его имени
            modules.insert(String::from_str(module_name.to_str().expect("Ошибка перевода &OsStr в &str")).expect("Ошибка перевода &str (мб валидной) в String"), PyFileModule::with_module(module.unbind()));
        }
    });
    
    return modules;
}

async fn collect_py_funcs_from_modules(mut modules: HashMap<String, PyFileModule>) -> HashMap<String, PyFileModule>
{
    Python::attach(|py: Python<'_>| //Py четверг
    {
        for module_elem in modules.values_mut() //Пройтись по модулям
        {
            let module: &Bound<'_, PyModule> = module_elem.module.bind(py); //Отвязать

            //Проверить __all__. Там все tools функции описаны
            for elem in module.index().expect("__all__ отсутствует").iter()
            {
                let funcname: String = elem.extract().expect("Ошибка получения имени функции"); //Имя функции

                //Взять функцию
                let func: Bound<'_, PyAny> = module.getattr(funcname.as_str()).expect("Ошибка получения функии из модуля");

                //Проверить
                if !func.is_callable()
                {
                    panic!("{:?} указан в __all__, но не является функцией", funcname);
                }

                //Добавить в таблицу
                module_elem.funcs.insert(funcname, func.unbind());
            }
        }
    });

    return modules;
}

//Создание пятницы
pub async fn load_py_env()
{
    PY_ENV.set(collect_py_funcs_from_modules(collect_py_modules().await).await).expect("PyEnv уже инициализирован");
}

//Получение доступа
pub async fn get_py_env() -> &'static HashMap<String, PyFileModule>
{
    return PY_ENV.get().expect("PyEnv не инициализирован");
}