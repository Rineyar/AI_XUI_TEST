import glob
__all__ = ["find_files"]

__tool_meta__ = {
    "find_files":{
        "description":"find_files tool."
    }
}




def find_files(*, pattern: str, path: str = ".") -> str:
    matches = glob.glob(f"{path}/**/{pattern}", recursive=True)
    matches += glob.glob(f"{path}/{pattern}")
    unique = sorted(set(matches))
    return "\n".join(unique) if unique else "(no matches)"

print(find_files(pattern="t*"))
