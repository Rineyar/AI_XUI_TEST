use std::fs::{read_to_string, read, write, read_dir, DirEntry};
use std::process::{Command, Output};
use std::borrow::Cow;
use reqwest::blocking::{Client, Response, RequestBuilder};
use std::time::Duration;
use std::collections::HashMap;
use std::env::vars;

pub fn unsafe_read_text_file(path: &str) -> Result<String, std::io::Error>
{
    return read_to_string(path);
}

pub fn unsafe_read_bin_file(path: &str) -> Result<Vec<u8>, std::io::Error>
{
    return read(path);
}

pub fn unsafe_write_text_file(path: &str, text: &str) -> Result<(), std::io::Error>
{
    return write(path, text);
}

pub fn unsafe_write_bin_file(path: &str, data: &[u8]) -> Result<(), std::io::Error>
{
    return write(path, data);
}

pub fn unsafe_execute(command: &str) -> Result<(String, String, Option<i32>), std::io::Error>
{
    let command: Output = Command::new("sh").arg("-c").arg(command).output()?;

    let stdout: Cow<'_, str> = String::from_utf8_lossy(&command.stdout);

    let stderr: Cow<'_, str> = String::from_utf8_lossy(&command.stderr);

    let exitcode: Option<i32> = command.status.code();

    return Ok((stdout.into_owned(), stderr.into_owned(), exitcode));
}

pub fn unsafe_http_get(url: &str, headers: Option<HashMap<String, String>>, timeout: Option<u64>) -> Result<(u16, String), reqwest::Error>
{
    let client: Client = Client::builder().timeout(Duration::from_secs(timeout.unwrap_or(15u64))).build()?;

    let mut request: RequestBuilder = client.get(url);

    if let Some(headers) = headers
    {
        for (key, value) in headers.into_iter()
        {
            request = request.header(key, value);
        }
    }

    let response: Response = request.send()?;

    let status: u16 = response.status().as_u16();
    let body: String = response.text()?;

    Ok((status, body))
}

pub fn unsafe_http_post(url: &str, data: &str, headers: Option<HashMap<String, String>>, timeout: Option<u64>) -> Result<(u16, String), reqwest::Error>
{
    let client: Client = Client::builder().timeout(Duration::from_secs(timeout.unwrap_or(15u64))).build()?;

    let mut request: RequestBuilder = client.get(url);

    if let Some(headers) = headers
    {
        for (key, value) in headers.into_iter()
        {
            request = request.header(key, value);
        }
    }

    let response: Response = request.body(data.to_owned()).send()?;

    let status: u16 = response.status().as_u16();
    let body: String = response.text()?;

    Ok((status, body))
}

pub fn unsafe_directory_contents(path: &str) -> Result<Vec<String>, std::io::Error>
{
    let mut entries: Vec<String> = Vec::with_capacity(16);

    for entry in read_dir(path)?
    {
        let entry: DirEntry = entry?;

        let name: String = entry.file_name().to_string_lossy().into_owned();

        entries.push(name);
    }

    return Ok(entries);
}

pub fn unsafe_dump_env() -> HashMap<String, String>
{
    let mut result: HashMap<String, String> = HashMap::from_iter(vars().into_iter());

    result.remove("MAIN_MODEL_API_KEY");

    result.remove("OPENROUTER_API_KEY");

    result.remove("DEEPSEEK_LOCAL_API_KEY");

    result.remove("VEGA_AGENT_SESSION_ID");

    result.remove("OPENAI_API_KEY");

    result.remove("OPENAI_BASE_URL");

    result.remove("LOCAL_AGENT_MODEL");

    return result;
}
