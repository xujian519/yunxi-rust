# 认知引擎修复与优化计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复 YunXi 认知决策管线的三个 P0 阻塞问题（推理空壳、工作流模拟、KG 数据缺失），并进行 P1 架构增强（统一工具接口、动态路由、记忆系统统一）

**Architecture:** 采用 trait-based 依赖注入模式，在 reasoning/workflow 叶子 crate 中定义抽象接口 trait，由 tools/runtime 上层 crate 实现，避免循环依赖。LLM 注入通过回调/trait 模式而非直接依赖。

**Tech Stack:** Rust 2021, Tokio, rusqlite, serde, BGE-M3 ONNX

---

## P0: 关键阻塞修复

### Task 1: ReasoningPipeline LLM 注入

**Why:** `ReasoningPipeline.plan()` 只生成 JSON 指令模板，从未调用 LLM 执行推理，是整个认知管线的核心缺陷

**Files:**
- Modify: `rust/crates/reasoning/Cargo.toml`
- Modify: `rust/crates/reasoning/src/pipeline.rs`
- Create: `rust/crates/reasoning/src/executor.rs`

**Architecture:** 定义 `ReasoningExecutor` trait（同步回调），由调用方传入。在 `plan()` 之后新增 `execute()` 方法接收 executor 异步执行各阶段。

### Task 2: Workflow AgentCall 连接真实 Agent

**Why:** `FlowStep::AgentCall` 的 `run_step()` 返回硬编码 success JSON，未实际创建子 Agent，工作流是断链的

**Files:**
- Modify: `rust/crates/workflow/src/executor.rs`
- Modify: `rust/crates/workflow/src/flow.rs`
- Create: `rust/crates/workflow/src/agent_bridge.rs`

### Task 3: KG 种子数据与优雅降级

**Why:** `legal_reasoning.rs` 的测试依赖外部 `patent_kg.db`，无此文件时所有测试跳过

**Files:**
- Create: `rust/crates/patent-domain/src/kg_seed.rs`
- Modify: `rust/crates/patent-domain/src/legal_reasoning.rs`

---

## P1: 架构增强

### Task 4: 统一工具接口 trait

**Files:**
- Create: `rust/crates/tools/src/tool_trait.rs`

### Task 5: 动态路由资源推荐

**Files:**
- Modify: `rust/crates/router/src/workflow_router.rs`

### Task 6: 记忆系统统一

**Files:**
- Modify: `rust/crates/memory/src/lib.rs`
- Create: `rust/crates/memory/src/unified.rs`

---
