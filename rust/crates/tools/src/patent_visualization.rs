// 专利可视化工具集
// 提供流程图生成、附图理解、技术图纸识别等功能

use serde::Deserialize;
use serde_json::json;

/// 流程图步骤
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartStep {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub step_type: Option<String>,
}

/// 流程图流转
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartFlow {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// 流程图生成输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessChartInput {
    pub steps: Vec<ChartStep>,
    pub flows: Vec<ChartFlow>,
    pub title: Option<String>,
}

/// 专利附图理解输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrawingUnderstandingInput {
    pub figure_number: String,
    pub image_description: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub technical_field: Option<String>, // 保留原因: 预留给领域特定的附图分析
    pub drawing_type: Option<String>,
}

/// 技术图纸识别输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechnicalDrawingInput {
    pub image_description: String,
    pub drawing_type: Option<String>,
    #[serde(default = "default_auto_detect")]
    pub auto_detect: Option<bool>,
}

#[allow(clippy::unnecessary_wraps)]
fn default_auto_detect() -> Option<bool> {
    Some(true)
}

/// 流程图生成器 - Mermaid 流程图生成工具
#[allow(clippy::unnecessary_wraps)]
pub fn process_chart(input: ProcessChartInput) -> Result<serde_json::Value, String> {
    let mut chart_lines = vec!["graph TD".to_string()];

    // 生成节点定义
    for step in &input.steps {
        let shape = match step.step_type.as_deref() {
            Some("start" | "end") => format!("([{}])", step.label),
            Some("decision") => format!("{{{}}}", step.label),
            _ => format!("[{}]", step.label),
        };
        chart_lines.push(format!("  {}{}", step.id, shape));
    }

    // 生成流转关系
    for flow in &input.flows {
        if let Some(label) = &flow.label {
            chart_lines.push(format!("  {} -->|{}| {}", flow.from, label, flow.to));
        } else {
            chart_lines.push(format!("  {} --> {}", flow.from, flow.to));
        }
    }

    let chart_code = chart_lines.join("\n");
    let mermaid_embed = format!("```mermaid\n{chart_code}\n```");

    Ok(json!({
        "chart_type": "mermaid",
        "title": input.title.unwrap_or_default(),
        "chart": mermaid_embed,
        "step_count": input.steps.len(),
        "flow_count": input.flows.len()
    }))
}

/// 专利附图理解 - 附图结构化分析工具
pub fn drawing_understanding(input: &DrawingUnderstandingInput) -> serde_json::Value {
    let desc = &input.image_description;
    let drawing_type = input.drawing_type.as_deref().unwrap_or("general");

    // 基于规则的组件提取（关键词模式匹配）
    let mut components = vec![];
    let keyword_patterns = [
        ("模块", "module"),
        ("单元", "unit"),
        ("装置", "device"),
        ("系统", "system"),
        ("电路", "circuit"),
        ("传感器", "sensor"),
        ("处理器", "processor"),
    ];

    let mut component_id = 100;
    for (pattern, comp_type) in keyword_patterns {
        if desc.contains(pattern) {
            // 简单提取：找到包含关键词的句子作为描述
            let description = desc
                .lines()
                .find(|line| line.contains(pattern))
                .unwrap_or(pattern)
                .to_string();
            components.push(json!({
                "id": component_id.to_string(),
                "name": format!("{}{}", pattern, component_id - 99),
                "type": comp_type,
                "description": description.trim()
            }));
            component_id += 100;
        }
    }

    // 连接关系提取
    let mut connections = vec![];
    let connection_patterns = [
        ("连接到", "data_flow"),
        ("耦合至", "coupling"),
        ("包括", "containment"),
        ("连接", "connection"),
        ("comprising", "containment"),
        ("connected to", "connection"),
    ];

    for (pattern, conn_type) in connection_patterns {
        if desc.contains(pattern) {
            connections.push(json!({
                "from": "100",
                "to": "200",
                "type": conn_type,
                "description": format!("检测到连接关系: {}", pattern)
            }));
            break; // 只取第一个连接示例
        }
    }

    // 技术特征提取
    let technical_features: Vec<String> = desc
        .lines()
        .filter(|line| line.contains("包括") || line.contains("特征") || line.contains("配置"))
        .map(|s| s.trim().to_string())
        .take(5)
        .collect();

    // 置信度计算（基于描述长度和关键词密度）
    let keyword_count = keyword_patterns
        .iter()
        .filter(|(p, _)| desc.contains(p))
        .count();
    #[allow(clippy::cast_precision_loss)]
    let confidence = if desc.len() > 50 {
        0.5 + (keyword_count as f64 * 0.1).min(0.4)
    } else {
        0.3 + (keyword_count as f64 * 0.05).min(0.3)
    };

    json!({
        "figure_number": &input.figure_number,
        "figure_type": drawing_type,
        "components": components,
        "connections": connections,
        "technical_features": technical_features,
        "summary": format!("基于描述的{}附图分析，识别出{}个组件和{}个连接关系",
            drawing_type, components.len(), connections.len()),
        "confidence": confidence
    })
}

/// 技术图纸识别 - 专业图纸自动识别工具
pub fn technical_drawing(input: &TechnicalDrawingInput) -> serde_json::Value {
    let desc = &input.image_description;
    let auto_detect = input.auto_detect.unwrap_or(true);

    // 自动检测图纸类型
    let detected_type = if auto_detect {
        let keywords = [
            (
                "chemical",
                &["苯环", "分子", "molecule", "benzene", "chemical", "化学式"][..],
            ),
            (
                "math",
                &["积分", "方程", "integral", "equation", "formula", "公式"][..],
            ),
            (
                "electrical",
                &["电路", "电阻", "电容", "circuit", "resistor", "capacitor"][..],
            ),
        ];

        let mut max_score = 0;
        let mut detected = "general";

        for (dtype, patterns) in keywords {
            let score = patterns.iter().filter(|p| desc.contains(**p)).count();
            if score > max_score {
                max_score = score;
                detected = dtype;
            }
        }
        detected
    } else {
        input.drawing_type.as_deref().unwrap_or("general")
    };

    // 组件提取（电气图纸专用）
    let components = if detected_type == "electrical" {
        let electrical_symbols = [
            ("电阻", "resistor", "R"),
            ("电容", "capacitor", "C"),
            ("电感", "inductor", "L"),
            ("二极管", "diode", "D"),
            ("晶体管", "transistor", "Q"),
            ("集成电路", "ic", "IC"),
        ];

        electrical_symbols
            .iter()
            .filter(|(name, _, _)| desc.contains(name))
            .enumerate()
            .map(|(i, (name, symbol, prefix))| {
                json!({
                    "symbol": format!("{}{}", prefix, i + 1),
                    "type": symbol,
                    "description": name
                })
            })
            .collect()
    } else {
        vec![]
    };

    // 分析文本生成
    let analysis = match detected_type {
        "chemical" => {
            let molecule_count = desc.matches("分子").count() + desc.matches("molecule").count();
            format!("检测到化学图纸，包含约{molecule_count}个分子结构描述")
        }
        "math" => {
            let formula_count = desc.matches("方程").count() + desc.matches("equation").count();
            format!("检测到数学图纸，包含约{formula_count}个方程或公式")
        }
        "electrical" => {
            format!("检测到电气图纸，识别出{}个电子元器件", components.len())
        }
        _ => "通用技术图纸，未检测到特定领域特征".to_string(),
    };

    // 置信度评估
    let confidence = if components.len() > 3 {
        0.8
    } else if !components.is_empty() {
        0.6
    } else if desc.len() > 100 {
        0.5
    } else {
        0.3
    };

    json!({
        "detected_type": detected_type,
        "components": components,
        "analysis": analysis,
        "confidence": confidence,
        "needs_ocr": desc.len() < 50 || desc.contains("无法识别")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_chart_basic() {
        let input = ProcessChartInput {
            steps: vec![
                ChartStep {
                    id: "A".to_string(),
                    label: "开始".to_string(),
                    step_type: Some("start".to_string()),
                },
                ChartStep {
                    id: "B".to_string(),
                    label: "处理".to_string(),
                    step_type: Some("process".to_string()),
                },
            ],
            flows: vec![ChartFlow {
                from: "A".to_string(),
                to: "B".to_string(),
                label: None,
            }],
            title: Some("测试流程".to_string()),
        };

        let result = process_chart(input).unwrap();
        assert_eq!(result["chart_type"], "mermaid");
        assert_eq!(result["step_count"], 2);
        assert!(result["chart"].as_str().unwrap().contains("```mermaid"));
    }

    #[test]
    fn test_drawing_understanding() {
        let input = DrawingUnderstandingInput {
            figure_number: "图1".to_string(),
            image_description: "系统包括处理模块、传感器单元，处理模块连接到传感器单元".to_string(),
            technical_field: Some("电子工程".to_string()),
            drawing_type: Some("block_diagram".to_string()),
        };

        let result = drawing_understanding(&input);
        assert_eq!(result["figure_number"], "图1");
        assert!(!result["components"].as_array().unwrap().is_empty());
        assert!(result["confidence"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_technical_drawing_auto_detect() {
        let input = TechnicalDrawingInput {
            image_description: "电路包括电阻R1、电容C1，连接到电源".to_string(),
            drawing_type: None,
            auto_detect: Some(true),
        };

        let result = technical_drawing(&input);
        assert_eq!(result["detected_type"], "electrical");
        assert!(!result["components"].as_array().unwrap().is_empty());
    }
}
