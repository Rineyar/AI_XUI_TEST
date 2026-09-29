from pathlib import Path
import policy


def check_path(path, *, directory=None):
    requested = Path(path)
    if requested.is_absolute() or ".." in requested.parts:
        raise ValueError("Путь должен находиться внутри проекта") # path should not be outside of a workspace

    try:
        target = (policy.ROOT / requested).resolve(strict=True)
        target.relative_to(policy.WORKSPACE)
    except Exception as error:
        raise ValueError("Путь не существует или выходит за пределы проекта") from error # Path does not exist or is out side of workspace

    if directory is True and not target.is_dir():
        raise ValueError("Ожидался каталог") # Expected a directory
    if directory is False and not target.is_file():
        raise ValueError("Ожидался файл") # Expected a file
    if target.is_file() and target.stat().st_size > policy.MAX_FILE_SIZE:
        raise ValueError("Файл превышает допустимый размер") # File is bigger than 1000000 bytes

    return target


def check_write_path(path):
    requested = Path(path)
    if requested.is_absolute() or ".." in requested.parts:
        raise ValueError("path should not be outside of a workspace") # path should not be outside of a workspace

    try:
        target = (policy.ROOT / requested).resolve(strict=False)
        target.relative_to(policy.WORKSPACE)
        parent = target.parent.resolve(strict=True)
        parent.relative_to(policy.WORKSPACE)
    except Exception as error:
        raise ValueError("Path does not exist or is out side of workspace") from error  # Path does not exist or is out side of workspace

    if target.exists() and not target.is_file():
        raise ValueError("Expected a file") # Expected a file

    return target
