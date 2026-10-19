import os

def get_status():
    env_name = os.getenv("APPLICATION_ENV", "unknown")
    return {"status": "running", "environment": env_name}

if __name__ == "__main__":
    print(get_status())