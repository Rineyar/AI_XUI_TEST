import subprocess
import json
import time
import requests
import files
import os
from pathlib import Path
import uuid

__all__ = ["run_nuclei","run_zap"]

__tool_meta__ = {
    "run_zap": {
        "description": "run_zap tool. Runs OWASP ZAP in Docker, "
                       "scans target URL and returns JSON-report."
    },
    "run_nuclei": {
            "description": "Runs Nuclei — a DAST tool for checking an HTTP target against templates."  
    }
}

def run_nuclei(target_url: str, template: str = None, severity: str = "info,low,medium,high,critical", timeout: int = 120):
    output = {
            "success" : False,
            "output" : "",
            "error" : ""
        }
    command = ["nuclei", "-u", target_url, "-type", "http", "-max-host-error", "0", "-severity", severity, "-j", '-silent']

    if template:
        templates_root = Path(
            os.environ["NUCLEI_TEMPLATES_DIR"]
        ).resolve(strict=True)

        requested = Path(template)

        if requested.is_absolute() or ".." in requested.parts:
            raise ValueError("Недопустимый путь к шаблону")

        template_path = (templates_root / requested).resolve(strict=True)

        if not template_path.is_relative_to(templates_root):
            raise ValueError("Шаблон находится вне каталога Nuclei")

        command.extend(["-t", str(template_path)])

    try:
        nuclei = subprocess.run(command, capture_output=True, text=True, timeout=timeout)

        if nuclei.returncode != 0:
            raise RuntimeError(
                f"Nuclei exited with code {nuclei.returncode}: "
                f"{nuclei.stderr.strip()}"
            )

        output["output"] = []
        for line in nuclei.stdout.strip().split('\n'):
            if line:
                try:
                    output["output"].append(json.loads(line))
                except json.JSONDecodeError:
                    continue
        output["success"] = True

    except subprocess.TimeoutExpired:
        output["error"] = f"Nuclei timed out after {timeout} seconds"

    except Exception as e:
        output["error"] = f"Exception: {e}"
    
    return json.dumps(output, indent=4, ensure_ascii=False)

IMAGE = "zaproxy/zap-stable"
PORT = 8090
KEY = "zapkey123"
cont_name = "zap-daemon"
ZAP_BASE_URL = os.environ.get(
    "ZAP_BASE_URL",
    "http://sandbox:8090"
).rstrip("/")

def _zap_api(path, params=None, timeout=30):
    if params is None:
        params = {}
    params["apikey"] = KEY
    url = f"{ZAP_BASE_URL}{path}"
    try:
        r = requests.get(url, params=params, timeout=(5, timeout))
        r.raise_for_status()
        return r.json()
    except requests.exceptions.ReadTimeout:
        raise Exception(f"Read timeout on {path}") from None

def _wait_for_zap(container_name: str, timeout: int = 90) -> None:
    start = time.monotonic()
    while time.monotonic() - start < timeout:
        state = subprocess.run(
            [
                "docker", "inspect",
                "-f", "{{.State.Status}}",
                container_name
            ],
            capture_output=True,
            text=True,
            timeout=5
        )

        if state.returncode != 0:
            raise RuntimeError(
                f"Не удалось проверить контейнер ZAP: "
                f"{state.stderr.strip()}"
            )

        container_status = state.stdout.strip()

        if container_status in ("exited", "dead"):
            logs = subprocess.run(
                ["docker", "logs", "--tail", "30", container_name],
                capture_output=True,
                text=True,
                timeout=5
            )

            raise RuntimeError(
                f"ZAP завершился со статусом {container_status}.\n"
                f"{(logs.stdout + logs.stderr)[-3000:]}"
            )
        try:
            _zap_api("/JSON/core/view/version/")
            return
        except Exception:
            pass
        time.sleep(2)

    raise TimeoutError(
        f"ZAP API не стал доступен за {timeout} секунд."
    )

def run_zap(*, target_url: str, plan_template: str = "plan.yaml.tpl", timeout: int = 1800) -> str:
    output = {"success": False, "output": "", "error": ""}
    plan_host_path = None

    plans_dir = Path(
                os.environ["ZAP_PLANS_DIR"]
            ).resolve(strict=True)

    try:
        os.makedirs(plans_dir, exist_ok=True)
        subprocess.run(["docker", "rm", "-f", cont_name], capture_output=True, text=True)

        cmd = [
            "docker", "run", "-d",
            "--name", cont_name,
            "--add-host=host.docker.internal:host-gateway",
            "-p", f"{PORT}:{PORT}",
            "-v", f"{os.path.abspath(plans_dir)}:/zap/wrk:rw",
            IMAGE,
            "zap.sh", "-daemon",
            "-host", "0.0.0.0",
            "-port", str(PORT),
            "-config", f"api.key={KEY}",
            "-config", "api.addrs.addr.name=.*",
            "-config", "api.addrs.addr.regex=true",
        ]
        subprocess.run(cmd, check=True, capture_output=True, text=True)
        _wait_for_zap("zap-daemon")

        prob = _zap_api("/JSON/core/action/accessUrl/", {"url": target_url, "followRedirects": "true"})
        if not prob.get("accessUrl"):
            raise Exception(f"Цель недоступна из ZAP: {target_url}")

        requested = Path(plan_template)

        if requested.is_absolute() or ".." in requested.parts:
            raise ValueError("Недопустимый путь к плану")

        template_path = (plans_dir / requested).resolve(strict=True)

        if not template_path.is_relative_to(plans_dir):
            raise ValueError("План находится вне каталога Zap")

        with open(template_path) as f:
            plan_content = f.read().replace("{{TARGET}}", target_url)

        plan_filename = f"plan_{int(time.time())}.yaml"
        plan_host_path = os.path.join(plans_dir, plan_filename)
        with open(plan_host_path, "w") as f:
            f.write(plan_content)

        run_resp = _zap_api("/JSON/automation/action/runPlan/",
                            {"filePath": f"/zap/wrk/{plan_filename}"})
        plan_id = run_resp.get("planId")
        if not plan_id:
            raise Exception(f"API не вернул planId: {run_resp}")
        if plan_id == "":
            raise Exception(f"planId не валидный: {run_resp}")

        start = time.time()
        while time.time() - start < timeout:
            p = _zap_api("/JSON/automation/view/planProgress/", {"planId": plan_id})
            if p.get("finished"):
                break
            time.sleep(3)
        else:
            raise Exception(f"План не завершился за {timeout} секунд")

        alerts =_zap_api("/JSON/core/view/alerts/").get("alerts", [])

        output["output"] = {
            "target": target_url,
            "plan_id": plan_id,
            "alerts_count": len(alerts),
            "alerts": alerts,
        }
        output["success"] = True

    except subprocess.CalledProcessError as e:
        output["error"] = f"Docker error: {e.stderr or e.stdout or e}"
    except requests.RequestException as e:
        output["error"] = f"ZAP API error: {e}"
    except Exception as e:
        output["error"] = f"Exception: {e}"
    finally:
        if plan_host_path and os.path.exists(plan_host_path):
            os.remove(plan_host_path)
        try:
            subprocess.run(["docker", "rm", "-f", cont_name], capture_output=True, text=True)
        except Exception as e:
            output["success"] = False
            output["error"] += f" Cleanup error: {e}"
    return json.dumps(output, indent=4, ensure_ascii=False)


if __name__ == "__main__":
    result = run_zap(target_url="http://host.docker.internal:3000", plan_template="plan.yaml.tpl")
    print(result)
