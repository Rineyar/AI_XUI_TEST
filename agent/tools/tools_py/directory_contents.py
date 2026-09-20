import os
import json

__all__ = ["directory_contents"]

__tool_meta__ = {
    "directory_contents": {
        "description": "'directory_contents' tool takes directory path and returns kinds, sizes and names of it's contents"
    }
}

def directory_contents(path):
    contents = []
    for entry in os.scandir(path):
        if entry.is_dir():
            kind = "DIR "  
        else:
            kind = "FILE"
        size = entry.stat().st_size
        contents.append({"kind":kind, "size":size, "name":entry.name})
    return json.dumps(contents)