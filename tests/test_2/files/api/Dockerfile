FROM python:3.9-slim
WORKDIR /app
COPY requirements.txt app.py ./
RUN pip install -r requirements.txt
ENV WORKER_COUNT=4
CMD sh -c "gunicorn --bind 0.0.0.0:8000 --workers ${WORKER_COUNT} app:app"
