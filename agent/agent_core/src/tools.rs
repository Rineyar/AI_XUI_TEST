use rig::{rig_tool}; //fn -> tool
use rig::tool::ToolExecutionError;//Ошибка для тулза

use serde::Serialize; //Сбор в json
use serde_json::{Value, json, Map}; //Json собранный
use serde_pyobject::to_pyobject; //Для когвертации в pydict

use std::time::Instant; //Для таймера
use std::collections::HashMap; //Они кста тут живут  
use std::path::{Path, PathBuf}; //Для путей к конту и вне
use std::ffi::OsStr; //Путь для имени архива

use tar::{Builder, Header, Archive, Entries, Entry}; //Упаковка файлов для конта

use tracing::{error, info, warn}; //Логи

use crate::py_env::{get_py_env, PyFileModule}; //Py воскресенье для тузлов
use crate::guards::{GuardResponse, tools_guard}; //Гварды
use crate::{CONSOLE_OUT_TX, Out}; //Связь с консолью
use crate::container; //Функции для работы с контом

//Для PyEnv
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFunction};

#[derive(Serialize, Clone, Debug)]
pub struct ToolRequest
{
    pub module: &'static str,
    pub function: &'static str,
    pub args: Value
}

//Вставляет арг, или ничего, чтобы пыхтун не умирал
fn insert_arg<T: Serialize>(args: &mut Map<String, Value>, key: &str, value: Option<T>)
{
    if let Some(value) = value
    {
        args.insert(String::from(key), json!(value));
    }
}

//Обёртка вызовов
async fn call_py_tool(request: ToolRequest) -> Result<Py<PyAny>, ToolExecutionError>
{
    let time: Instant = Instant::now();

    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(Out::CompressedStart)
    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
    .expect("Ошибка связи с консолью");

    info!("\nИнструмент {:?} вызван с: {:?}\t|\t{:?}", request.function, request.args, time.elapsed());
    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
    Out::Text(format!("Инструмент {:?} вызван с: {:?}\t|\t{:?}", request.function, request.args, time.elapsed())))
    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
    .expect("Ошибка связи с консолью");

    let (verdict, request): (GuardResponse, ToolRequest) = tools_guard(request).await; //Вызов гварда

    if !verdict.allowed //Можно?
    {
        warn!("\nВердикт: гвард запретил - {:?}\t|\t{:?}", verdict.reason, time.elapsed());
        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
        Out::Text(format!("Вердикт: гвард запретил - {:?}\t|\t{:?}", verdict.reason, time.elapsed())))
        .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
        .expect("Ошибка связи с консолью");

        CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
        Out::CompressedEnd)
        .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
        .expect("Ошибка связи с консолью");

        return Err(ToolExecutionError::permission_denied(verdict.reason)); //Нельзя
    }

    info!("\nВердикт: гвард разрешил - {:?}\t|\t{:?}", verdict.reason, time.elapsed());
    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
    Out::Text(format!("Вердикт: гвард разрешил - {:?}\t|\t{:?}", verdict.reason, time.elapsed())))
    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
    .expect("Ошибка связи с консолью");

    let py_env: &HashMap<String, PyFileModule> = get_py_env(); //Получить вторник

    let func: &Py<PyFunction> = match py_env.get(request.module) //Функция
    {
        Some(module) => //Из модуля
        {
            match module.funcs.get(request.function) //Там лежит
            {
                Some(func) => { func }

                None =>
                {
                    error!("\nИнструмента нет - {:?}\t|\t{:?}", request.function, time.elapsed());
                    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                    Out::Text(format!("Инструмента нет - {:?}\t|\t{:?}", request.function, time.elapsed())))
                    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
                    .expect("Ошибка связи с консолью");

                    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                    Out::CompressedEnd)
                    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
                    .expect("Ошибка связи с консолью");

                    return Err(ToolExecutionError::not_found(format!("Tool {:?} in module {:?} is missing", request.function, request.module)));
                }
            }
        }

        None => 
        {
            error!("\nМодуля нет - {:?}\t|\t{:?}", request.module, time.elapsed());
            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::Text(format!("Модуля нет - {:?}\t|\t{:?}", request.module, time.elapsed())))
            .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
            .expect("Ошибка связи с консолью");

            CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
            Out::CompressedEnd)
            .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
            .expect("Ошибка связи с консолью");

            return Err(ToolExecutionError::not_found(format!("Module {:?} with tool {:?} is missing", request.module, request.function)));
        }
    };

    return tokio::task::spawn_blocking(move || -> Result<Py<PyAny>, ToolExecutionError>
    {
        return Python::attach(|py: Python<'_>| -> PyResult<Py<PyAny>>
        {
            let kwargs: Bound<'_, PyDict> = to_pyobject(py, &request.args)?.cast_into()?;

            let ret: Result<Py<PyAny>, PyErr> = if kwargs.is_empty()
            {
                func.call0(py)
            } else {
                func.call(py, (), Some(&kwargs))
            };

            match ret
            {
                Ok(_) => 
                {
                    info!("\nУспешно выполнено\t|\t{:?}", time.elapsed());
                    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                    Out::Text(format!("Успешно выполнено\t|\t{:?}", time.elapsed())))
                    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
                    .expect("Ошибка связи с консолью");

                    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                    Out::CompressedEnd)
                    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
                    .expect("Ошибка связи с консолью");
                }
                
                Err(_) => 
                {
                    error!("\nОшибка выполнения - {:?}\t|\t{:?}", ret, time.elapsed()); 
                    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                    Out::Text(format!("Ошибка выполнения - {:?}\t|\t{:?}", ret, time.elapsed())))
                    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
                    .expect("Ошибка связи с консолью");

                    CONSOLE_OUT_TX.get().expect("TX-RX консоли лёг").send(
                    Out::CompressedEnd)
                    .inspect_err(|err| error!("\nОшибка связи с консолью - {}\t|\t{:?}", err, time.elapsed()))
                    .expect("Ошибка связи с консолью");
                }
            }

            return ret;
        }).map_err(ToolExecutionError::from_error);
    }).await.inspect_err(|err| error!("Ошибка присоединения потока исполения - {:?}\t|\t{:?}", err, time.elapsed()))
    .expect("Ошибка присоединения потока исполения");
}

#[rig_tool(
    name = "create_container",
    description = "Create and start a new Docker sandbox container. Returns the container ID."
)]
pub async fn create_container() -> Result<String, ToolExecutionError>
{
    return container::create_container().await
    .map_err(|err| ToolExecutionError::other(format!("Failed to create container: {:?}", err)));
}

#[rig_tool(
    name = "remove_container",
    description = "Remove a Docker sandbox container by its ID.",
    params(
        id = "ID of the container to remove."
    )
)]
pub async fn remove_container(id: String) -> Result<(), ToolExecutionError>
{
    return container::remove_container(&id).await
    .map_err(|err| ToolExecutionError::other(format!("Failed to remove container: {:?}", err)));
}

#[rig_tool(
    name = "execute_command",
    description = "Execute a shell command inside a running Docker sandbox container. Returns command output and exit code.",
    params(
        id = "ID of the target container.",
        command = "Shell command to execute inside the container."
    )
)]
pub async fn execute_command(id: String, command: String) -> Result<String, ToolExecutionError>
{
    let (output, exit_code) = container::execute_command(&id, &command).await
    .map_err(|err| ToolExecutionError::other(format!("Failed to execute command: {:?}", err)))?;

    return Ok(format!("Exit code: {:?}\nOutput:\n{:?}", exit_code, output));
}

#[rig_tool(
    name = "send_path",
    description = "Upload a file or directory from the agent workspace into a Docker sandbox container. Files and directories are packaged automatically.",
    params(
        id = "ID of the target container.",
        local_path = "File or directory path relative to the agent workspace.",
        remote_path = "Destination directory relative to /workspace inside the container. Use an empty string for the workspace root."
    )
)]
pub async fn send_path(id: String, local_path: String, remote_path: String) -> Result<(), ToolExecutionError>
{
    let source: PathBuf = Path::new("/agent/workspace").join(&local_path);

    let filename: &OsStr = source.file_name().ok_or_else(|| ToolExecutionError::invalid_args("Invalid source path."))?/*.to_string_lossy().into_owned()*/;

    let mut archive: Builder<Vec<u8>> = Builder::new(Vec::new());
    archive.follow_symlinks(false);

    if source.is_dir()
    {
        archive.append_dir_all(filename, &source).map_err(|err| ToolExecutionError::other(format!("Failed to archive directory: {:?}", err)))?;
    } else {
        archive.append_path_with_name(&source, filename).map_err(|err| ToolExecutionError::other(format!("Failed to archive file: {:?}", err)))?;
    }

    let files: Vec<u8> = archive.into_inner().map_err(|err| ToolExecutionError::other(format!("Failed to finalize TAR archive: {:?}", err)))?;

    container::send_data(&id, &remote_path, files).await
    .map_err(|err| ToolExecutionError::other(format!("Failed to upload files: {:?}", err)))?;

    return Ok(());
}

#[rig_tool(
    name = "send_text",
    description = "Create or overwrite a UTF-8 text file directly inside a Docker sandbox container.",
    params(
        id = "ID of the target container.",
        path = "Destination directory relative to /workspace inside the container.",
        filename = "Name of the file to create.",
        content = "UTF-8 text content to write."
    )
)]
pub async fn send_text(id: String, path: String, filename: String, content: String) -> Result<(), ToolExecutionError>
{
    let mut archive: Builder<Vec<u8>> = Builder::new(Vec::new());
    let mut header: Header = Header::new_gnu();

    header.set_size(content.len() as u64);
    header.set_mode(0o644);

    archive.append_data(&mut header, &filename, content.as_bytes())
    .map_err(|err| ToolExecutionError::other(format!("Failed to archive text file: {:?}", err)))?;

    let files: Vec<u8> = archive.into_inner()
    .map_err(|err| ToolExecutionError::other(format!("Failed to finalize TAR archive: {:?}", err)))?;

    container::send_data(&id, &path, files).await
    .map_err(|err| ToolExecutionError::other(format!("Failed to upload text file: {:?}", err)))?;

    return Ok(());
}

#[rig_tool(
    name = "download_path",
    description = "Download a file or directory from a Docker sandbox container and extract it into the agent workspace.",
    params(
        id = "ID of the target container.",
        remote_path = "Source file or directory relative to /workspace inside the container.",
        local_path = "Destination directory relative to the agent workspace."
    )
)]
pub async fn download_path(id: String, remote_path: String, local_path: String) -> Result<(), ToolExecutionError>
{
    let files: Vec<u8> = container::load_data(&id, &remote_path).await
    .map_err(|err| ToolExecutionError::other(format!("Failed to download files: {:?}", err)))?;

    let destination: PathBuf = Path::new("/agent/workspace").join(&local_path);

    let mut archive: Archive<&[u8]> = Archive::new(files.as_slice());

    archive.unpack(&destination).map_err(|err| ToolExecutionError::other(format!("Failed to extract TAR archive: {:?}", err)))?;

    return Ok(());
}

#[rig_tool(
    name = "read_container_file",
    description = "Read the UTF-8 contents of a text file inside a Docker sandbox container without saving it locally.",
    params(
        id = "ID of the target container.",
        filename = "Text file path relative to /workspace inside the container."
    )
)]
pub async fn read_container_file(id: String, filename: String) -> Result<String, ToolExecutionError>
{
    use std::io::Read;

    let files: Vec<u8> = container::load_data(&id, &filename).await
    .map_err(|err| ToolExecutionError::other(format!("Failed to download text file: {:?}", err)))?;

    let mut archive: Archive<&[u8]> = tar::Archive::new(files.as_slice());

    let entries: Entries<'_, &[u8]> = archive.entries()
    .map_err(|err| ToolExecutionError::other(format!("Failed to read TAR archive: {:?}", err)))?;

    for entry in entries.into_iter()
    {
        let mut entry: Entry<'_, &[u8]> = entry.map_err(|err| ToolExecutionError::other(format!("Failed to read TAR entry: {:?}", err)))?;

        if !entry.header().entry_type().is_file()
        {
            continue;
        }

        let mut content: String = String::new();

        entry.read_to_string(&mut content).map_err(|err| ToolExecutionError::other(format!("Failed to decode text file: {:?}", err)))?;

        return Ok(content);
    }

    return Err(ToolExecutionError::not_found("No regular file found in downloaded archive."));
}

#[rig_tool(
    name = "write_file",
    description = "Write text to a file.",
    params(
        filename = "Path to the file relative",
        text = "Text content to write to the file."
    )
)]
pub async fn write_file(filename: String, text: String) -> Result<(), ToolExecutionError>
{
    let mut args: Map<String, Value> = Map::with_capacity(2);

    insert_arg(&mut args, "filename", Some(filename));
    insert_arg(&mut args, "text", Some(text));

    //Вызов
    call_py_tool(ToolRequest { module: "files", function: "write_file", args: Value::Object(args) }).await?;

    //Сбора нет
    return Ok(());
}

#[rig_tool(
    name = "read_file",
    description = "Read the contents of a text file from the workspace.",
    params(
        filename = "Path to the file relative to the workspace."
    )
)]
pub async fn read_file(filename: String) -> Result<String, ToolExecutionError>
{
    let mut args: Map<String, Value> = Map::with_capacity(1);

    insert_arg(&mut args, "filename", Some(filename));
    
    //Вызов
    let res: Py<PyAny> = call_py_tool(ToolRequest { module: "files", function: "read_file", args: Value::Object(args) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(
    name = "http_request",
    description = "Perform an HTTP or HTTPS GET or POST request.",
    params(
        url = "Target HTTP or HTTPS URL.",
        req_type = "Request method: get or post.",
        post_data = "Optional request body for POST requests.",
        get_params = "Optional query parameters for GET requests."
    )
)]
pub async fn http_request(url: String, req_type: String, post_data: Option<HashMap<String, String>>, get_params: Option<HashMap<String, String>>) -> Result<String, ToolExecutionError>
{
    let mut args: Map<String, Value> = Map::with_capacity(4);

    insert_arg(&mut args, "url", Some(url));
    insert_arg(&mut args, "req_type", Some(req_type));
    insert_arg(&mut args, "post_data", post_data);
    insert_arg(&mut args, "get_params", get_params);

    //Вызов
    let res: Py<PyAny> = call_py_tool(ToolRequest { module: "http_request", function: "make_request", args: Value::Object(args) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>| -> PyResult<String>
    {
        res.extract::<String>(py)
    }).map_err(ToolExecutionError::from_error);
}

/*
#[rig_tool(
    name = "dump_env",
    description = "Return available environment variables."
)]
pub async fn dump_env() -> Result<String, ToolExecutionError>
{
    //Вызов
    let res: Py<PyAny> = call_py_tool( ToolRequest { module: "dump_env", function: "dump_env", args: json!({ }) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}
*/

#[rig_tool(
    name = "find_files",
    description = "Recursively find files and directories in the workspace that match a glob pattern.",
    params(
        pattern = "Glob pattern to match file or directory names, for example '*.py' or 'config*'.",
        path = "Directory to search from, relative to the workspace."
    )
)]
pub async fn find_files(pattern: String, path: Option<String>) -> Result<String, ToolExecutionError>
{
    let mut args: Map<String, Value> = Map::with_capacity(2);

    insert_arg(&mut args, "pattern", Some(pattern));
    insert_arg(&mut args, "path", path);

    //Вызов
    let res: Py<PyAny> = call_py_tool( ToolRequest { module: "find_file", function: "find_files", args: Value::Object(args) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(
    name = "directory_contents",
    description = "List files and directories contained directly in a workspace directory.",
    params(
        path = "Path to the directory relative to the workspace."
    )
)]
pub async fn directory_contents(path: Option<String>) -> Result<String, ToolExecutionError>
{
    let mut args: Map<String, Value> = Map::with_capacity(1);

    insert_arg(&mut args, "path", path);

    //Вызов
    let res: Py<PyAny> = call_py_tool( ToolRequest { module: "directory_contents", function: "directory_contents",
    args: Value::Object(args) }).await?;

    //Сбор результата
    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py) //Принят return как String
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(
    name = "run_bandit",
    description = "Run Bandit SAST analysis for Python code. Returns detected issues and skipped files as JSON.",
    params(
        targets = "Files or directories to analyze. Optional; defaults to the workspace root.",
        recursive = "Whether to recursively discover files in target directories. Optional; defaults to true.",
        config_file = "Optional path to a Bandit configuration file, relative to the workspace.",
        agg_type = "Issue aggregation type: vuln or file. Optional; defaults to vuln.",
        sev_level = "Minimum issue severity to report: LOW, MEDIUM, or HIGH. Optional; defaults to LOW.",
        conf_level = "Minimum issue confidence to report: LOW, MEDIUM, or HIGH. Optional; defaults to LOW."
    )
)]
//Идите нахер со своей тонной option аргов господи питонисты хуевы
pub async fn run_bandit(targets: Option<Vec<String>>, recursive: Option<bool>, 
config_file: Option<String>, agg_type: Option<String>, sev_level: Option<String>,
conf_level: Option<String>) -> Result<String, ToolExecutionError>
{
    let mut args: Map<String, Value> = Map::new();

    insert_arg(&mut args, "targets", targets);
    insert_arg(&mut args, "recursive", recursive);
    insert_arg(&mut args, "config_file", config_file);
    insert_arg(&mut args, "agg_type", agg_type);
    insert_arg(&mut args, "sev_level", sev_level);
    insert_arg(&mut args, "conf_level", conf_level);

    let res: Py<PyAny> = call_py_tool(ToolRequest { module: "sast", function: "run_bandit", args: Value::Object(args) }).await?;

    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py)
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(
    name = "run_semgrep",
    description = "Run Semgrep SAST analysis using configured rules. The p/security-audit and p/secrets rulesets are always included.",
    params(
        targets = "Files or directories to analyze. Optional; defaults to the workspace root.",
        configs = "Additional Semgrep rulesets or configuration identifiers. Optional; p/security-audit and p/secrets are always included.",
        timeout = "Per-file analysis timeout in seconds. Optional; defaults to 5 seconds."
    )
)]
pub async fn run_semgrep(targets: Option<Vec<String>>, configs: Option<Vec<String>>, timeout: Option<i32>) -> Result<String, ToolExecutionError>
{
    let mut args: Map<String, Value> = Map::new();

    insert_arg(&mut args, "targets", targets);
    insert_arg(&mut args, "configs", configs);
    insert_arg(&mut args, "timeout", timeout);

    let res: Py<PyAny> = call_py_tool(ToolRequest { module: "sast", function: "run_semgrep", args: Value::Object(args) }).await?;

    return Python::attach(|py: Python<'_>|
    {
        res.extract::<String>(py)
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(
    name = "run_zap",
    description = "Runs OWASP ZAP in Docker, scans the target URL \
                   and returns a JSON report with discovered vulnerabilities.",
    params(
        url = "Target URL to scan (e.g., http://example.com).",
        do_ascan = "Enable active scanning (true/false). Default: false.",
        spider_timeout = "Spider timeout in seconds. Default: 600.",
        ascan_timeout = "Active scan timeout in seconds. Default: 1800."
    )
)]
pub async fn run_zap(
    url: String,
    do_ascan: Option<bool>,
    spider_timeout: Option<i32>,
    ascan_timeout: Option<i32>,
) -> Result<String, ToolExecutionError>
{
    // Collect arguments
    let mut args: Map<String, Value> = Map::with_capacity(4);
    insert_arg(&mut args, "url", Some(url));                 // required
    insert_arg(&mut args, "do_ascan", do_ascan);             // optional
    insert_arg(&mut args, "spider_timeout", spider_timeout); // optional
    insert_arg(&mut args, "ascan_timeout", ascan_timeout);   // optional

    // Call the Python function
    let res: Py<PyAny> = call_py_tool(ToolRequest {
        module: "dast",
        function: "run_zap",
        args: Value::Object(args),
    }).await?;

    // Extract the result (Python returns a JSON string)
    return Python::attach(|py: Python<'_>| {
        res.extract::<String>(py)
    }).map_err(ToolExecutionError::from_error);
}

#[rig_tool(
    name = "run_nuclei",
    description = "Runs Nuclei, scans the target URL using templates \
                   and returns a JSON report with discovered vulnerabilities.",
    params(
        target_url = "Target URL to scan (http protocol only).",
        template = "Template name from the agent folder (a .yaml file). \
                    If not specified, built-in Nuclei templates are used.",
        severity = "Comma-separated severity levels to search for \
                    (info,low,medium,high,critical). Default: all."
    )
)]
pub async fn run_nuclei(
    target_url: String,
    template: Option<String>,
    severity: Option<String>,
) -> Result<String, ToolExecutionError>
{
    // Collect arguments
    let mut args: Map<String, Value> = Map::with_capacity(3);
    insert_arg(&mut args, "target_url", Some(target_url)); // required
    insert_arg(&mut args, "template", template);           // optional
    insert_arg(&mut args, "severity", severity);           // optional

    // Call the Python function
    let res: Py<PyAny> = call_py_tool(ToolRequest {
        module: "dast",
        function: "run_nuclei",
        args: Value::Object(args),
    }).await?;

    // Extract the result (Python returns a JSON string)
    return Python::attach(|py: Python<'_>| {
        res.extract::<String>(py)
    }).map_err(ToolExecutionError::from_error);
}
