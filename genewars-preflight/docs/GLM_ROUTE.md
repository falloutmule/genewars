# GeneWars GLM worker through 9router

The isolated profile is `~/.codex/genewars-glm53.config.toml`. Select it explicitly:

```powershell
codex exec --profile genewars-glm53 --sandbox read-only --ephemeral --skip-git-repo-check "A bounded read-only task"
```

The profile uses `http://127.0.0.1:20128/v1`, Responses, and gateway model `openrouter/z-ai/glm-5.3`. The installed gateway maps the prefix to OpenRouter and strips it before sending upstream. It uses the existing saved OpenRouter connection. The companion `~/.codex/genewars-9router-auth.py` supplies the existing gateway token internally; never run it to display the token. Default Desktop settings and Luna/Sol routing were not changed. Profile-only MCP connections are disabled for this bounded coding route.

Start the installed gateway when necessary with `9router --no-browser --skip-update`. It is currently running. No login, funds addition or billing-limit changes were needed. Preserve the existing gateway database.

The complete Codex streaming/tool cycle and a narrow Extractor metadata task passed. The CLI's usage counters returned zero incorrectly; gateway usage and OpenRouter key billing are recorded separately in `research/integration/glm53-gateway-evaluation-001.json`. Gateway cost estimates differ from actual billing. Exact per-change cost and root token savings are not yet established. A direct generation-ID lookup returned404 and is recorded as unavailable, not successful generation-record verification. Account privacy controls also remain unverified.

Keep GLM tasks small and provide compact local comparison summaries. Root remains integration authority. A provider failure ends that compatibility test; it does not end independently actionable GeneWars work. Do not substitute another model or route.
