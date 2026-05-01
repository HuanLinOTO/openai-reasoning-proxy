# OpenAI Reasoning Proxy

A small Rust proxy for OpenAI-compatible chat completion requests.

It accepts requests at `/{url}`, forwards the request to that raw absolute URL, and preserves method, query parameters, headers, and body. If the JSON body contains a `messages` array, every assistant message that is missing `reasoning_content` is patched to include an empty string before forwarding.

## Run

```powershell
cargo run
```

The server listens on `127.0.0.1:3000` by default. Set `PORT` to use another port.

## Example

```powershell
curl.exe -X POST "http://127.0.0.1:3000/https://api.openai.com/v1/chat/completions" `
  -H "Authorization: Bearer $env:OPENAI_API_KEY" `
  -H "Content-Type: application/json" `
  -d '{"model":"gpt-4o-mini","messages":[{"role":"assistant","content":"hello"},{"role":"user","content":"continue"}]}'
```

The forwarded body will contain:

```json
{"role":"assistant","content":"hello","reasoning_content":""}
```

## Notes

- The upstream URL must use `http` or `https`.
- Non-JSON bodies are forwarded unchanged.
- Existing `reasoning_content` values are preserved.
