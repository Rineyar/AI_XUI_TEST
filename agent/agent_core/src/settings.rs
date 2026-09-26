/*Пока что они не нужны по раздельности*/
// pub static SYSTEM_PROMPT: &str = include_str!("../../system_prompt.md");
// pub static MAIN_SKILLS: &str = include_str!("../../skills/main_skills.md");
// pub static SAST_SKILLS: &str = include_str!("../../skills/sast/sast_skills.md");
// pub static DAST_SKILLS: &str = include_str!("../../skills/dast/dast_skills.md");

//system_prompt. Собирается из файлов на этапе компиляции
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

//Максимум вывовов модели
pub const MAX_LLM_CALLS: usize = 16;

//Ссылка на модель на локалке
pub static MODEL_LOCAL_URL: &str = "http://localhost:1234";

//id модели на локалке
pub static MODEL_LOCAL_ID: &str = "openai/gpt-oss-20b";

//Ссылка на местный дипсик
pub static DEEPSEEK_LOCAL_URL: &str = "https://deepcode.ci.nsu.ru/api";

//id дисписка
pub static MODEL_DEEPSEEK_ID: &str = "deepseek-ai/DeepSeek-V4-Flash-0731";
