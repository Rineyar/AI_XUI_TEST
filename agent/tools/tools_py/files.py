def read_file(*, filename):
    with open(filename, "r") as file:
        return file.read()

def write_file(*, filename, text):
    with open(filename, "w") as file:
        file.write(text)

__all__ = ["read_file", "write_file"]