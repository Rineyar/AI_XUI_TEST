use bollard::Docker; //Для DinD
use bollard::models::ContainerCreateBody; //Для выбора конта
use bollard::plugin::ContainerCreateResponse; //Создание конта
use bollard::query_parameters::{RemoveContainerOptions, RemoveContainerOptionsBuilder}; //Удаление конта
use bollard::query_parameters::{UploadToContainerOptionsBuilder, UploadToContainerOptions}; //Загрузка туда
use bollard::body_full; //Для упаковки вектора в хз даже
use bollard::query_parameters::{DownloadFromContainerOptions, DownloadFromContainerOptionsBuilder}; //Загрузка оттуда
use futures_util::TryStreamExt; //Для сбора хз чего в вектор
use bollard::exec::{CreateExecOptions, CreateExecResults, StartExecResults}; //Исполнение
use bollard::plugin::ExecInspectResponse;

use std::{sync::OnceLock, time::Instant};
use std::sync::{Mutex, MutexGuard};
use std::collections::HashSet;

use tracing::{error, info, warn}; //Макросы логирования

type SandboxResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>; //Господи господи прости

static SANDBOX_MANAGER: OnceLock<SandboxManager> = OnceLock::new();

#[derive(Debug)]
struct SandboxManager
{
    docker: Docker,
    containers: Mutex<HashSet<String>>
}

pub async fn init_sandbox_manager(time: &Instant)
{
    rustls::crypto::aws_lc_rs::default_provider().install_default().expect("Не удалось установить Rustls CryptoProvider");

    let docker: Docker = Docker::connect_with_ssl_defaults().unwrap();

    info!("Проверка соединения - {}\t|\t{:?}", docker.ping().await
    .inspect_err(|err| error!("Отсутствует соединение с Docker - {}\t|\t{:?}", err, time.elapsed()))
    .expect("Отсутствует соединение с Docker"), time.elapsed());

    SANDBOX_MANAGER.set(SandboxManager { docker, containers: Mutex::new(HashSet::with_capacity(2)) })
    .inspect_err(|err| error!("Невозможное - {:?}\t|\t{:?}", err, time.elapsed()))
    .expect("Невозможное");
}

pub async fn create_container() -> SandboxResult<String>
{
    let manager: &SandboxManager = SANDBOX_MANAGER.get().ok_or("Невозможное")
    .inspect_err(|err| error!("{:?}", err)).expect("Невозможное");

    let config: ContainerCreateBody = ContainerCreateBody { image: Some("alpine:3.22".to_string()), working_dir: Some("/workspace".to_string()),
    cmd: Some(vec!["sleep".to_string(), "infinity".to_string()]), ..Default::default() };

    let created: ContainerCreateResponse = manager.docker.create_container(None, config).await
    .inspect_err(|err| error!("Ошибка создания контейнера - {:?}", err))?;

    let id: String = created.id;

    for warn in created.warnings.iter()
    {
        warn!("{}", warn);
    }

    manager.docker.start_container(&id, None).await
    .inspect_err(|err| error!("Ошибка запуска созданного контейнера - {:?}", err))?;

    {
        let mut mutex_guard: MutexGuard<'_, HashSet<String>> = manager.containers.lock()
        .inspect_err(|err| error!("Ошибка Mutex - {:?}", err)).expect("Ошибка Mutex");

        if !mutex_guard.insert(id.clone())
        {
            //return Err();

            //panic!("Какого-то Х 2 одинаковых id контейнера - {:?}\n", id);

            return Err(format!("In container creation getted a currently existed id - {:?}", id).into());
        }
    }

    return Ok(id);
}

pub async fn remove_container(id: &str) -> SandboxResult<()>
{
    let manager: &SandboxManager = SANDBOX_MANAGER.get().ok_or("Невозможное")
    .inspect_err(|err| error!("{:?}", err)).expect("Невозможное");

    {
        let mutex_guard: MutexGuard<'_, HashSet<String>> = manager.containers.lock()
        .inspect_err(|err| error!("Ошибка Mutex - {:?}", err)).expect("Ошибка Mutex");

        if !mutex_guard.contains(id)
        {
            warn!("Попытка удалить несуществующий контейнер - {:?}\n", id);

            return Err(format!("Container {:?} not found", id).into());
        }
    }

    let options: RemoveContainerOptions = RemoveContainerOptionsBuilder::default().force(true).build();

    manager.docker.remove_container(id, Some(options)).await
    .inspect_err(|err| error!("Ошибка удаления контейнера - {:?}", err))?;

    {
        let mut mutex_guard: MutexGuard<'_, HashSet<String>> = manager.containers.lock()
        .inspect_err(|err| error!("Ошибка Mutex - {:?}", err)).expect("Ошибка Mutex");

        mutex_guard.remove(id);
    }

    return Ok(());
}

pub async fn send_data(id: &str, path: &str, files: Vec<u8>) -> SandboxResult<()>
{
    let manager: &SandboxManager = SANDBOX_MANAGER.get().ok_or("Невозможное")
    .inspect_err(|err| error!("{:?}", err)).expect("Невозможное");

    {
        let mutex_guard: MutexGuard<'_, HashSet<String>> = manager.containers.lock()
        .inspect_err(|err| error!("Ошибка Mutex - {:?}", err)).expect("Ошибка Mutex");

        if !mutex_guard.contains(id)
        {
            warn!("Попытка получить доступ к несуществующему контейнеру - {:?}\n", id);

            return Err(format!("Container {:?} not found", id).into());
        }
    }

    let options: UploadToContainerOptions = UploadToContainerOptionsBuilder::new().path(&format!("/workspace/{}", path)).build();

    manager.docker.upload_to_container(id, Some(options), body_full(files.into())).await
    .inspect_err(|err| error!("Ошибка передачи файлов в контейнер - {:?}", err))?;

    info!("Файлы переданы в контейнер");

    return Ok(());
}

pub async fn load_data(id: &str, path: &str) -> SandboxResult<Vec<u8>>
{
    let manager: &SandboxManager = SANDBOX_MANAGER.get().ok_or("Невозможное")
    .inspect_err(|err| error!("{:?}", err)).expect("Невозможное");

    {
        let mutex_guard: MutexGuard<'_, HashSet<String>> = manager.containers.lock()
        .inspect_err(|err| error!("Ошибка Mutex - {:?}", err)).expect("Ошибка Mutex");

        if !mutex_guard.contains(id)
        {
            warn!("Попытка получить доступ к несуществующему контейнеру - {:?}\n", id);

            return Err(format!("Container {:?} not found", id).into());
        }
    }

    let options: DownloadFromContainerOptions = DownloadFromContainerOptionsBuilder::new().path(&format!("/workspace/{}", path)).build();

    let loaded: Vec<u8> = manager.docker.download_from_container(id, Some(options))
    .try_fold(Vec::new(), |mut buffer: Vec<u8>, chunk| async move
    {
        buffer.extend_from_slice(&chunk);
        Ok(buffer)
    }).await.inspect_err(|err| error!("Ошибка передачи файлов из контейнера - {:?}", err))?;

    info!("Файлы получены с контейнера - {} байт", loaded.len());

    return Ok(loaded);
}

pub async fn execute_command(id: &str, command: &str) -> SandboxResult<(String, Option<i64>)>
{
    let manager: &SandboxManager = SANDBOX_MANAGER.get().ok_or("Невозможное")
    .inspect_err(|err| error!("{:?}", err)).expect("Невозможное");

    {
        let mutex_guard: MutexGuard<'_, HashSet<String>> = manager.containers.lock()
        .inspect_err(|err| error!("Ошибка Mutex - {:?}", err)).expect("Ошибка Mutex");

        if !mutex_guard.contains(id)
        {
            warn!("Попытка получить доступ к несуществующему контейнеру - {:?}\n", id);

            return Err(format!("Container {:?} not found", id).into());
        }
    }

    let options: CreateExecOptions<&str> = CreateExecOptions {
    cmd: Some(vec!["sh", "-c", command]), attach_stdout: Some(true), attach_stderr: Some(true),
    working_dir: Some("/workspace"), ..Default::default() };

    let exec: CreateExecResults = manager.docker.create_exec(id, options).await
    .inspect_err(|err| error!("Ошибка передачи файлов в контейнер - {:?}", err))?;

    let result: StartExecResults = manager.docker.start_exec(&exec.id, None).await?;

    let mut output_text: String = String::new();

    if let StartExecResults::Attached { mut output, .. } = result 
    {
        while let Some(chunk) = output.try_next().await
        .inspect_err(|err| error!("Ошибка получения результата выполнения команды - {:?}", err))?
        {
            output_text.push_str(&chunk.to_string());
        }
    }

    let status: ExecInspectResponse = manager.docker.inspect_exec(&exec.id).await?;
    info!("Exit code: {:?}", status.exit_code);

    return Ok((output_text, status.exit_code));
}
