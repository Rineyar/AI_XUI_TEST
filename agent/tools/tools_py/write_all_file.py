def write_file(filename, text):
    with open(filename, "w") as file:
        file.write(text)

    return True