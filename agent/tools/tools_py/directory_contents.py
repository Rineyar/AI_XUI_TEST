import os
import json
import files

__all__ = ["directory_contents"]


def directory_contents(*, path):
    full_path = files.resolve_projects_path(path)
    contents = []
    for entry in os.scandir(full_path):
        kind = "DIR " if entry.is_dir() else "FILE"
        size = entry.stat().st_size
        contents.append({"kind": kind, "size": size, "name": entry.name})
    return json.dumps(contents)
