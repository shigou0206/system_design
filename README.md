# TRN (Tool Resource Name) Python Library

一个用于处理工具资源名称(TRN)的企业级Python库，专为AI Agent平台设计。

## 概述

TRN库提供了完整的TRN标识符管理功能，包括解析、验证、URL转换、模式匹配等。TRN格式基于最新的设计规范，支持多租户、版本管理、内容验证等高级特性。

### TRN格式

```
trn:platform[:scope]:resource_type:type[:subtype]:instance_id:version[:tag][@hash]
```

**示例:**
```
trn:user:alice:tool:openapi:github-api:v1.0
trn:org:company:tool:workflow:data-pipeline:v2.1:production@sha256:abc123
trn:aiplatform:tool:python:async:data-processor:latest
```

## 特性

- ✅ **完整的TRN解析和验证** - 支持所有TRN组件和格式
- ✅ **URL转换** - TRN与URL格式互相转换
- ✅ **模式匹配** - 支持通配符的高级模式匹配
- ✅ **版本管理** - 语义化版本比较和别名解析
- ✅ **错误处理** - 详细的错误信息和建议修复
- ✅ **工具函数** - 丰富的辅助函数和Builder模式
- ✅ **类型安全** - 完整的类型注解支持
- ✅ **高性能** - 缓存和优化设计

## 安装

```bash
# 从源码安装
git clone <repository-url>
cd trn-library
pip install -e .

# 或者直接安装依赖
pip install dataclasses typing
```

## 快速开始

### 基本使用

```python
import trn
from trn import TRN

# 解析TRN字符串
trn_obj = TRN.parse("trn:user:alice:tool:openapi:github-api:v1.0")
print(f"Platform: {trn_obj.platform}")  # user
print(f"Scope: {trn_obj.scope}")        # alice
print(f"Type: {trn_obj.type}")          # openapi

# 创建TRN对象
new_trn = TRN(
    platform="aiplatform",
    resource_type="tool",
    type="workflow", 
    instance_id="data-pipeline",
    version="v2.0"
)
print(str(new_trn))  # trn:aiplatform:tool:workflow:data-pipeline:v2.0

# 验证TRN
if trn.validate("trn:user:alice:tool:openapi:github-api:v1.0"):
    print("Valid TRN!")
```

### URL转换

```python
from trn import TRN

trn_obj = TRN.parse("trn:user:alice:tool:openapi:github-api:v1.0")

# 转换为URL格式
trn_url = trn_obj.to_url()
print(trn_url)  # trn://user/alice/tool/openapi/github-api/v1.0

# 转换为HTTP URL
http_url = trn.TRNURLConverter.to_url(trn_obj, scheme="https")
print(http_url)  # https://your-platform.com/tools/user/alice/openapi/github-api/v1.0

# 从URL解析回TRN
parsed = trn.from_url(trn_url)
print(str(parsed))  # trn:user:alice:tool:openapi:github-api:v1.0
```

### 模式匹配

```python
from trn.utils import find_matching_trns, match_trn_pattern

trn_list = [
    "trn:user:alice:tool:openapi:github-api:v1.0",
    "trn:user:alice:tool:openapi:slack-api:v2.0", 
    "trn:user:bob:tool:python:data-processor:v1.5",
    "trn:org:company:tool:workflow:etl-pipeline:v3.0"
]

# 查找所有Alice的工具
alice_tools = find_matching_trns(trn_list, "trn:user:alice:*")
print(alice_tools)

# 查找所有OpenAPI工具
openapi_tools = find_matching_trns(trn_list, "trn:*:*:tool:openapi:*")
print(openapi_tools)

# 单个匹配
if match_trn_pattern("trn:user:alice:tool:openapi:github-api:v1.0", "trn:user:*"):
    print("Matches!")
```

### Builder模式

```python
from trn import TRNBuilder

# 使用Builder构建TRN
trn_obj = (TRNBuilder()
           .platform("user")
           .scope("alice")
           .resource_type("tool")
           .type("openapi")
           .instance_id("github-api")
           .version("v1.0")
           .tag("stable")
           .build())

print(str(trn_obj))  # trn:user:alice:tool:openapi:github-api:v1.0:stable
```

### 版本管理

```python
from trn.utils import compare_trn_versions, get_latest_version_trn

# 版本比较
assert compare_trn_versions("v1.1", "v1.0", ">") == True
assert compare_trn_versions("v2.0", "v2.*", "~") == True

# 获取最新版本
versions = [
    "trn:user:alice:tool:openapi:github-api:v1.0",
    "trn:user:alice:tool:openapi:github-api:v1.1", 
    "trn:user:alice:tool:openapi:github-api:v2.0"
]
latest = get_latest_version_trn(versions)
print(latest)  # trn:user:alice:tool:openapi:github-api:v2.0
```

## 错误处理

```python
from trn import TRN
from trn.exceptions import TRNError, TRNValidationError

try:
    trn_obj = TRN.parse("invalid-trn-format")
except TRNValidationError as e:
    print(f"Validation error: {e}")
    # 获取修复建议
    suggestions = e.suggest_fixes("invalid-trn-format")
    print(f"Suggestions: {suggestions}")
except TRNError as e:
    print(f"TRN error: {e}")
```

## 高级功能

### 批量验证

```python
from trn.utils import batch_validate_trns

trn_list = [
    "trn:user:alice:tool:openapi:github-api:v1.0",
    "invalid-trn",
    "trn:org:company:tool:workflow:etl:v2.0"
]

results = batch_validate_trns(trn_list)
print(f"Valid: {results['valid_count']}/{results['total_count']}")
print(f"Success rate: {results['success_rate']:.2%}")
```

### 高级匹配器

```python
from trn import TRNMatcher

# 创建复杂模式匹配器
matcher = TRNMatcher("trn:user:*:tool:openapi:*:v1.*")

test_trns = [
    "trn:user:alice:tool:openapi:github-api:v1.0",
    "trn:user:bob:tool:python:data-processor:v2.0"
]

for trn_str in test_trns:
    if matcher.matches(trn_str):
        print(f"Matches: {trn_str}")
```

## API参考

### 核心类

- **`TRN`** - 主要的TRN对象类
- **`TRNComponents`** - TRN组件数据结构
- **`TRNValidator`** - TRN验证器
- **`TRNParser`** - TRN解析器
- **`TRNURLConverter`** - URL转换器

### 工具类

- **`TRNBuilder`** - Builder模式构建器
- **`TRNMatcher`** - 高级模式匹配器

### 异常类

- **`TRNError`** - 基础TRN异常
- **`TRNValidationError`** - 验证失败异常
- **`TRNFormatError`** - 格式错误异常
- **`TRNComponentError`** - 组件错误异常

## 运行示例

```bash
# 运行基本使用示例
python examples/basic_usage.py

# 运行特定示例函数
python -c "
from examples.basic_usage import example_basic_parsing
example_basic_parsing()
"
```

## 配置

TRN库支持全局配置：

```python
from trn.constants import DEFAULT_CONFIG

# 修改默认配置
DEFAULT_CONFIG["strict_validation"] = False
DEFAULT_CONFIG["cache_validation_results"] = True
DEFAULT_CONFIG["max_cache_size"] = 2000
```

## 支持的TRN格式

### 平台类型
- `aiplatform` - 系统平台
- `user` - 用户平台  
- `org` - 组织平台

### 资源类型
- `tool` - 可执行工具
- `dataset` - 数据集资源
- `pipeline` - 工作流模板
- `model` - AI模型资源

### 工具类型
- `openapi` - RESTful API工具
- `workflow` - 业务流程工具
- `python` - Python执行工具
- `shell` - Shell命令工具
- `system` - 系统操作工具

## 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开Pull Request

## 许可证

本项目采用MIT许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 更新日志

### v1.0.0
- 初始发布
- 完整的TRN解析和验证功能
- URL转换支持
- 模式匹配和工具函数
- 企业级错误处理 