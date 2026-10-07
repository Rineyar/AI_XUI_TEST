from pathlib import Path
import policy


def check_path(path, *, directory=None):
    requested = Path(path)
    if requested.is_absolute() or ".." in requested.parts:
        raise ValueError("Path should be inside the project")

    try:
        target = (policy.WORKSPACE / requested).resolve(strict=True)
        target.relative_to(policy.WORKSPACE)
    except Exception as error:
        raise ValueError("path doesn't exist or is out of the project directory") from error

    if directory is True and not target.is_dir():
        raise ValueError("Expected directory")
    if directory is False and not target.is_file():
        raise ValueError("Expected file")
    if target.is_file() and target.stat().st_size > policy.MAX_FILE_SIZE:
        raise ValueError("File is over maximum size")

    return target


def check_write_path(path):
    requested = Path(path)
    if requested.is_absolute() or ".." in requested.parts:
        raise ValueError("Path should be inside the project")

    try:
        target = (policy.WORKSPACE / requested).resolve(strict=False)
        target.relative_to(policy.WORKSPACE)
        parent = target.parent.resolve(strict=True)
        parent.relative_to(policy.WORKSPACE)
    except Exception as error:
        raise ValueError("path doesn't exist or is out of the project directory") from error

    if target.exists() and not target.is_file():
        raise ValueError("Expected path to file")

    return target
