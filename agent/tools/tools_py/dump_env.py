import os
import json

__all__ = ["dump_env"]

__tool_meta__ = {
    "dump_env": {
        "description": "dump_env tool."
    }
}

# Можно будет расширить потом
BLACK_LIST = ["PASSWORD", "TOKEN", "SECRET"]

# Проверяет вхождения слов из черного списка
def is_env_secure(env : dict) -> bool:
    return not any(black in key for black in BLACK_LIST for key in env)

# Создает строку из переменных окружения
def get_env() -> str:
    env = dict(os.environ)

    if (is_env_secure(env)): 
        return json.dumps(env, indent=4)

    return ""

# Дампит переменные окружения в stdout как json
def dump_env() -> None:
    output = {
        "success" : False,
        "output" : "",
        "error" : ""
    }

    dump = get_env()
    if dump:
        output["success"] = True
        output["output"] = dump
    else:
        output["error"] = "Could not get the environment."
    
    print(json.dumps(output, indent=4))

if __name__ == "__main__":
    dump_env()
