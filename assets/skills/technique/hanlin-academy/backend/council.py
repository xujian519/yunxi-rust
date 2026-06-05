"""3-stage deliberation logic for Hanlin Academy (翰林院)."""

import re
import logging
from typing import List, Dict, Any, Tuple
from collections import defaultdict
from .client import query_models_parallel, query_model
from .config import SCHOLAR_MODELS, CHAIRMAN_MODEL

logger = logging.getLogger(__name__)


async def stage1_collect_responses(
    user_query: str,
    models: List[str] = None
) -> List[Dict[str, Any]]:
    """
    Stage 1: Collect individual responses from all scholar models (各抒己见).
    """
    if models is None:
        models = SCHOLAR_MODELS

    messages = [{"role": "user", "content": user_query}]
    responses = await query_models_parallel(models, messages)

    stage1_results = []
    for model, response in responses.items():
        if response is not None:
            stage1_results.append({
                "model": model,
                "response": response.get('content', '')
            })
        else:
            logger.warning("Scholar %s failed in Stage 1, skipping", model)

    return stage1_results


async def stage2_collect_rankings(
    user_query: str,
    stage1_results: List[Dict[str, Any]]
) -> Tuple[List[Dict[str, Any]], Dict[str, str]]:
    """
    Stage 2: Each scholar ranks the anonymized responses from OTHER scholars (匿名互评).
    """
    labels = [chr(65 + i) for i in range(len(stage1_results))]

    label_to_model = {
        f"Response {label}": result['model']
        for label, result in zip(labels, stage1_results)
    }

    stage2_results = []

    for i, result in enumerate(stage1_results):
        scholar_model = result['model']

        # Exclude this scholar's own response
        other_responses = [
            (label, r['response'])
            for label, r in zip(labels, stage1_results)
            if r['model'] != scholar_model
        ]

        if not other_responses:
            logger.warning("Scholar %s has no other responses to evaluate", scholar_model)
            continue

        responses_text = "\n\n".join([
            f"Response {label}:\n{response}"
            for label, response in other_responses
        ])

        ranking_prompt = f"""你是一位翰林院学士，正在审议以下问题：

问题：{user_query}

以下是其他学士的回答（已匿名，不包含你自己的回答）：

{responses_text}

你的任务：
1. 逐个评价每个回答，分析其优点和不足。
2. 从**准确性（accuracy）**和**洞察力（insight）**两个维度进行评估。
3. 在回答的最后，给出你的最终排名。

重要：你的最终排名必须严格按以下格式输出：
- 以 "FINAL RANKING:" 开头（全大写，带冒号）
- 然后按从好到差的顺序排列，每行一个
- 每行格式为：序号、句点、空格、回答标签（如 "1. Response C"）
- 排名部分之后不要再添加任何其他文字

示例格式：

Response A 在准确性方面表现出色，但洞察力略显不足...
Response C 提供了独到的见解...

FINAL RANKING:
1. Response C
2. Response A

现在给出你的评价和排名："""

        messages = [{"role": "user", "content": ranking_prompt}]
        response = await query_model(scholar_model, messages)

        if response is not None:
            full_text = response.get('content', '')
            parsed = parse_ranking_from_text(full_text)
            stage2_results.append({
                "model": scholar_model,
                "ranking": full_text,
                "parsed_ranking": parsed
            })
        else:
            logger.warning("Scholar %s failed in Stage 2 ranking", scholar_model)

    return stage2_results, label_to_model


async def stage3_synthesize_final(
    user_query: str,
    stage1_results: List[Dict[str, Any]],
    stage2_results: List[Dict[str, Any]],
    chairman_model: str = None
) -> Dict[str, Any]:
    """
    Stage 3: Chairman synthesizes the final response (综合定论).
    """
    if chairman_model is None:
        chairman_model = CHAIRMAN_MODEL

    stage1_text = "\n\n".join([
        f"学士模型：{result['model']}\n回答：{result['response']}"
        for result in stage1_results
    ])

    stage2_text = "\n\n".join([
        f"学士模型：{result['model']}\n评价与排名：{result['ranking']}"
        for result in stage2_results
    ])

    aggregate_text = ""
    if stage2_results:
        labels = [chr(65 + i) for i in range(len(stage1_results))]
        label_to_model = {
            f"Response {label}": result['model']
            for label, result in zip(labels, stage1_results)
        }
        agg = calculate_aggregate_rankings(stage2_results, label_to_model)
        aggregate_text = "\n\n综合排名（按平均排名从优到差）：\n" + "\n".join([
            f"第{i+1}名：{r['model']}（平均排名 {r['average_rank']:.1f}，{r['rankings_count']} 票）"
            for i, r in enumerate(agg)
        ])

    chairman_prompt = f"""你是翰林院大学士，负责综合多位学士的意见，做出最终裁决。

原始问题：{user_query}

═══ Stage 1：各学士的独立回答 ═══

{stage1_text}

═══ Stage 2：学士互评 ═══

{stage2_text}
{aggregate_text}

═══ 你的任务 ═══

作为大学士，你需要综合以上所有信息，给出一个单一、全面、准确的最终答案。

请考虑：
- 每位学士回答的优点和不足
- 互评中揭示的共识与分歧
- 综合排名反映的质量差异
- 不要简单复制某一位学士的回答，而是真正综合提炼

请给出翰林院的最终裁决："""

    messages = [{"role": "user", "content": chairman_prompt}]
    response = await query_model(chairman_model, messages)

    if response is None:
        logger.warning("Chairman failed, falling back to top-ranked scholar response")
        if stage2_results:
            labels = [chr(65 + i) for i in range(len(stage1_results))]
            label_to_model = {
                f"Response {label}": result['model']
                for label, result in zip(labels, stage1_results)
            }
            agg = calculate_aggregate_rankings(stage2_results, label_to_model)
            if agg:
                best_model = agg[0]['model']
                for r in stage1_results:
                    if r['model'] == best_model:
                        return {
                            "model": f"{chairman_model} (fallback: {best_model})",
                            "response": r['response']
                        }

        return {
            "model": chairman_model,
            "response": "错误：大学士未能生成综合答案，且无可用的学士回答作为备选。"
        }

    return {
        "model": chairman_model,
        "response": response.get('content', '')
    }


def parse_ranking_from_text(ranking_text: str) -> List[str]:
    """Parse the FINAL RANKING section from a scholar's evaluation."""
    if "FINAL RANKING:" in ranking_text:
        parts = ranking_text.split("FINAL RANKING:")
        if len(parts) >= 2:
            ranking_section = parts[1]
            numbered_matches = re.findall(r'\d+\.\s*Response [A-Z]', ranking_section)
            if numbered_matches:
                return [re.search(r'Response [A-Z]', m).group() for m in numbered_matches]

            matches = re.findall(r'Response [A-Z]', ranking_section)
            return matches

    matches = re.findall(r'Response [A-Z]', ranking_text)
    return matches


def calculate_aggregate_rankings(
    stage2_results: List[Dict[str, Any]],
    label_to_model: Dict[str, str]
) -> List[Dict[str, Any]]:
    """Calculate aggregate rankings across all scholars."""
    model_positions = defaultdict(list)

    for ranking in stage2_results:
        parsed_ranking = parse_ranking_from_text(ranking['ranking'])
        for position, label in enumerate(parsed_ranking, start=1):
            if label in label_to_model:
                model_name = label_to_model[label]
                model_positions[model_name].append(position)

    aggregate = []
    for model, positions in model_positions.items():
        if positions:
            avg_rank = sum(positions) / len(positions)
            aggregate.append({
                "model": model,
                "average_rank": round(avg_rank, 2),
                "rankings_count": len(positions)
            })

    aggregate.sort(key=lambda x: x['average_rank'])
    return aggregate


async def run_full_council(
    user_query: str,
    scholar_models: List[str] = None,
    chairman_model: str = None
) -> Tuple[List, List, Dict, Dict]:
    """Run the complete 3-stage Hanlin Academy deliberation."""
    models = scholar_models or SCHOLAR_MODELS

    stage1_results = await stage1_collect_responses(user_query, models)

    if not stage1_results:
        return [], [], {
            "model": "error",
            "response": "所有学士均未能回答。请检查 SILICONFLOW_API_KEY 和网络连接。"
        }, {}

    if len(stage1_results) == 1:
        return stage1_results, [], {
            "model": stage1_results[0]["model"],
            "response": stage1_results[0]["response"] + "\n\n[翰林院提示：仅 1 位学士成功回答，审议不充分。]"
        }, {}

    stage2_results, label_to_model = await stage2_collect_rankings(user_query, stage1_results)

    aggregate_rankings = calculate_aggregate_rankings(stage2_results, label_to_model)

    stage3_result = await stage3_synthesize_final(
        user_query,
        stage1_results,
        stage2_results,
        chairman_model
    )

    metadata = {
        "label_to_model": label_to_model,
        "aggregate_rankings": aggregate_rankings
    }

    return stage1_results, stage2_results, stage3_result, metadata
