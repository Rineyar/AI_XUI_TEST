import os
import json

__all__ = ["directory_contents"]

def directory_contents(*, path = "."):
    contents = []
    full_path = os.path.abspath(os.path.join("./workspace", path))
    for entry in os.scandir(full_path):
        kind = "DIR " if entry.is_dir() else "FILE"
        size = entry.stat().st_size
        contents.append({"kind": kind, "size": size, "name": entry.name})
    return json.dumps(contents)