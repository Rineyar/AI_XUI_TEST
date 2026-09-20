__all__ = ["guard_select"]

def guard_select(request):
    function = request["function"]
    args = request["args"]

    if function == "read_file":
        return _guard_read_file(args)

    elif function == "write_file":
        return _guard_write_file(args)
    elif function == "find_files":
        return {"allowed": True, "reason": "Unsafe testing"}
    elif function == "directory_contents":
        return {"allowed": True, "reason": "Unsafe testing"}

    return {"allowed": False, "reason": "Guard not covered this call"}

def _guard_read_file(args):
    return _guard_path_check(args)

def _guard_write_file(args):
    return _guard_path_check(args)

def _guard_path_check(args):
    from pathlib import Path

    WORKSPACE = Path("../workspace").resolve()

    filename = args.get("filename")

    if not filename:
        return {"allowed": False, "reason": "Filename is missing"}

    target = (WORKSPACE / filename).resolve()

    print(f"[GUARD] workspace={WORKSPACE}")
    print(f"[GUARD] filename={filename}")
    print(f"[GUARD] target={target}")

    try:
        target.relative_to(WORKSPACE)
    except ValueError:
        return {"allowed": False, "reason": "Access outside workspace is denied"}

    return {"allowed": True, "reason": "Allowed"}