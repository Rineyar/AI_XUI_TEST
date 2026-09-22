use pyo3::prelude::*; //Для работы с PyO3
use pyo3::types::{PyFunction, PyModule}; //Тип для модуля и функции

use std::{ffi::CString, str::FromStr}; //Спецстроки
use std::collections::HashMap; //Таблица для хранения структур
use std::sync::OnceLock; //Для получения глобального py_env

use include_dir::{Dir, include_dir}; //Для сборка всей папки и её тип

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

async fn collect_py_modules(directory: &Dir<'static>) -> HashMap<String, PyFileModule> //Сбор модулей
{
    let mut modules: HashMap<String, PyFileModule> = HashMap::with_capacity(16);

    Python::attach(|py: Python<'_>| //Py среда
    {
        let sys: Bound<'_, PyModule> = PyModule::import(py, "sys").unwrap();
        let sys_modules: Bound<'_, PyAny> = sys.getattr("modules").unwrap();

        for file in directory.files()
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
            let module_name: &str = file.path().file_stem().expect("Отсутствует имя файла").to_str().expect("Ошибка перевода &OsStr в &str");

            let new_module: Bound<'_, PyModule> = PyModule::new(py, module_name).unwrap();

            sys_modules.set_item(module_name, &new_module).unwrap();

            modules.insert(module_name.to_string(), PyFileModule::with_module(new_module.unbind()));
        }

        for file in directory.files()
        {
            if file.path().extension().unwrap_or_default() != "py"
            {
                continue;
            }

            let code: CString = CString::from_str(file.contents_utf8().expect("Py файл не в UTF-8. Не порядок")).expect("Ошибка перевода &str (мб валидной) в CString");

            if code.is_empty()
            {
                continue;
            }

            let module_name: &str = file.path().file_stem().expect("Отсутствует имя файла").to_str().expect("Ошибка перевода &OsStr в &str");

            let module_elem: &PyFileModule = modules.get(module_name).expect("Модуль затерялся");

            let module: &Bound<'_, PyModule> = module_elem.module.bind(py);

            py.run(&code, Some(&module.dict()), Some(&module.dict())).expect("Ошибка выполнения Python модуля");
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

    return modules;
}

//Создание пятницы
pub async fn load_py_env()
{
    PY_ENV.set(collect_py_funcs_from_modules(collect_py_modules(&PY_TOOLS_DIR).await).await).expect("PyEnv уже инициализирован");
}

//Получение доступа
pub async fn get_py_env() -> &'static HashMap<String, PyFileModule>
{
    return PY_ENV.get().expect("PyEnv не инициализирован");
}

pub async fn load_py_guards()
{
    PY_GUARDS.set(collect_py_funcs_from_modules(collect_py_modules(&PY_GUARDS_DIR).await).await).expect("PyGuards уже инициализирован");
}

pub async fn get_py_guards() -> &'static HashMap<String, PyFileModule>
{
    return PY_GUARDS.get().expect("PyGuards не инициализирован");
}