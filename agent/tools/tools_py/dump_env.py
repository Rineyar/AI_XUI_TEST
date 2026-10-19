import os
import json
import re

__all__ = ["dump_env"]

__tool_meta__ = {
    "dump_env": {
        "description": "dump_env tool."
    }
}

# 1. Точные имена системных ключей
BLACK_LIST = [
    "MAIN_MODEL_API_KEY",
    "OPENROUTER_API_KEY",
    "VEGA_AGENT_SESSION_ID",
    "DEEPSEEK_LOCAL_API_KEY",
    "OPENAI_API_KEY",
    "OPENAI_BASE_URL",
    "LOCAL_AGENT_MODEL",
]

# 2. Паттерн для поиска любых чувствительных переменных окружения (токены, пароли, секреты)
SENSITIVE_ENV_PATTERN = re.compile(
    r"(?i)(key|secret|token|password|passwd|auth|credential|deepseek|openrouter)"
)


def get_env() -> str:
    env = dict(os.environ)

    # Удаление по точному совпадению
    for key in BLACK_LIST:
        env.pop(key, None)

    # Удаление по маске чувствительных данных
    keys_to_delete = [k for k in env if SENSITIVE_ENV_PATTERN.search(k)]
    for key in keys_to_delete:
        env.pop(key, None)

    return json.dumps(env, indent=4, ensure_ascii=False)


def dump_env() -> str:
    output = {
        "success": False,
        "output": "",
        "error": ""
    }

    dump = get_env()
    if dump:
        output["success"] = True
        output["output"] = dump
    else:
        output["error"] = "Could not get the environment."

    return json.dumps(output, indent=4, ensure_ascii=False)


if __name__ == "__main__":
    print(dump_env())