import { tool } from "@opencode-ai/plugin"
import path from "path"

export default tool({
  description:
    "Re-parse the knowledge base (Obsidian vaults + Scrivener projects) and refresh the context files under /common/artifacts/context. Run this when the source notes have changed and the @kb reference is stale. No arguments.",
  args: {},
  async execute(_args, context) {
    const script = path.join(context.worktree, "run.sh")
    const proc = await Bun.$`bash ${script}`.nothrow()
    const stdout = proc.stdout.toString().trim()
    const stderr = proc.stderr.toString().trim()
    if (proc.exitCode !== 0) {
      return `kbparse failed (exit ${proc.exitCode})\n--- stderr ---\n${stderr}\n--- stdout ---\n${stdout}`
    }
    return `kbparse refreshed /common/artifacts/context\n--- stdout ---\n${stdout}${stderr ? `\n--- stderr ---\n${stderr}` : ""}`
  },
})