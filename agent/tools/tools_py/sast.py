import json
from bandit.core import config as b_config
from bandit.core import manager as b_manager

__all__ = ["run_bandit", "run_semgrep"]

__tool_meta__ = {
    "run_bandit": {
        "description": "run_bandit tool."
    },
    "run_semgrep": {
        "description": "run_semgrep tool."
    }
}

def run_bandit(*, targets: list = ["."], 
               recursive: bool = True,
               config_file: str | None = None, 
               agg_type: str = "vuln",
               sev_level: str = "LOW",
               conf_level: str = "LOW") -> str:
    """ Запустить Bandit - инструмент 
    для SAST анализа Python кода """

    output = {
        "success" : False,
        "output" : "",
        "error" : ""
    }

    try:
        # Создаем конфигурация
        bc = b_config.BanditConfig(config_file)
        # Создаем менеджер
        bm = b_manager.BanditManager(bc, agg_type)
        # Обнаруживаем файлы
        bm.discover_files(targets, recursive)
        # Тестируем
        bm.run_tests()
        # Получаем проблемы
        issues = bm.get_issue_list(sev_level=sev_level, 
                                conf_level=conf_level)

        # Переписываем проблемы как словари
        issues = [issue.as_dict() for issue in issues]
        # Записываем пропущенные файлы
        skipped = [{"fname": fname, "reason": reason} for fname, reason in bm.get_skipped()]

        output["output"] = {
            "result": issues,
            "skipped": skipped
        }
        output["success"] = True

    except Exception as e:
        output["error"] = f"Exception: {e}"


    return json.dumps(output, indent=4)

def run_semgrep(*, targets: list = ["."]) -> str:
    """ Запустить Semgrep - инструмент 
    для SAST анализа кода """
    
    output = {
        "success" : False,
        "output" : "",
        "error" : ""
    }

    return json.dumps(output, indent=4)
    
if __name__ == "__main__":
    print(run_bandit())
    print(run_semgrep())