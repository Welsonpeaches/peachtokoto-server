# Jiangtokoto Server

一个高性能的表情包随机抽取 API 服务器。

## 特性

- 🚀 高性能异步 Web 框架 (基于 axum)
- 💾 内存缓存支持
- 🔄 CORS 支持
- 📝 YAML 配置
- 🔍 智能 MIME 类型检测
- ✨ 零拷贝文件传输
- 📊 内置监控和日志

## 快速开始

### 1. 环境要求

- Rust 1.70.0 或更高版本
- Cargo 包管理器

### 2. 配置

1. 复制配置文件模板：
   ```bash
   cp config.yml.example config.yml
   ```

2. 编辑 `config.yml` 配置文件：
   ```yaml
   server:
     host: "0.0.0.0"  # 监听地址
     port: 3000       # 监听端口

   storage:
     memes_dir: "assets/memes"  # 表情包目录

   cache:
     max_size: 100    # 缓存大小
     ttl_secs: 300    # 缓存过期时间
   ```

### 3. 构建和运行

```bash
# 构建项目
cargo build --release

# 运行服务器
cargo run --release
```

## API 端点

### 获取随机表情包

```http
GET /memes/random
```

响应:

- 200: 成功返回随机表情包
- 404: 未找到表情包
- 500: 服务器内部错误

### 健康检查

```http
GET /memes/health
```

响应:

- 200: 服务正常

## 开发

### 目录结构

```
.
├── src/
│   ├── config/     # 配置管理
│   ├── handlers/   # 请求处理器
│   ├── models/     # 数据模型
│   ├── services/   # 业务逻辑
│   └── utils/      # 工具函数
├── assets/         # 静态资源
└── config.yml      # 配置文件
```

### 调试模式运行

```bash
RUST_LOG=debug cargo run
```

## 性能优化

服务器使用了多项性能优化技术：

- 异步 I/O
- 内存缓存
- 零拷贝文件传输
- 连接池优化

## 贡献指南

1. Fork 本项目
2. 创建新的功能分支
3. 提交更改
4. 创建 Pull Request

## 许可证

MIT License