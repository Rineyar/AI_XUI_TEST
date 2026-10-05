import json
import re
from urllib.parse import urlsplit

import path_guard
import policy


__all__ = ["guard_select", "guard_response"]


RISKS = {"critical", "high", "medium", "low", "info"}
REQUIRED_FINDING_FIELDS = {"title", "risk", "confidence", "cwe", "path", "line", "potential_threat"}

SECRET_PATTERNS = (re.compile(r"\bgh[opusr]_[A-Za-z0-9]{20,}\b"),)
NAMED_SECRET = re.compile(r"(?i)\b(?:password|passwd|token|secret|api[_-]?key|access[_-]?key)"
                          r"\s*[:=]\s*[\"']?([^\s\"',;}{]{6,})")

SAFE_VALUES = {"redacted", "[redacted]", "<redacted>", "placeholder", "example"}

TOOL_GUARDS = {
    "read_file": lambda args: _guard_read_file(args),
    "write_file": lambda args: _guard_write_file(args),
    "make_request": lambda args: _guard_make_request(args),
    "dump_env": lambda args: _guard_dump_env(args),
    "find_files": lambda args: _guard_find_files(args),
    "directory_contents": lambda args: _guard_directory_contents(args),
    "run_bandit": lambda args: _guard_run_bandit(args),
    "run_semgrep": lambda args: _guard_run_semgrep(args),
}


def guard_select(request):
    try:
        function = request.get("function")
        args = request.get("args")

        if function in policy.TOOLS_IN_TEST:
            return {"allowed": True, "reason": "Unsafe Testing!"}

        guard = TOOL_GUARDS.get(function)
        if guard is None:
            return _deny("Для инструмента нет guard-функции")
        if function not in policy.ALLOWED_TOOLS:
            return _deny("Инструмент запрещён")

        guard(args)
    except Exception as error:
        return _deny(str(error))

    return {"allowed": True, "reason": "Allow"}


def _deny(reason):
    return {"allowed": False, "reason": reason}


def _guard_read_file(args):
    path_guard.check_path(args.get("filename"), directory=False)


def _guard_write_file(args):
    path_guard.check_write_path(args.get("filename"))
    if "text" not in args:
        raise ValueError("Не указан текст для записи")
    if len(args.get("text").encode("utf-8")) > policy.MAX_FILE_SIZE:
        raise ValueError("Записываемый файл превышает допустимый размер")


def _guard_make_request(args):
    url = urlsplit(args.get("url"))
    if url.scheme.lower() != "https":
        raise ValueError("Разрешены только HTTPS-запросы")
    if not url.hostname:
        raise ValueError("В URL отсутствует имя хоста")
    if url.username or url.password:
        raise ValueError("Учётные данные в URL запрещены")

    if args.get("req_type", "get").lower() not in {"get", "post"}:
        raise ValueError("Разрешены только GET и POST запросы")


# Исправить
def _guard_dump_env(args):
    if args:
        raise ValueError("dump_env не принимает аргументы")


def _guard_find_files(args):
    path_guard.check_path(args.get("path", "."), directory=True)
    pattern = args.get("pattern")
    if not pattern or len(pattern) > 100:
        raise ValueError("Некорректный шаблон поиска")
    if ".." in pattern or "/" in pattern or "\\" in pattern:
        raise ValueError("Шаблон должен описывать имя, а не путь")


def _guard_directory_contents(args):
    path_guard.check_path(args.get("path", "."), directory=True)


def _guard_run_bandit(args):
    for target in args.get("targets", ["."]):
        path_guard.check_path(target)

    config_file = args.get("config_file")
    if config_file:
        path_guard.check_path(config_file, directory=False)

    if args.get("agg_type", "vuln") not in {"vuln", "file", "baseline"}:
        raise ValueError("Недопустимый agg_type для Bandit")
    if args.get("sev_level", "LOW").upper() not in {"LOW", "MEDIUM", "HIGH"}:
        raise ValueError("Недопустимый sev_level для Bandit")
    if args.get("conf_level", "LOW").upper() not in {"LOW", "MEDIUM", "HIGH"}:
        raise ValueError("Недопустимый conf_level для Bandit")
    if args.get("recursive", True) not in {True, False}:
        raise ValueError("Недопустимое значение recursive")


def _guard_run_semgrep(args):
    for target in args.get("targets", ["."]):
        path_guard.check_path(target)

    for config in args.get("configs", []):
        if re.fullmatch(r"p/[A-Za-z0-9_.-]+", config):
            continue
        path_guard.check_path(config, directory=False)

    timeout = args.get("timeout", 5)
    if timeout < 1 or timeout > 60:
        raise ValueError("Timeout Semgrep должен быть от 1 до 60 секунд")


def guard_response(response):
    try:
        report = json.loads(response)
        findings = report["findings"]

        if _contains_secret(json.dumps(report, ensure_ascii=False)):
            return _deny("Ответ содержит потенциальные секретные данные")

        for number, finding in enumerate(findings, start=1):
            error = _check_finding(finding)
            if error:
                return _deny(f"Находка {number}: {error}")
    except Exception as error:
        return _deny(f"Некорректный ответ: {error}")

    return {"allowed": True, "reason": "Ответ прошёл проверку"}


def _check_finding(finding):
    missing = REQUIRED_FINDING_FIELDS - finding.keys()
    if missing:
        return f"отсутствуют поля: {', '.join(sorted(missing))}"

    text_fields = ("title", "potential_threat")
    for field in text_fields:
        if not finding[field].strip():
            return "текстовые поля не должны быть пустыми"

    risk = finding["risk"]
    if risk.lower() not in RISKS:
        return "указан недопустимый risk"

    confidence = finding["confidence"]
    if not 0 <= confidence <= 1:
        return "confidence должен быть числом от 0 до 1"

    cwe = finding["cwe"]
    if not re.fullmatch(r"CWE-[0-9]+", cwe):
        return "CWE должен иметь формат CWE-N"

    try:
        source = path_guard.check_path(finding["path"])
    except Exception as error:
        return f"некорректный путь: {error}"

    line = finding["line"]
    if line < 1:
        return "line должен быть положительным целым числом"

    with source.open("r", encoding="utf-8", errors="replace") as file:
        line_count = sum(1 for _ in file)
    if line > max(line_count, 1):
        return "указанная строка отсутствует в файле"

    return None


def _contains_secret(value):
    for pattern in SECRET_PATTERNS:
        if pattern.search(value):
            return True

    for match in NAMED_SECRET.finditer(value):
        secret = match.group(1).lower()
        if secret not in SAFE_VALUES and not secret.startswith(("[", "<")):
            return True

    return False

