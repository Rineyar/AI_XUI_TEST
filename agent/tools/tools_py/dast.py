import subprocess
import json

__all__ = ["run_nuclei"]

'''
target_url - url адрес цели. Допускается только http протокол
template - Путь к шаблону из папки agent. Шаблон должен быть в формате .yaml. По умолчанию использует свои шаблоны. 
severity - Важность искомых угроз. По умолчанию ищет все.
'''
def run_nuclei(target_url: str, template: str = None, severity: str = "info,low,medium,high,critical"):
    output = {
            "success" : False,
            "output" : "",
            "error" : ""
        }
    command = ["nuclei", "-u", target_url, "-type", "http", "-max-host-error", "0", "-severity", severity, "-j", '-silent']

    if template:
        command.extend(["-t", r"./agent/"+template])

    try:
        nuclei = subprocess.run(command, capture_output=True, text=True)
        output["output"] = []
        for line in nuclei.stdout.strip().split('\n'):
            if line:
                try:
                    output["output"].append(json.loads(line))
                except json.JSONDecodeError:
                    continue
        output["success"] = True
    except Exception as e:
        output["error"] = f"Exception: {e}"
    
    return json.dumps(output, indent=4, ensure_ascii=False)


if __name__ == "__main__":
    TARGET_URL = "http://127.0.0.1:8080"
    print(run_nuclei(TARGET_URL))
    
