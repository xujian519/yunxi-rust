"""FastAPI server for Hanlin Academy (翰林院)."""

import logging
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Dict, Any, Optional

from .config import SCHOLAR_MODELS, CHAIRMAN_MODEL, SERVER_HOST, SERVER_PORT
from .council import run_full_council

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="翰林院 Hanlin Academy API")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class DeliberateRequest(BaseModel):
    """Request to start a deliberation."""
    query: str
    config: Optional[Dict[str, Any]] = None


class DeliberateResponse(BaseModel):
    """Response from a deliberation."""
    stage1: List[Dict[str, Any]]
    stage2: List[Dict[str, Any]]
    stage3: Dict[str, Any]
    metadata: Dict[str, Any]


@app.get("/")
async def root():
    """Health check."""
    return {"status": "ok", "service": "翰林院 Hanlin Academy API"}


@app.get("/api/hanlin/health")
async def health():
    """Show current configuration and connectivity status."""
    return {
        "status": "ok",
        "scholars": SCHOLAR_MODELS,
        "chairman": CHAIRMAN_MODEL,
        "scholar_count": len(SCHOLAR_MODELS),
    }


@app.post("/api/hanlin/deliberate")
async def deliberate(request: DeliberateRequest):
    """
    Run a full 3-stage deliberation (三阶段审议).

    Accepts a query and optional config overrides for scholars and chairman.
    Returns all stages plus metadata.
    """
    config = request.config or {}

    scholar_models = config.get("scholars", None)
    chairman_model = config.get("chairman", None)

    logger.info(
        "Starting deliberation: query=%r, scholars=%s, chairman=%s",
        request.query[:80] + "..." if len(request.query) > 80 else request.query,
        scholar_models or "default",
        chairman_model or "default"
    )

    stage1_results, stage2_results, stage3_result, metadata = await run_full_council(
        user_query=request.query,
        scholar_models=scholar_models,
        chairman_model=chairman_model
    )

    return {
        "stage1": stage1_results,
        "stage2": stage2_results,
        "stage3": stage3_result,
        "metadata": metadata
    }


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host=SERVER_HOST, port=SERVER_PORT)
