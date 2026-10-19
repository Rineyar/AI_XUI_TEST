import json
import re
import unicodedata
from typing import Dict, List, Tuple

__all__ = ["validate_prompt"]

# 1. Критические маркеры инъекций разметки (мгновенная блокировка без подсчета баллов)
CRITICAL_DELIMITERS: List[Tuple[str, re.Pattern]] = [
    (
        "SPECIAL_TOKEN_INJECTION",
        re.compile(
            r"(?i)(?:<\|im_start\|>|<\|im_end\|>|<\|system\|>|<\|user\|>|<\|assistant\|>|[system]|[inst]|<s>|</s>|\[/inst\])"
        ),
    ),
    (
        "SYSTEM_CODEBLOCK_INJECTION",
        re.compile(r"(?i)```(?:system|developer|preamble)\b"),
    ),
]

# 2. Эвристические правила со скорингом весов (пороговое срабатывание)
# Порог блокировки по умолчанию — 3 балла
SCORE_THRESHOLD = 3

HEURISTIC_RULES: List[Tuple[str, int, re.Pattern]] = [
    # Попытка сброса системных инструкций и ограничений безопасности (высокий вес)
    (
        "SYSTEM_OVERRIDE",
        3,
        re.compile(
            r"(?i)\b(?:ignore|disregard|bypass|override|forget)\s+(?:all\s+)?(?:system|developer|safety|guardrail|runtime|preamble)\s+(?:instructions|rules|prompts|guidelines|constraints|policy)\b"
        ),
    ),
    # Явные триггеры смены режима на DAN / Developer Mode (высокий вес)
    (
        "JAILBREAK_ROLE_MODE",
        3,
        re.compile(
            r"(?i)\b(?:do\s+anything\s+now|developer\s+mode|unfiltered\s+mode|jailbreak(?:ed)?\s+mode|god\s+mode)\b"
        ),
    ),
    # Смена ролевой модели с полным отказом от текущих рамок (средний вес)
    (
        "ROLE_REVERSAL_STRICT",
        2,
        re.compile(
            r"(?i)\b(?:you\s+are\s+no\s+longer\s+(?:an?\s+)?ai|from\s+now\s+on\s+you\s+(?:have\s+no\s+rules|are\s+completely\s+unrestricted)|act\s+as\s+an?\s+(?:evil|unfiltered|rogue)\s+ai)\b"
        ),
    ),
    # Попытка вытянуть системный промпт или мета-инструкции (средний вес)
    (
        "SYSTEM_PROMPT_LEAK",
        2,
        re.compile(
            r"(?i)\b(?:output|repeat|print|reveal|display|show|dump)\s+(?:your\s+)?(?:initial|system|exact|complete|full)\s+(?:prompt|instructions|preamble|meta-prompt)\b"
        ),
    ),
    # Требование игнорировать любые этические нормы в связке с командой (средний вес)
    (
        "ETHICAL_BYPASS",
        2,
        re.compile(
            r"(?i)\b(?:disregard|without)\s+(?:any\s+)?(?:moral|ethical|safety)\s+(?:restrictions|filters|protocols)\b"
        ),
    ),
    # Маскировка под гипотетический сценарий для обхода ограничений (низкий вес)
    (
        "HYPOTHETICAL_EVASION",
        1,
        re.compile(
            r"(?i)\b(?:in\s+a\s+fictional\s+world\s+without\s+rules|pretend\s+there\s+are\s+no\s+guidelines)\b"
        ),
    ),
]


def _normalize_text(text: str) -> str:
    """Устраняет попытки обфускации: нормализует Unicode, удаляет zero-width символы и лишние пробелы."""
    normalized = unicodedata.normalize("NFKD", text)
    # Удаление невидимых символов форматирования (Zero-Width Space, Joiner, BOM)
    cleaned = re.sub(r"[\u200B-\u200D\uFEFF\u202A-\u202E]", "", normalized)
    return " ".join(cleaned.split())


def validate_prompt(prompt_text: str) -> str:
    """Анализирует текст входящего запроса на атаки DAN, сброс ролей и prompt injection.

    Возвращает JSON с полями allowed, reason, score и списком violations.
    """
    if not prompt_text or not prompt_text.strip():
        return json.dumps(
            {"allowed": True, "reason": "Empty prompt", "score": 0, "violations": []},
            ensure_ascii=False,
        )

    clean_prompt = _normalize_text(prompt_text)

    # 1. Проверка на жесткие служебные инъекции (моментальный бан)
    for rule_name, pattern in CRITICAL_DELIMITERS:
        if pattern.search(clean_prompt):
            return json.dumps(
                {
                    "allowed": False,
                    "reason": f"Critical prompt delimiter injection detected: {rule_name}",
                    "score": 999,
                    "violations": [rule_name],
                },
                ensure_ascii=False,
            )

    # 2. Балльная проверка по эвристикам
    total_score = 0
    violations: List[str] = []

    for rule_name, weight, pattern in HEURISTIC_RULES:
        if pattern.search(clean_prompt):
            total_score += weight
            violations.append(f"{rule_name}(+{weight})")

    # 3. Вынесение вердикта на основе порога
    if total_score >= SCORE_THRESHOLD:
        return json.dumps(
            {
                "allowed": False,
                "reason": f"Potential DAN/Jailbreak detected (Score: {total_score}/{SCORE_THRESHOLD})",
                "score": total_score,
                "violations": violations,
            },
            ensure_ascii=False,
        )

    return json.dumps(
        {
            "allowed": True,
            "reason": "OK",
            "score": total_score,
            "violations": violations,
        },
        ensure_ascii=False,
    )