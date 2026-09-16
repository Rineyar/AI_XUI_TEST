import sys
import json


def check(tool_name, args):
    if tool_name == "read_file" and args.get("filename") == "forbidden.txt":
        return False, "Access to forbidden.txt is denied"

    return True, "Allowed"


if __name__ == "__main__":
    request = json.load(sys.stdin)

    allowed, reason = check(
        request["tool"],
        request["args"]
    )

    response = {
        "allowed": allowed,
        "reason": reason
    }

    print(json.dumps(response))