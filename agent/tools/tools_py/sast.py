import json
import subprocess
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

def run_semgrep(*, targets: list = ["."], 
                configs: list = [], 
                timeout: int = 5) -> str:
    """ Запустить Semgrep - инструмент 
    для SAST анализа кода.\n
    targets - Список целей для анализа\n
    configs - Список semgrep конфигов\n
    timeout - Лимит времени выполнения одного файла\n
    В configs всегда добавляются p/security-audit, p/secrets """
    
    output = {
        "success" : False,
        "output" : "",
        "error" : ""
    }

    try:
        # создаем команду
        cmd = ["semgrep", "scan", "--json", "--quiet", 
               "--metrics=off", f"--timeout={timeout}"] 
        # Добавляем конфиги в команду
        configs += ["p/security-audit", "p/secrets"]
        for c in configs:
            cmd += ["--config", c]

        # Запускаем семгреп как процесс
        semgrep_p = subprocess.run(cmd + targets,
                                   capture_output=True,
                                   text=True)

        if semgrep_p.returncode > 1:
            raise Exception(f"Semgrep exited with code: {semgrep_p.returncode}")

        output["success"] = True
        output["output"] = json.loads(semgrep_p.stdout)

    except Exception as e:
        output["error"] = f"Exception: {e}"

    return json.dumps(output, indent=4)
    
if __name__ == "__main__":
    # print(run_bandit())
    print(run_semgrep())