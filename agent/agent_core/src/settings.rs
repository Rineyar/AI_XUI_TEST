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
    // "SAST_SKILLS:\n",
    // include_str!("../../skills/sast/sast_skills.md"),
    // "DAST_SKILLS:\n",
    // include_str!("../../skills/dast/dast_skills.md"),
    "TOOLS:\n",
    include_str!("../../tools/tools.md")
);

//Ссылка на модель на локалке
pub static BASE_URL: &str = "http://localhost:1234";

//Ссылка на местный дипсик
pub static DEEPSEEK_LOCAL_URL: &str = "https://deepcode.ci.nsu.ru/api";

//API ключ местного разлива
pub static DEEPSEEK_LOCAL_API_KEY: &str = include_str!("../DEEPSEEK_LOCAl_API_KEY.local");

//id модели на локалке
pub static MODEL_ID: &str = "openai/gpt-oss-20b";

//Максимум вывовов модели
pub const MAX_LLM_CALLS: usize = 32;