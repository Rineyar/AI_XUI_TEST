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
        r = requests.get(url, params=params)
        return r.status_code, ""
    except requests.RequestException as e:
        return None, f"GET failed: {e}"

# Возвращает json
def make_post(url : str, data : dict | None = None):
    try:
        r = requests.post(url, data=data)
        return r.json(), ""
    except requests.RequestException as e:
        return None, f"POST failed: {e}"

# Делает http запрос и выводит в stdout
def make_request(url : str, req_type : str = "get", *,
                 post_data : dict | None = None, 
                 get_params : dict | None = None) -> None:
    output = {
        "success" : False,
        "output" : "",
        "error" : ""
    }

    r = None 
    error = ""
    match req_type:
        case "get":
            r, error = make_get(url, get_params)
        case "post":
            r, error = make_post(url, post_data)
        case _:
            error = "Incorrect request type."

    if r:
        output["success"] = True
        output["output"] = r
    else:
        output["error"] = error

    print(json.dumps(output, indent=4))
            
if __name__ == "__main__":
    make_request("https://ya.ru")
