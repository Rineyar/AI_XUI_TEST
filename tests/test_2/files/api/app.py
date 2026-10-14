from flask import Flask, request
import requests
from loguru import logger
import sys

app = Flask(__name__)

logger.remove()
logger.add(
    sys.stdout,
    format="<green>{time:YYYY-MM-DD HH:mm:ss.SSS}</green> | <level>{level: <8}</level> | <cyan>{name}</cyan>:<cyan>{function}</cyan>:<cyan>{line}</cyan> - <level>{message}</level>",
    colorize=True,
)

@app.before_request
def log_request():
    method = request.method
    path = request.path
    query_string = request.query_string.decode('utf-8')
    url = f"{path}?{query_string}" if query_string else path

    if request.method in ['POST', 'PUT', 'PATCH']:
        data = request.get_data(as_text=True)
    else:
        data = None

    log_message = f"{method} {url}"
    if data:
        log_message += f" {data}"
    logger.info(log_message)

@app.route('/')
def home():
    return 'Content loading service'

@app.route('/load')
def load_url():
    url = request.args.get('website')
    logger.info(f"Got request URL: {url}")
    if url:
        try:
            response = requests.get(url, timeout=5)
            return response.text
        except Exception as e:
            logger.exception("Error making request")
            return str(e)
    return "No URL provided"
