FROM python:3.11-slim

WORKDIR /app

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

COPY requirements.txt /app/requirements.txt
RUN pip install --no-cache-dir -r /app/requirements.txt

COPY app /app

EXPOSE 8000

CMD ["gunicorn", "-b", "0.0.0.0:8000", "app:app", "--workers", "2", "--access-logfile", "-", "--error-logfile", "-"]

# Copy binary and runtime assets
COPY --from=builder /usr/src/jeebs/target/release/jeebs /usr/local/bin/jeebs
COPY --from=builder /usr/src/jeebs/VERSION /app/VERSION
COPY --from=builder /usr/src/jeebs/migrations /app/migrations

ENV RUST_LOG=info
EXPOSE 8080

VOLUME ["/data"]

COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["jeebs", "--port", "8080"]
