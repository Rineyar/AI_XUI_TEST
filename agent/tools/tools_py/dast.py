import subprocess
import json
import os
import time

def run_nuclei(target: str, template_id: str = None, severity: str = "info,low,medium,high,critical"):

    command = [
        "nuclei",
        "-u", target,
        "-severity", severity,
        "-j"
        #"-silent"  
    ]

    if template_id:
        command.extend(["-id", template_id])

    subprocess.run(command, text=True)
    
    findings = []
    # for line in result.stdout.strip().split("\n"):
    #     if line:
    #         try:
    #             findings.append(json.loads(line))
    #         except json.JSONDecodeError:
    #             continue
    return findings

LAB_PATH = "nuclei-templates-labs/http/cves/2024/CVE-2024-55416"
TARGET_URL = "http://172.30.0.2:8000"
TEMPLATE_FILE = "cve-2024-55416.yaml"

subprocess.run(["docker-compose", "up", "-d"], cwd=LAB_PATH, check=True)
time.sleep(5)
template_path = os.path.join(LAB_PATH, TEMPLATE_FILE)

findings = run_nuclei(TARGET_URL, template_id="cve-2024-55416")
for f in findings:
    print(f"[{f['info']['severity']}] {f['info']['name']} — {f['matched-at']}\n")

subprocess.run(["docker-compose", "down"], cwd=LAB_PATH, check=True)