"""Configuration for the Hanlin Academy (翰林院) deliberation system."""

import os
from dotenv import load_dotenv

load_dotenv()

# SiliconFlow API configuration
SILICONFLOW_API_KEY = os.getenv("SILICONFLOW_API_KEY")
SILICONFLOW_API_URL = "https://api.siliconflow.cn/v1/chat/completions"

# Default scholar models (学士)
SCHOLAR_MODELS = [
    "deepseek-ai/DeepSeek-V4-Flash",
    "deepseek-ai/DeepSeek-V4-Pro",
    "Pro/zai-org/GLM-5.1",
    "Qwen/Qwen3.6-27B",
]

# Chairman model (大学士) - synthesizes final response
CHAIRMAN_MODEL = "deepseek-ai/DeepSeek-V4-Pro"

# Server configuration
SERVER_HOST = "0.0.0.0"
SERVER_PORT = 8010

# Timeout defaults
DEFAULT_TIMEOUT = 120.0
