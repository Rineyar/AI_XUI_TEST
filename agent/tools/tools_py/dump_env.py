import os
import json

__all__ = ["dump_env"]

__tool_meta__ = {
    "dump_env": {
        "description": "dump_env tool."
    }
}

BLACK_LIST = ["MAIN_MODEL_API_KEY", "OPENROUTER_API_KEY", "VEGA_AGENT_SESSION_ID", "DEEPSEEK_LOCAL_API_KEY", "OPENAI_API_KEY", "OPENAI_BASE_URL", "LOCAL_AGENT_MODEL"]

# Создает строку из переменных окружения
def get_env() -> str:
    env = dict(os.environ)

    for key in BLACK_LIST:
        env.pop(key, None)

    return json.dumps(env, indent=4, ensure_ascii=False)

# Дампит переменные окружения в stdout как json
def dump_env() -> str:
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
    
    return json.dumps(output, indent=4, ensure_ascii=False)

if __name__ == "__main__":
    print(dump_env())
