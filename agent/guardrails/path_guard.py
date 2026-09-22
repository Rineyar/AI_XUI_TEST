from pathlib import Path
import policy


def check_path(path, *, directory):
    requested = Path(path)
    if requested.is_absolute() or ".." in requested.parts:
        raise ValueError("Путь должен находиться внутри проекта")

    try:
        target = (policy.WORKSPACE / requested).resolve(strict=True)
        target.relative_to(policy.WORKSPACE)
    except Exception as error:
        raise ValueError("Путь не существует или выходит за пределы проекта") from error

    if directory and not target.is_dir():
        raise ValueError("Ожидался каталог")
    if not target.is_file():
        raise ValueError("Ожидался файл")

    return target
