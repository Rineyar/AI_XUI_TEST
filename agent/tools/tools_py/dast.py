import subprocess
import json
import json
import subprocess
import time
import requests

__all__ = ["run_nuclei","run_zap"]

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
        command.extend(["-t", template])

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
    



__tool_meta__ = {
    "run_zap": {
        "description": "run_zap tool. Запускает OWASP ZAP в Docker, "
                       "сканирует указанный URL и возвращает JSON-отчёт."
    }
}


image = "zaproxy/zap-stable"
port = 8090
api_key = "zapkey123"
cont_name = "zap-daemon"


def _zap_api(path: str, params: dict = None) -> dict:
    if params is None:
        params = {}
    params["apikey"] = api_key
    url = f"http://localhost:{port}{path}"
    r = requests.get(url, params=params, timeout=30)
    r.raise_for_status()
    return r.json()


def _wait_for_zap(timeout: int = 90) -> None:
    start = time.time()
    while time.time() - start < timeout:
        try:
            _zap_api("/JSON/core/view/version/")
            return
        except Exception:
            time.sleep(2)
    raise Exception(f"ZAP не запустился за {timeout} секунд")


def _wait_for_scan(scan_type: str, scan_id: str, timeout: int = 600) -> None:
    start = time.time()
    while time.time() - start < timeout:
        status = _zap_api(f"/JSON/{scan_type}/view/status/",
                          {"scanId": scan_id})
        if int(status.get("status", 0)) >= 100:
            return
        time.sleep(3)
    raise Exception(f"{scan_type} не завершился за {timeout} секунд")


def run_zap(*, url: str,
            spider_timeout: int = 600,
            ascan_timeout: int = 1800,
            do_ascan: bool = False) -> str:

    output = {"success": False, "output": "", "error": ""}

    try:
        subprocess.run(["docker", "rm", "-f", cont_name],
                       capture_output=True, text=True)

        cmd = [
            "docker", "run", "-d",
            "--name", cont_name,
            "-p", f"{port}:{port}",
            image,
            "zap.sh", "-daemon",
            "-host", "0.0.0.0",
            "-port", str(port),
            "-config", f"api.key={api_key}",
            "-config", "api.addrs.addr.name=.*",
            "-config", "api.addrs.addr.regex=true",
        ]
        subprocess.run(cmd, check=True, capture_output=True, text=True)

        _wait_for_zap()

        spider_resp = _zap_api("/JSON/spider/action/scan/", {"url": url})
        spider_id = spider_resp["scan"]
        _wait_for_scan("spider", spider_id, timeout=spider_timeout)

        ascan_id = None
        if do_ascan:
            ascan_resp = _zap_api("/JSON/ascan/action/scan/", {"url": url})
            ascan_id = ascan_resp["scan"]
            _wait_for_scan("ascan", ascan_id, timeout=ascan_timeout)

        alerts_resp = _zap_api("/JSON/core/view/alerts/", {"baseurl": url})
        alerts = alerts_resp.get("alerts", [])

        output["output"] = {
            "target": url,
            "ascan_done": do_ascan,
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
        subprocess.run(["docker", "rm", "-f", cont_name],
                       capture_output=True, text=True)

    return json.dumps(output, indent=4, ensure_ascii=False)


if __name__ == "__main__":
    result = run_zap(url="http://host.docker.internal:3000", do_ascan=False)
    print(result)
    TARGET_URL = "http://127.0.0.1:8080"
    print(run_nuclei(TARGET_URL))
