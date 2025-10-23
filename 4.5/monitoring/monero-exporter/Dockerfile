FROM python:3.11-slim

WORKDIR /app

# Install dependencies
RUN pip install --no-cache-dir \
    prometheus-client==0.19.0 \
    requests==2.31.0

# Copy exporter script
COPY exporter.py /app/exporter.py
RUN chmod +x /app/exporter.py

# Run as non-root user
RUN useradd -m -u 1000 exporter && \
    chown -R exporter:exporter /app
USER exporter

# Expose Prometheus metrics port
EXPOSE 9101

CMD ["python3", "/app/exporter.py"]
