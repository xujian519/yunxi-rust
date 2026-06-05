"""SiliconFlow API client for Hanlin Academy."""

import httpx
import logging
from typing import List, Dict, Any, Optional
from .config import SILICONFLOW_API_KEY, SILICONFLOW_API_URL, DEFAULT_TIMEOUT

logger = logging.getLogger(__name__)


async def query_model(
    model: str,
    messages: List[Dict[str, str]],
    timeout: float = DEFAULT_TIMEOUT
) -> Optional[Dict[str, Any]]:
    """
    Query a single model via SiliconFlow API.

    Args:
        model: SiliconFlow model identifier (e.g., "deepseek-ai/DeepSeek-V4-Pro")
        messages: List of message dicts with 'role' and 'content'
        timeout: Request timeout in seconds

    Returns:
        Response dict with 'content' and optional 'reasoning_details', or None if failed
    """
    if not SILICONFLOW_API_KEY:
        logger.error("SILICONFLOW_API_KEY is not configured")
        return None

    headers = {
        "Authorization": f"Bearer {SILICONFLOW_API_KEY}",
        "Content-Type": "application/json",
    }

    payload = {
        "model": model,
        "messages": messages,
    }

    try:
        async with httpx.AsyncClient(timeout=timeout) as client:
            response = await client.post(
                SILICONFLOW_API_URL,
                headers=headers,
                json=payload
            )
            response.raise_for_status()

            data = response.json()
            message = data['choices'][0]['message']

            return {
                'content': message.get('content'),
                'reasoning_details': message.get('reasoning_details')
            }

    except httpx.HTTPStatusError as e:
        logger.error(
            "HTTP error querying model %s: %d %s",
            model, e.response.status_code, e.response.text[:200]
        )
        return None
    except httpx.TimeoutException:
        logger.error("Timeout querying model %s after %.0fs", model, timeout)
        return None
    except Exception as e:
        logger.error("Error querying model %s: %s", model, e)
        return None


async def query_models_parallel(
    models: List[str],
    messages: List[Dict[str, str]],
    timeout: float = DEFAULT_TIMEOUT
) -> Dict[str, Optional[Dict[str, Any]]]:
    """
    Query multiple models in parallel.

    Args:
        models: List of SiliconFlow model identifiers
        messages: List of message dicts to send to each model
        timeout: Request timeout in seconds

    Returns:
        Dict mapping model identifier to response dict (or None if failed)
    """
    import asyncio

    tasks = [query_model(model, messages, timeout) for model in models]
    responses = await asyncio.gather(*tasks)
    return {model: response for model, response in zip(models, responses)}
