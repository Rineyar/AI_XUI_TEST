import os
import json

__all__ = ["directory_contents"]

'''
path - путь к директории относительно папки agent
Функция возвращает тип, размер и имя содержимого
'''
def directory_contents(*, path):
    contents = []
    full_path = os.path.join("./agent", path)
    for entry in os.scandir(full_path):
        if entry.is_dir():
            kind = "DIR "  
        else:
            kind = "FILE"
        size = entry.stat().st_size
        contents.append({"kind":kind, "size":size, "name":entry.name})
    return json.dumps(contents)