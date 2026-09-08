use rig::{client::{AgentClientExt, Client}, completion::Prompt};

use rig::rig_tool;
use rig::tool::ToolExecutionError;

#[rig_tool(description = "Return a sum of 2 32bit numbers")]
async fn tool_sum(a: i32, b: i32) -> Result<i32, ToolExecutionError> 
{
    return Ok(a + b);
}

#[tokio::main]
async fn main()
{
    let base_url: String = String::from("http://localhost:1234");

    let model = Client::from_url(&base_url).expect("Не грузит по ссылке");

    let agent = model.agent("openai/gpt-oss-20b").preamble("Try to tell sth").tool(ToolSum).build();

    let response = agent.prompt("Sum two 32-bit integers of your choice. You must use the tool.").await.expect("Не отвечает");

    println!("{}", response);
}