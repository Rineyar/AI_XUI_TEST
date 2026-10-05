import os
import json
import files

__all__ = ["directory_contents"]

__tool_meta__ = {
    "directory_contents": {
        "description": "Returns files and directories in the specified workspace folder."
    }
}

def directory_contents(*, path = "."):
    contents = []
    resolved_path = files._resolve_workspace_path(path)
    for entry in os.scandir(resolved_path):
        kind = "DIR " if entry.is_dir() else "FILE"
        size = entry.stat().st_size
        contents.append({"kind": kind, "size": size, "name": entry.name})
    return json.dumps(contents)