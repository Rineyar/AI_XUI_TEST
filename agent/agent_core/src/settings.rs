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
pub static SAST_SKILLS: &str = include_str!("../../skills/sast_skills.md");
pub static DAST_SKILLS: &str = include_str!("../../skills/dast_skills.md");
pub static DOCKER_SKILLS: &str = include_str!("../../skills/docker_skills.md");

//Инструменты
pub static MAIN_TOOLS: &str = include_str!("../../tools/main_tools.md");
pub static SAST_TOOLS: &str = include_str!("../../tools/sast_tools.md");
pub static DAST_TOOLS: &str = include_str!("../../tools/dast_tools.md");
pub static DOCKER_TOOLS: &str = include_str!("../../tools/docker_tools.md");

//Максимум вывовов модели
pub const MAX_LLM_CALLS: usize = 200;

//Ссылка на сервер с моделью
pub static MODEL_SERVER_URL: &str = "https://deepcode.ci.nsu.ru/api";

//id модели
pub static MODEL_SERVER_ID: &str = "qwen3.8-flash-next";
