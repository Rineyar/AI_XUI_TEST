import os

__all__ = ["directory_contents"]

__tool_meta__ = {
    "directory_contents": {
        "description": "'directory_contents' принимает путь относительно папки agent, выводит тип, размер и имя содержимого"
    }
}

def directory_contents(path):
    contents = []
    path = r"./agent/"+path
    for entry in os.scandir(path):
        if entry.is_dir():
            kind = "DIR "  
        else:
            kind = "FILE"
        size = entry.stat().st_size
        print(kind, size, entry.name, "\n")
    return contents
