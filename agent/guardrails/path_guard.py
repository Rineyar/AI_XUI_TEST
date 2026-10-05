from pathlib import Path
import policy


def check_path(path, *, directory=None):
    requested = Path(path)
    if requested.is_absolute() or ".." in requested.parts:
        raise ValueError("Путь должен находиться внутри проекта")

    try:
        # Резолвим относительно WORKSPACE!
        target = (policy.WORKSPACE / requested).resolve(strict=True)
        target.relative_to(policy.WORKSPACE)
    except Exception as error:
        raise ValueError("Путь не существует или выходит за пределы проекта") from error

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
        target.relative_to(policy.WORKSPACE)
        parent = target.parent.resolve(strict=True)
        parent.relative_to(policy.WORKSPACE)
    except Exception as error:
        raise ValueError("Path does not exist or is out side of workspace") from error

    if target.exists() and not target.is_file():
        raise ValueError("Expected a file")

    return target
