services:
  app:
    # ...
    volumes:
      - jeebs_data:/data
      - ./webui:/app/webui:ro