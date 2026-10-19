from pathlib import Path
import policy


def _validate_aks_path(rel_target: Path, target: Path):
    """Проверяет путь на ограничения изоляции базы знаний и системных файлов (AKS)."""
    # 1. Проверяем, не пытается ли путь обратиться к служебным папкам агента
    for part in rel_target.parts:
        if part.lower() in getattr(policy, "FORBIDDEN_DIR_NAMES", set()):
            raise ValueError(f"Доступ к системной директории '{part}' запрещен политикой безопасности (AKS)")

    # 2. Проверяем имя файла на совпадение с защищенными промптами и секретами
    filename = target.name
    patterns = getattr(policy, "FORBIDDEN_FILE_PATTERNS", [])
    for pattern in patterns:
        if pattern.search(filename):
            raise ValueError(f"Доступ к защищенному системному файлу '{filename}' запрещен политикой безопасности (AKS)")


def check_path(path, *, directory=None):
    requested = Path(path)
    if requested.is_absolute() or ".." in requested.parts:
        raise ValueError("Путь должен находиться внутри проекта")

    try:
        # Резолвим относительно WORKSPACE!
        target = (policy.WORKSPACE / requested).resolve(strict=True)
        rel_target = target.relative_to(policy.WORKSPACE)
    except Exception as error:
        raise ValueError("Путь не существует или выходит за пределы проекта") from error

    # AKS: проверка на запрещенные каталоги/файлы базы знаний и промптов
    _validate_aks_path(rel_target, target)

    if directory is True and not target.is_dir():
        raise ValueError("Ожидался каталог")
    if directory is False and not target.is_file():
        raise ValueError("Ожидался файл")
    if target.is_file() and target.stat().st_size > policy.MAX_FILE_SIZE:
        raise ValueError("Файл превышает допустимый размер")

    return target


def check_write_path(path):
    requested = Path(path)
    if requested.is_absolute() or ".." in requested.parts:
        raise ValueError("path should not be outside of a workspace")

    try:
        # Резолвим относительно WORKSPACE!
        target = (policy.WORKSPACE / requested).resolve(strict=False)
        rel_target = target.relative_to(policy.WORKSPACE)
    except Exception as error:
        raise ValueError("Path does not exist or is out side of workspace") from error

    # AKS: запрещаем запись в системные каталоги или перезапись промптов
    _validate_aks_path(rel_target, target)

    if target.exists() and not target.is_file():
        raise ValueError("Expected a file")

    return target


# =====================================================================
# УСТАРЕВШИЙ КОД (DEPRECATED):
# Блок ниже использовал policy.PROJECT_PATH вместо изолированного
# policy.WORKSPACE, поэтому заменен на актуальную версию выше.
# =====================================================================
# from pathlib import Path
# import policy
#
#
# def check_path(path, *, directory=None):
#     requested = Path(path)
#     if requested.is_absolute() or ".." in requested.parts:
#         raise ValueError("Путь должен находиться внутри проекта")
#
#     try:
#         # Резолвим относительно WORKSPACE!
#         target = (policy.PROJECT_PATH / requested).resolve(strict=True)
#         target.relative_to(policy.PROJECT_PATH)
#     except Exception as error:
#         raise ValueError("Путь не существует или выходит за пределы проекта") from error
#
#     if directory is True and not target.is_dir():
#         raise ValueError("Ожидался каталог")
#     if directory is False and not target.is_file():
#         raise ValueError("Ожидался файл")
#     if target.is_file() and target.stat().st_size > policy.MAX_FILE_SIZE:
#         raise ValueError("Файл превышает допустимый размер")
#
#     return target
#
#
# def check_write_path(path):
#     requested = Path(path)
#     if requested.is_absolute() or ".." in requested.parts:
#         raise ValueError("path should not be outside of a workspace")
#
#     try:
#         # Резолвим относительно WORKSPACE!
#         target = (policy.PROJECT_PATH / requested).resolve(strict=False)
#         target.relative_to(policy.PROJECT_PATH)
#         parent = target.parent.resolve(strict=True)
#         parent.relative_to(policy.PROJECT_PATH)
#     except Exception as error:
#         raise ValueError("Path does not exist or is out side of workspace") from error
#
#     if target.exists() and not target.is_file():
#         raise ValueError("Expected a file")
#
#     return target