import task from "tasuku"
import { $ } from "bun"

await task.group(
  (task) => [
    task("TypeScript type check", async () => {
      await $`tsc --noEmit`
    }),
    task("oxlint", async () => {
      await $`oxlint .`
    }),
    task("oxfmt check", async () => {
      await $`oxfmt --check .`
    }),
    task("Unit tests", async () => {
      await $`vitest run`
    }),
  ],
  { concurrency: 4, stopOnError: false },
)
