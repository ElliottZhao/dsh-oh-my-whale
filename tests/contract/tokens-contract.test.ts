/**
 * 接缝一：#29 Token 计量。
 *
 * 口径只在 Core 实现一次；这里断言真实 Core 的估算结果、与 DSH `estimateMessage()`
 * 的密度对齐、混排不整体偏向，以及 Adapter 源码里没有第二个换算。每场景 Pass^k。
 */
import assert from "node:assert/strict";
import { readFile, rm } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, it } from "node:test";

import { CoreConnection } from "../../packages/adapter/src/index.ts";
import { PASS_K, passK, startCore, tempConfigPath, tempDataDir } from "./helpers.ts";

/** DSH `ctx.tokenMeter` 的固定密度：4 字符/token。 */
const CHARS_PER_TOKEN = 4;
/** DSH 每条消息的固定框架开销：4 块结构 + 4 角色框架。 */
const MESSAGE_OVERHEAD = 8;

function dshEnglishDensity(text: string): number {
  return Math.ceil(text.length / CHARS_PER_TOKEN);
}

const ENGLISH_FIXTURES = [
  "",
  "a",
  "abcd",
  "abcde",
  "hello world",
  "The quick brown fox jumps over the lazy dog.",
  "I prefer concise answers, please.",
  "pnpm install --frozen-lockfile",
  "GET /v1/observations?sessionId=s1&limit=10",
  "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
];

/** 先停 Core 再删目录：Windows 上运行中的 SQLite 文件不能被删除。 */
async function stopAndRemove(core: { stop: () => Promise<void> }, dir: string): Promise<void> {
  await core.stop().catch(() => undefined);
  await rm(dir, { recursive: true, force: true }).catch(() => undefined);
}

describe("Token 计量契约（#29）", () => {
  it(`纯英文文本的 Core 计数与 DSH estimateMessage 密度对齐（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const connection = new CoreConnection({ endpoint: core.ready.endpoint });
        for (const text of ENGLISH_FIXTURES) {
          const estimated = await connection.estimateTokens(text);
          assert.equal(
            estimated.tokens,
            dshEnglishDensity(text),
            `英文密度应与 ceil(chars/4) 一致：${JSON.stringify(text)}`,
          );
          assert.equal(
            estimated.messageTokens,
            dshEnglishDensity(text) + MESSAGE_OVERHEAD,
            `整条消息口径 = 文本密度 + 8：${JSON.stringify(text)}`,
          );
          assert.equal(estimated.wHan, 1.5, "默认按硬上限倍率 1.5");
        }
      } finally {
        await stopAndRemove(core, dir);
      }
    });
  });

  it(`16 个汉字的计数落在 o200k 与 cl100k 实测区间内（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const connection = new CoreConnection({ endpoint: core.ready.endpoint });
        const text = "我今天下午想去西湖边散步然后回家";
        assert.equal([...text].length, 16, "夹具应是 16 个汉字");
        const estimated = await connection.estimateTokens(text);
        assert.ok(
          estimated.tokens >= 14 && estimated.tokens <= 26,
          `16 汉字应落在 14–26 之间（o200k≈14、cl100k≈23），实际 ${estimated.tokens}`,
        );
      } finally {
        await stopAndRemove(core, dir);
      }
    });
  });

  it(`混排文本按 Han / 非 Han 分段加权，不整体偏向（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const connection = new CoreConnection({ endpoint: core.ready.endpoint });
        const text = "用 pnpm 装依赖";
        const han = [...text].filter((ch) => /\p{Script=Han}/u.test(ch)).length;
        const other = [...text].length - han;
        const expected = Math.ceil(han * 1.5 + other * 0.25);
        const estimated = await connection.estimateTokens(text);
        assert.equal(estimated.tokens, expected, "混排应按分段加权计算");
      } finally {
        await stopAndRemove(core, dir);
      }
    });
  });

  it(`Adapter 源码里没有第二个 token 换算（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const here = dirname(fileURLToPath(import.meta.url));
      const adapterSrc = resolve(here, "../../packages/adapter/src");
      const files = ["connection.ts", "index.ts", "validate.ts"];
      for (const file of files) {
        const source = await readFile(join(adapterSrc, file), "utf8");
        assert.doesNotMatch(
          source,
          /\.length\s*\/\s*4\b/,
          `${file} 不应含 length / 4 形式的自算；口径只在 Core`,
        );
      }
    });
  });
});
