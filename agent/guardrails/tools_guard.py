import json
import re

import path_guard
import policy


RISKS = {"critical", "high", "medium", "low", "info"}
REQUIRED_FINDING_FIELDS = {"title", "risk", "confidence", "cwe", "path", "line", "potential_threat"}

SECRET_PATTERNS = (re.compile(r"\bgh[opusr]_[A-Za-z0-9]{20,}\b"),)
NAMED_SECRET = re.compile(r"(?i)\b(?:password|passwd|token|secret|api[_-]?key|access[_-]?key)"
                          r"\s*[:=]\s*[\"']?([^\s\"',;}{]{6,})")

SAFE_VALUES = {"redacted", "[redacted]", "<redacted>", "placeholder", "example", "changeme"}


def guard_select(request):
    try:
        function = request.get("function")
        args = request.get("args")

        if function in policy.TOOLS_IN_TEST:
            return {"allowed": True, "reason": "Testing"}

        if function not in policy.ALLOWED_TOOLS:
            return _deny("Инструмент запрещён")
        

        if function == "read_file":
            path_guard.check_path(args.get("filename"))
        else:
            path_guard.check_path(args.get("path", "."), directory=True)
    except Exception as error:
        return _deny(str(error))

    return {"allowed": True, "reason": "Разрешено"}


def _deny(reason):
    return {"allowed": False, "reason": reason}


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


__all__ = ["guard_select", "guard_response"]
