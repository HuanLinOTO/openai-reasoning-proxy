# OpenAI Reasoning Proxy

一个用于 OpenAI 兼容 Chat Completion 请求的小型 Rust 代理。

它接受 `/{url}` 形式的请求，将请求转发到这个原始绝对 URL，并尽量透传 method、query、headers 和 body。如果 JSON body 中包含 `messages` 数组，代理会在转发前检查历史消息：所有缺少 `reasoning_content` 的 assistant 消息都会被补上空字符串。

## 运行

```powershell
cargo run
```

默认监听 `127.0.0.1:3000`。如果需要修改端口，设置 `PORT`：

```powershell
$env:PORT="8080"
cargo run
```

如果需要监听其他地址，设置 `HOST`：

```powershell
$env:HOST="0.0.0.0"
$env:PORT="3000"
cargo run
```

## Docker 部署

使用已发布的 GHCR 镜像后台运行：

```powershell
docker run -d --name openai-reasoning-proxy --restart unless-stopped -p 3000:3000 ghcr.io/huanlinoto/openai-reasoning-proxy:latest
```

容器内默认监听 `0.0.0.0:3000`。

使用自定义端口：

```powershell
docker run -d --name openai-reasoning-proxy --restart unless-stopped -p 8080:8080 -e PORT=8080 ghcr.io/huanlinoto/openai-reasoning-proxy:latest
```

查看日志：

```powershell
docker logs -f openai-reasoning-proxy
```

停止并删除容器：

```powershell
docker stop openai-reasoning-proxy
docker rm openai-reasoning-proxy
```

本地构建并运行：

```powershell
docker build -t openai-reasoning-proxy:local .
docker run -d --name openai-reasoning-proxy --restart unless-stopped -p 3000:3000 openai-reasoning-proxy:local
```

## 请求示例

```powershell
curl.exe -X POST "http://127.0.0.1:3000/https://api.openai.com/v1/chat/completions" `
  -H "Authorization: Bearer $env:OPENAI_API_KEY" `
  -H "Content-Type: application/json" `
  -d '{"model":"gpt-4o-mini","messages":[{"role":"assistant","content":"hello"},{"role":"user","content":"continue"}]}'
```

转发前的 body 会被补成类似这样：

```json
{"role":"assistant","content":"hello","reasoning_content":""}
```

## 说明

- 上游 URL 必须使用 `http` 或 `https`。
- 非 JSON body 会原样转发。
- 已存在的 `reasoning_content` 会保持不变。
- raw URL 写在路径中，例如 `/https://api.openai.com/v1/chat/completions`。如果前面还有其他反向代理，确认它不会改写或截断路径中的 `https://`。

## Credits
 - linux.do