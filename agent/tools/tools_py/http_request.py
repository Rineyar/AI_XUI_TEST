# Требует requests для работы
import requests
import json

__all__ = ["make_request"]

__tool_meta__ = {
    "make_request": {
        "description": "make_request tool."
    }
}

# Возвращает код ответа
def make_get(url : str, params : dict | None = None):
    try:
        r = requests.get(url, params=params, timeout=(5, 15), allow_redirects=False)
        return {
                    "status_code": r.status_code,
                    "headers": dict(r.headers),
                    "body": r.text[:10_000],
                    "truncated": len(r.text) > 10_000
                }, ""
    except requests.RequestException as e:
        return None, f"GET failed: {e}"

# Возвращает json
def make_post(url : str, data : dict | None = None):
    try:
        r = requests.post(url, data=data, timeout=(5, 15), allow_redirects=False)
        body = r.text

        return {
                    "status_code": r.status_code,
                    "headers": dict(r.headers),
                    "body": body[:10_000],
                    "truncated": len(body) > 10_000
                }, ""
    except requests.RequestException as e:
        return None, f"POST failed: {e}"

# Делает http запрос и выводит в stdout
def make_request(*, url : str, req_type : str = "get",
                 post_data : dict | None = None, 
                 get_params : dict | None = None) -> str:
    output = {
        "success" : False,
        "output" : "",
        "error" : ""
    }

    r = None 
    error = ""
    match req_type.lower():
        case "get":
            r, error = make_get(url, get_params)
        case "post":
            r, error = make_post(url, post_data)
        case _:
            error = "Incorrect request type."

    if error:
        output["error"] = error
    else:
        output["success"] = True
        output["output"] = r

    return json.dumps(output, indent=4, ensure_ascii=False)
            
if __name__ == "__main__":
    print(make_request(url="https://ya.ru"))
