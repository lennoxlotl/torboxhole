FROM python:3.13-slim

WORKDIR /app

COPY main.py .
COPY config.py .
COPY database/ ./database/
COPY watchdog/ ./watchdog/

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

ENTRYPOINT ["python", "main.py"]