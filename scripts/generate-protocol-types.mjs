// 由 JSON Schema 生成 Adapter 侧的 TypeScript 类型、schema 常量与协议版本常量。
//
// 输入是 `export-schema`（Rust）写出的 `protocol.json`；输出是 `protocol.generated.ts`。
// 本脚本不读 Rust 源码，只消费 schema，保证类型只有一处定义。

import { readFile, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { compile } from "json-schema-to-typescript";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const generatedDir = join(root, "packages", "adapter", "src", "generated");
const schemaPath = join(generatedDir, "protocol.json");
const outputPath = join(generatedDir, "protocol.generated.ts");

const HEADER = [
  "/**",
  " * 本文件由 `pnpm run gen:protocol` 生成，请勿手改。",
  " * 类型的唯一来源是 `crates/core/src/protocol.rs`：Rust 类型 → JSON Schema → 这里。",
  " */",
  "",
].join("\n");

async function main() {
  const raw = await readFile(schemaPath, "utf8");
  const bundle = JSON.parse(raw);
  const schema = bundle.schema;
  if (typeof schema !== "object" || schema === null) {
    throw new Error(`schema 文件缺少 schema 字段：${schemaPath}`);
  }

  const types = await compile(schema, "WireContract", {
    bannerComment: "",
    additionalProperties: false,
    declareExternallyReferenced: true,
    style: {
      singleQuote: false,
      semi: true,
      printWidth: 100,
    },
  });

  const major = String(bundle.protocolVersion).split(".")[0];
  const degradationCodes = bundle.degradationCodes ?? [];
  const skipCodes = bundle.injectionSkipCodes ?? [];
  const degradationMessages = bundle.degradationMessages ?? {};
  const body = [
    HEADER,
    `export const PROTOCOL_VERSION = ${JSON.stringify(bundle.protocolVersion)};`,
    `export const PROTOCOL_MAJOR = ${major};`,
    "",
    "// 降级编码与跳过原因同样是契约；编码表由 Rust 侧 `degrade` 模块导出。",
    `export const DEGRADATION_CODES = ${JSON.stringify(degradationCodes, null, 2)} as const;`,
    "export type DegradationCode = (typeof DEGRADATION_CODES)[number];",
    `export const DEGRADATION_MESSAGES: Record<DegradationCode, string> = ${JSON.stringify(
      degradationMessages,
      null,
      2,
    )};`,
    "",
    `export const INJECTION_SKIP_CODES = ${JSON.stringify(skipCodes, null, 2)} as const;`,
    "export type InjectionSkipCode = (typeof INJECTION_SKIP_CODES)[number];",
    "",
    "// 运行时校验也读这份 schema，因此它随类型一起生成，不单独维护。",
    `export const PROTOCOL_SCHEMA: Record<string, unknown> = ${JSON.stringify(schema, null, 2)};`,
    "",
    types.trim(),
    "",
  ].join("\n");

  await writeFile(outputPath, body, "utf8");
  console.log(`已生成 ${outputPath}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
