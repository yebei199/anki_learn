# 智谱AI模型文档

## 概述

本文档描述了用于与智谱AI API交互的数据结构。所有结构体都支持序列化和反序列化，用于API请求和响应。

## 结构体定义

### 1. ZhiPuMessage - 智谱AI消息结构体

表示对话中的单条消息，包含发送者角色和消息内容。

#### 字段
- `role` (String): 消息发送者的角色，通常为 "user"（用户）或 "assistant"（助手）
- `content` (String): 消息的具体文本内容

#### 示例
```rust
ZhiPuMessage {
    role: "user".to_string(),
    content: "你好，请介绍一下你自己。".to_string(),
}
```

### 2. ZhiPuRequest - 智谱AI请求结构体

表示发送给智谱AI API的完整请求体，包含模型选择、消息列表和可选参数。

#### 字段
- `model` (String): 使用的模型名称，如 "glm-4.7"
- `messages` (Vec<ZhiPuMessage>): 消息列表，包含对话历史和当前请求
- `stream` (Option<bool>): 是否使用流式响应，None 表示不使用
- `temperature` (Option<f32>): 控制输出的随机性，0.0-2.0 之间，越高越随机

#### 示例
```rust
ZhiPuRequest {
    model: "glm-4.7".to_string(),
    messages: vec![
        ZhiPuMessage {
            role: "user".to_string(),
            content: "你好".to_string(),
        }
    ],
    stream: Some(false),
    temperature: Some(0.7),
}
```

### 3. ZhiPuResponse - 智谱AI响应结构体

表示智谱AI API返回的完整响应，包含请求ID、时间戳、模型信息和响应内容。

#### 字段
- `id` (String): 响应的唯一标识符
- `request_id` (String): 请求的唯一标识符
- `created` (i64): 响应生成的时间戳（Unix时间戳）
- `model` (String): 实际使用的模型名称
- `choices` (Vec<ZhiPuChoice>): 响应选项列表，通常包含一个选项
- `usage` (ZhiPuUsage): Token使用统计信息

### 4. ZhiPuUsage - Token使用情况结构体

统计本次请求中Token的使用情况，用于计费和使用量监控。

#### 字段
- `prompt_tokens` (i32): 输入（提示）部分使用的Token数量
- `completion_tokens` (i32): 输出（完成）部分使用的Token数量
- `total_tokens` (i32): 本次请求使用的总Token数量

### 5. ZhiPuChoice - 响应选项结构体

表示API响应中的单个选项，包含响应内容和完成原因。

#### 字段
- `index` (i32): 选项的索引号，从0开始
- `message` (ZhiPuResponseMessage): 响应消息内容
- `finish_reason` (String): 响应完成的原因，如 "stop"（正常结束）或 "length"（达到最大长度）

### 6. ZhiPuResponseMessage - 响应消息内容结构体

表示API返回的响应消息，包含角色、内容和推理过程。

#### 字段
- `role` (String): 消息发送者的角色，通常为 "assistant"
- `content` (String): 响应的文本内容
- `reasoning_content` (Option<String>): 可选的推理过程内容，用于展示模型的思考过程

## 使用示例

### 完整的API调用流程

```rust
// 创建请求
let request = ZhiPuRequest {
    model: "glm-4.7".to_string(),
    messages: vec![
        ZhiPuMessage {
            role: "user".to_string(),
            content: "请解释什么是人工智能？".to_string(),
        }
    ],
    stream: None,
    temperature: Some(0.8),
};

// 调用API
let response = zhi_pu_completion(api_key, request).await?;

// 处理响应
for choice in response.choices {
    println!("回复: {}", choice.message.content);
    println!("Token使用情况: {}", response.usage.total_tokens);
}
```

## 注意事项

1. 所有结构体都使用了 `#[derive(Debug, Serialize, Deserialize)]`，支持调试输出和JSON序列化
2. `ZhiPuRequest` 中的可选字段使用了 `#[serde(skip_serializing_if = "Option::is_none")]`，在序列化时会跳过None值
3. Token使用情况可用于计费和成本控制
4. 响应ID和请求ID可用于追踪和调试