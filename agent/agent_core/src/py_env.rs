use pyo3::prelude::*; //Для работы с PyO3
use pyo3::types::{PyFunction, PyModule}; //Тип для модуля и функции

use std::{ffi::CString, str::FromStr}; //Спецстроки
use std::collections::HashMap; //Таблица для хранения структур
use std::sync::OnceLock; //Для получения глобального py_env

use include_dir::{Dir, include_dir}; //Для сборка всей папки и её тип

use tracing::{error, info, warn}; //Логи

static PY_ENV: OnceLock<HashMap<String, PyFileModule>> = OnceLock::new(); //Ждёт своего часа

static PY_TOOLS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../tools/tools_py"); //Сбор всей папки

static PY_GUARDS: OnceLock<HashMap<String, PyFileModule>> = OnceLock::new();

static PY_GUARDS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../guardrails");

#[derive(Debug)]
pub struct PyFileModule //Структура с модулем и его функциями
{
    pub module: Py<PyModule>,
    pub funcs: HashMap<String, Py<PyFunction>>,
}

impl PyFileModule //Имплементация ей функции создания
{
    fn with_module(module: Py<PyModule>) -> Self
    {
        return Self { module, funcs: HashMap::with_capacity(4) };
    }
}

fn collect_py_modules(directory: &Dir<'static>) -> HashMap<String, PyFileModule> //Сбор модулей
{
    let mut modules: HashMap<String, PyFileModule> = HashMap::with_capacity(16);

    info!("Начат сбор модулей из: {:?}", directory);

    Python::attach(|py: Python<'_>| //Py среда
    {
        let sys: Bound<'_, PyModule> = PyModule::import(py, "sys")
        .inspect_err(|err| error!("Py SYS отсутствует - {}", err)).expect("Py SYS отсутствует");

        let sys_modules: Bound<'_, PyAny> = sys.getattr("modules")
        .inspect_err(|err| error!("Отсутствует списоку модулей - {}", err)).expect("Отсутствует список модулей");

        for file in directory.files()
        {
            if file.path().extension().unwrap_or_default() != "py" //Скип залётного
            {
                warn!("Файл {:?} не .py", file.path().file_name().unwrap_or_default());
                continue;
            }

            //Имя файла
            let module_name: &str = file.path().file_stem()
            .ok_or("Отсутствует имя файла").inspect_err(|err| error!("{}", err)).expect("Отсутствует имя файла")
            .to_str().ok_or("Ошибка перевода &OsStr в &str").inspect_err(|err| error!("{}", err)).expect("Ошибка перевода &OsStr в &str");

            let new_module: Bound<'_, PyModule> = PyModule::new(py, module_name)
            .inspect_err(|err| error!("Ошибка создания Py модуля - {}", err)).expect("Ошибка создания Py модуля");

            sys_modules.set_item(module_name, &new_module)
            .inspect_err(|err| error!("Ошибка добавления Py модуля - {}", err)).expect("Ошибка добавления Py модуля");

            modules.insert(module_name.to_string(), PyFileModule::with_module(new_module.unbind()));
        }

        info!("Головы {:?} собраны", directory);

        for file in directory.files()
        {
            if file.path().extension().unwrap_or_default() != "py"
            {
                warn!("Файл {:?} не .py", file.path().file_name().unwrap_or_default());
                continue;
            }

            //Вытащить код
            let code: CString = CString::from_str(file.contents_utf8().ok_or("Py файл не в UTF-8. Не порядок")
            .inspect_err(|err| error!("{}", err)).expect("Py файл не в UTF-8. Не порядок"))
            .inspect_err(|err| error!("Ошибка перевода &str (мб валидной) в CString - {}", err)).expect("Ошибка перевода &str (мб валидной) в CString");

            //Пустой файл
            if code.is_empty()
            {
                warn!("Файл {:?} пустой", file.path().file_name().unwrap_or_default());
                continue;
            }

            let module_name: &str = file.path().file_stem()
            .ok_or("Отсутствует имя файла").inspect_err(|err| error!("{}", err)).expect("Отсутствует имя файла")
            .to_str().ok_or("Ошибка перевода &OsStr в &str").inspect_err(|err| error!("{}", err)).expect("Ошибка перевода &OsStr в &str");

            let module_elem: &PyFileModule = modules.get(module_name).ok_or("Модуль затерялся")
            .inspect_err(|err| error!("{}", err)).expect("Модуль затерялся");

            let module: &Bound<'_, PyModule> = module_elem.module.bind(py);

            py.run(&code, Some(&module.dict()), Some(&module.dict()))
            .inspect_err(|err| error!("Ошибка выполнения Python модуля - {}", err)).expect("Ошибка выполнения Python модуля");
        }
    });

    info!("Модули {:?} собраны", directory);
    
    return modules;
}

fn collect_py_funcs_from_modules(mut modules: HashMap<String, PyFileModule>) -> HashMap<String, PyFileModule>
{
    info!("Начат сбор функций");

    Python::attach(|py: Python<'_>| //Py четверг
    {
        for module_elem in modules.values_mut() //Пройтись по модулям
        {
            let module: &Bound<'_, PyModule> = module_elem.module.bind(py); //Отвязать

            info!("Модуль - {:?}", module);

            //Проверить __all__. Там все tools функции описаны
            for elem in module.index().expect("__all__ отсутствует").iter()
            {
                let funcname: String = elem.extract().expect("Ошибка получения имени функции"); //Имя функции

                info!("Функция - {:?}", funcname);

                //Взять функцию
                let func: Bound<'_, PyAny> = module.getattr(funcname.as_str()).expect("Ошибка получения функии из модуля");
                let func: &Bound<'_, PyFunction> = func.cast::<PyFunction>().expect("Объект из __all__ не является функцией");

                //Проверить
                if !func.is_callable()
                {
                    panic!("{:?} указан в __all__, но не является функцией", funcname);
                }

                //Добавить в таблицу
                module_elem.funcs.insert(funcname, func.clone().unbind());
            }
        }
    });

    info!("Функции собраны");

    return modules;
}

//Создание пятницы
pub fn load_py_env()
{
    PY_ENV.set(collect_py_funcs_from_modules(collect_py_modules(&PY_TOOLS_DIR)))
    .inspect_err(|err| error!("PyEnv уже инициализирован - {:?}", err)).expect("PyEnv уже инициализирован");
}

//Получение доступа
pub fn get_py_env() -> &'static HashMap<String, PyFileModule>
{
    return PY_ENV.get().ok_or("PyEnv не инициализирован").inspect_err(|err| error!("{}", err)).expect("PyEnv не инициализирован");
}

pub fn load_py_guards()
{
    PY_GUARDS.set(collect_py_funcs_from_modules(collect_py_modules(&PY_GUARDS_DIR)))
    .inspect_err(|err| error!("PyGuards уже инициализирован - {:?}", err)).expect("PyGuards уже инициализирован");
}

pub fn get_py_guards() -> &'static HashMap<String, PyFileModule>
{
    return PY_GUARDS.get().ok_or("PyGuards не инициализирован").inspect_err(|err| error!("{}", err)).expect("PyGuards не инициализирован");
}