import { execSync } from "child_process"
import { writeFileSync, unlinkSync } from "fs"
import { tmpdir } from "os"
import { join } from "path"

interface Step {
  action: string
  selector?: string
  command?: string
  args?: Record<string, unknown>
  expect?: string
}

export class TauriPilotFlow {
  private name: string
  private steps: Step[] = []

  constructor(name: string = `flow-${Date.now()}`) {
    this.name = name
  }

  click(selector: string): this {
    this.steps.push({ action: "click", selector })
    return this
  }

  wait(selector: string, text?: string): this {
    const step: Step = { action: "wait", selector }
    if (text) step.args = { text }
    this.steps.push(step)
    return this
  }

  assert(selector: string, args: Record<string, unknown>): this {
    this.steps.push({ action: "assert", selector, args })
    return this
  }

  ipc(command: string, args: Record<string, unknown>): this {
    this.steps.push({ action: "ipc", command, args })
    return this
  }

  snapshot(args?: Record<string, unknown>): this {
    this.steps.push({ action: "snapshot", ...(args ? { args } : {}) })
    return this
  }

  stepCount(): number {
    return this.steps.length
  }

  run(): void {
    const toml = this.serialize()
    const file = join(tmpdir(), `gauge-flow-${Date.now()}.toml`)
    writeFileSync(file, toml, "utf-8")
    try {
      execSync(`tauri-pilot run "${file}"`, { stdio: "inherit" })
    } finally {
      unlinkSync(file)
    }
  }

  private serialize(): string {
    const lines: string[] = [`name = "${this.name}"`, ""]
    for (const step of this.steps) {
      lines.push("[[steps]]")
      lines.push(`action = "${step.action}"`)
      if (step.selector !== undefined)
        lines.push(`selector = "${step.selector}"`)
      if (step.command !== undefined) lines.push(`command = "${step.command}"`)
      if (step.expect !== undefined) lines.push(`expect = "${step.expect}"`)
      if (step.args !== undefined) {
        const argsStr = Object.entries(step.args)
          .map(([k, v]) => `${k} = ${typeof v === "string" ? `"${v}"` : v}`)
          .join(", ")
        lines.push(`args = { ${argsStr} }`)
      }
      lines.push("")
    }
    return lines.join("\n")
  }
}
