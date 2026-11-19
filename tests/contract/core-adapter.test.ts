/**
 * 接缝一：Core 的进程间契约。
 *
 * 这里只用真实进程说话：启动真实的 omw-core 二进制，用 Adapter 的公开连接层与之交互，
 * 断言外部可观察的行为。不 import Core 内部类型，也不读数据文件的内部结构。
 *
 * 每个场景都按 Pass^k 重复（见 docs/spec/v1.md 的 Testing Decisions）：单次成功不认。
 */
import assert from "node:assert/strict";
import { readFile, readdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { describe, it } from "node:test";

import {
  CoreConnection,
  CoreProtocolError,
  CoreUnreachableError,
  PROTOCOL_VERSION,
} from "../../packages/adapter/src/index.ts";
import {
  PASS_K,
  dataFileBytes,
  findNonLoopbackIpv4,
  passK,
  runCoreOnce,
  startCore,
  startStubServer,
  tempConfigPath,
  tempDataDir,
  tryTcpConnect,
  type RunningCore,
} from "./helpers.ts";

const DATA_FILE = "core.db";

/** 先停 Core 再删目录：Windows 上运行中的 SQLite 文件不能被删除。 */
async function stopAndRemove(
  cores: readonly RunningCore[],
  dirs: ReadonlyArray<string | undefined>,
): Promise<void> {
  for (const core of cores) {
    await core.stop().catch(() => undefined);
  }
  for (const dir of dirs) {
    if (dir === undefined) {
      continue;
    }
    await rm(dir, { recursive: true, force: true }).catch(() => undefined);
  }
}

describe("两端契约", () => {
  it(`启动 Core 只创建一个数据文件，重启后 Observation 仍在（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const first = await startCore({ dataDir: dir, configPath });
      try {
        assert.deepEqual(await readdir(dir), [DATA_FILE], "数据目录里应只有一个数据文件");

        const connection = new CoreConnection({ endpoint: first.ready.endpoint });
        const written = await connection.recordObservation({
          sessionId: "session-restart",
          source: "host.message",
          speaker: "user",
          content: "我偏好简洁的回答",
        });
        assert.ok(written.id > 0);
        await first.stop();

        const second = await startCore({ dataDir: dir, configPath });
        try {
          const read = await new CoreConnection({
            endpoint: second.ready.endpoint,
          }).observations({ sessionId: "session-restart" });
          assert.equal(read.observations.length, 1, "重启后应读回写入的那条 Observation");
          assert.equal(read.observations[0]?.content, "我偏好简洁的回答");
          assert.equal(read.observations[0]?.speaker, "user");
          assert.deepEqual(await readdir(dir), [DATA_FILE], "重启不应产生额外数据文件");
        } finally {
          await second.stop();
        }
      } finally {
        await stopAndRemove([], [dir]);
      }
    });
  });

  it(`Adapter 能连上 Core 并完成一次协议版本握手（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const handshake = await new CoreConnection({ endpoint: core.ready.endpoint }).handshake();
        assert.equal(handshake.protocolVersion, PROTOCOL_VERSION, "两端应报告同一个协议版本");
        assert.equal(handshake.coreVersion, core.ready.coreVersion);
        assert.equal(handshake.schemaVersion, core.ready.schemaVersion);
        assert.equal(handshake.startedAt, core.ready.startedAt);
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });

  it(`协议主版本不匹配时拒绝连接并给出可读原因（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const mismatched = new CoreConnection({
          endpoint: core.ready.endpoint,
          protocolVersion: "2.0.0",
        });

        await assert.rejects(mismatched.handshake(), (error: unknown) => {
          assert.ok(
            error instanceof CoreProtocolError,
            `应抛 CoreProtocolError，实际：${String(error)}`,
          );
          assert.equal(error.code, "protocol_version_mismatch");
          assert.equal(error.status, 409, "Core 的拒绝应带真实的 HTTP 状态码");
          assert.match(error.message, /协议版本不匹配/, "拒绝原因应是人能读懂的一句话");
          assert.match(error.message, /2\.0\.0/, "原因里应带 Adapter 请求的版本");
          assert.ok(error.message.includes(PROTOCOL_VERSION), "原因里应带 Core 支持的版本");
          return true;
        });
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });

  it(`握手请求不是合法 JSON 时返回可读错误（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const response = await fetch(`${core.ready.endpoint}/v1/handshake`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: "{ 这不是 JSON",
        });
        assert.equal(response.status, 400);

        const body = (await response.json()) as { error?: { code?: string; message?: string } };
        assert.equal(body.error?.code, "invalid_request");
        assert.match(body.error?.message ?? "", /JSON/);
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });

  it(`Core 只监听 loopback，外部地址连不上（Pass^${PASS_K}）`, async (t) => {
    const externalAddress = await findNonLoopbackIpv4();
    if (externalAddress === undefined) {
      t.skip("本机没有非 loopback 的 IPv4 地址，跳过外部不可达断言");
      return;
    }
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const port = Number(new URL(core.ready.endpoint).port);
        assert.ok(Number.isInteger(port) && port > 0, `ready 行应带端口：${core.ready.endpoint}`);
        assert.equal(await tryTcpConnect("127.0.0.1", port, 2_000), true, "loopback 地址应可连接");
        assert.equal(
          await tryTcpConnect(externalAddress, port, 1_500),
          false,
          `Core 不应在非 loopback 地址 ${externalAddress}:${port} 上可达`,
        );
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });

  it(`Adapter 连不上 Core 时给出可读错误（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const endpoint = "http://127.0.0.1:1";
      await assert.rejects(
        new CoreConnection({ endpoint, timeoutMs: 2_000 }).handshake(),
        (error: unknown) => {
          assert.ok(
            error instanceof CoreUnreachableError,
            `应抛 CoreUnreachableError，实际：${String(error)}`,
          );
          assert.match(error.message, /无法连接 Core/);
          assert.ok(error.message.includes(endpoint), "报错应带上尝试连接的地址");
          assert.match(error.message, /omw-core/, "报错应提示怎么启动 Core");
          return true;
        },
      );
    });
  });

  it(`数据目录被一个普通文件占住时，Core 启动失败并给出可读错误（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      try {
        const notADirectory = join(dir, "data");
        await writeFile(notADirectory, "占位");

        const result = await runCoreOnce([
          "--data-dir",
          notADirectory,
          "--config",
          configPath,
          "--port",
          "0",
        ]);
        assert.notEqual(result.code, 0, "Core 应在启动阶段失败");
        assert.match(result.stderr, /omw-core: /);
        assert.match(result.stderr, /数据目录/);
        assert.match(result.stderr, /不是目录/);
      } finally {
        await stopAndRemove([], [dir]);
      }
    });
  });

  it(`数据文件损坏时，Core 启动失败并给出可读错误（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      try {
        await writeFile(join(dir, DATA_FILE), "这不是一个 SQLite 数据文件");
        const result = await runCoreOnce([
          "--data-dir",
          dir,
          "--config",
          configPath,
          "--port",
          "0",
        ]);
        assert.notEqual(result.code, 0, "Core 应在启动阶段失败");
        assert.match(result.stderr, /无法读取/);
        assert.match(result.stderr, /core\.db/);
      } finally {
        await stopAndRemove([], [dir]);
      }
    });
  });

  it(`端口被占用时，Core 启动失败并给出可读错误（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const firstDir = await tempDataDir();
      const firstConfig = await tempConfigPath();
      const secondDir = await tempDataDir();
      const secondConfig = await tempConfigPath();
      const first = await startCore({ dataDir: firstDir, configPath: firstConfig });
      try {
        const busyPort = new URL(first.ready.endpoint).port;
        const result = await runCoreOnce([
          "--data-dir",
          secondDir,
          "--config",
          secondConfig,
          "--port",
          busyPort,
        ]);
        assert.notEqual(result.code, 0, "第二个 Core 应启动失败，而不是悄悄换端口");
        assert.match(result.stderr, /端口.*已被占用/);
      } finally {
        await stopAndRemove([first], [firstDir, secondDir]);
      }
    });
  });

  it(`启动失败（端口占用）不应改动数据文件（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const blockerDir = await tempDataDir();
      const blockerConfig = await tempConfigPath();
      const targetDir = await tempDataDir();
      const targetConfig = await tempConfigPath();
      const blocker = await startCore({ dataDir: blockerDir, configPath: blockerConfig });
      try {
        // 先成功启动一次并写一条数据，记下文件当时的字节。
        const seeded = await startCore({ dataDir: targetDir, configPath: targetConfig });
        await new CoreConnection({ endpoint: seeded.ready.endpoint }).recordObservation({
          sessionId: "session-failure",
          source: "host.message",
          content: "失败启动不应动到我",
        });
        await seeded.stop();
        const before = await dataFileBytes(targetDir);

        // 再把端口占住，让第二次启动失败。
        const busyPort = new URL(blocker.ready.endpoint).port;
        const failed = await runCoreOnce([
          "--data-dir",
          targetDir,
          "--config",
          targetConfig,
          "--port",
          busyPort,
        ]);
        assert.notEqual(failed.code, 0, "第二次启动应失败");
        assert.deepEqual(
          await dataFileBytes(targetDir),
          before,
          "端口占用导致的启动失败不应改动数据文件",
        );
      } finally {
        await stopAndRemove([blocker], [blockerDir, targetDir]);
      }
    });
  });

  it(`Adapter 拒绝对端报告的不兼容协议版本（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      // 不启动真实 Core，用一个总是返回 2.0.0 的桩端点，单独测 Adapter 自己的校验。
      const server = await startStubServer(() => ({
        status: 200,
        body: JSON.stringify({
          protocolVersion: "2.0.0",
          coreVersion: "0.1.0",
          schemaVersion: 2,
          startedAt: "2026-01-01T00:00:00Z",
        }),
      }));
      try {
        await assert.rejects(
          new CoreConnection({ endpoint: server.endpoint }).handshake(),
          (error: unknown) => {
            assert.ok(
              error instanceof CoreProtocolError,
              `应抛 CoreProtocolError，实际：${String(error)}`,
            );
            assert.equal(error.code, "protocol_version_mismatch");
            assert.equal(
              error.status,
              undefined,
              "本地校验失败没有对应的 HTTP 状态码，不应伪造一个 200",
            );
            assert.match(error.message, /2\.0\.0/, "报错应带上对端报告的版本");
            return true;
          },
        );
      } finally {
        await server.close();
      }
    });
  });

  it(`Adapter 拒绝对端返回的畸形响应（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const server = await startStubServer(() => ({
        status: 200,
        body: JSON.stringify({ protocolVersion: "1.0.0" }),
      }));
      try {
        await assert.rejects(
          new CoreConnection({ endpoint: server.endpoint }).handshake(),
          (error: unknown) => {
            assert.ok(error instanceof CoreProtocolError, `应抛 CoreProtocolError`);
            assert.equal(error.code, "malformed_response");
            assert.match(error.message, /coreVersion/, "报错应点名缺了哪个字段");
            return true;
          },
        );
      } finally {
        await server.close();
      }
    });
  });

  it(`Observation 可按会话与时间查回（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const connection = new CoreConnection({ endpoint: core.ready.endpoint });
        await connection.recordObservation({
          sessionId: "a",
          source: "host.message",
          speaker: "user",
          content: "第一条",
        });
        await connection.recordObservation({
          sessionId: "a",
          source: "tool.call",
          speaker: "assistant",
          content: "调用了一次工具",
        });
        await connection.recordObservation({
          sessionId: "b",
          source: "host.message",
          content: "别的会话",
        });

        const onlyA = await connection.observations({ sessionId: "a" });
        assert.equal(onlyA.observations.length, 2, "应按会话过滤");
        assert.deepEqual(
          onlyA.observations.map((item) => item.content),
          ["第一条", "调用了一次工具"],
          "应按写入顺序返回，工具事件与消息都在",
        );
        for (const item of onlyA.observations) {
          assert.match(
            item.occurredAt,
            /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/,
            `时间应是 RFC3339 时刻：${item.occurredAt}`,
          );
        }

        const all = await connection.observations();
        assert.equal(all.observations.length, 3, "不按会话过滤时应读回全部");
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });

  it(`Observation 保留事件时刻，与摄入时刻分开（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const connection = new CoreConnection({ endpoint: core.ready.endpoint });
        const written = await connection.recordObservation({
          sessionId: "s",
          source: "host.message",
          content: "三天前说过的话",
          occurredAt: "2026-11-16T09:30:00Z",
        });
        assert.equal(written.occurredAt, "2026-11-16T09:30:00Z", "应保留事件发生时刻");

        const invalid = await fetch(`${core.ready.endpoint}/v1/observations`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            sessionId: "s",
            source: "host.message",
            content: "时间写错",
            occurredAt: "昨天下午",
          }),
        });
        assert.equal(invalid.status, 400, "非法 occurredAt 应在边界被拒绝");
        const body = (await invalid.json()) as { error?: { message?: string } };
        assert.match(body.error?.message ?? "", /occurredAt/);
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });

  it(`请求体超限也走统一错误信封（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const response = await fetch(`${core.ready.endpoint}/v1/observations`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            sessionId: "s",
            source: "host.message",
            content: "x".repeat(2 * 1024 * 1024),
          }),
        });
        assert.equal(response.status, 413, "超限请求应返回 413");
        const body = (await response.json()) as { error?: { code?: string; message?: string } };
        assert.equal(body.error?.code, "invalid_request", "413 也应是错误信封，不是纯文本");
        assert.match(body.error?.message ?? "", /上限/, "报错应说明上限");
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });

  it(`未配置 provider 时角色显式报告未配置，不隐式挑一个（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const snapshot = await new CoreConnection({
          endpoint: core.ready.endpoint,
        }).readConfig();
        assert.equal(snapshot.providers.length, 0);
        assert.deepEqual(
          [...snapshot.unconfiguredRoles].sort(),
          ["abstention", "extraction", "queryRewrite", "ranking"],
          "四个角色都应显式报告未配置",
        );
        assert.deepEqual(snapshot.outboundWhitelist, [], "没有 provider 时出站白名单为空");
        assert.deepEqual(snapshot.degradedProviders, [], "没有 provider 时没有降级项");
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });

  it(`可登记多个 provider 与有序 fallback 链，且跨重启保留（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const first = await startCore({ dataDir: dir, configPath });
      try {
        const connection = new CoreConnection({ endpoint: first.ready.endpoint });
        const saved = await connection.writeConfig({
          providers: [
            {
              name: "primary",
              endpoint: "http://127.0.0.1:9101/v1",
              capabilities: ["llm", "embedding"],
              credential: { kind: "env", reference: "PATH" },
            },
            {
              name: "fallback",
              endpoint: "http://127.0.0.1:9102/v1",
              capabilities: ["llm"],
              credential: { kind: "env", reference: "PATH" },
            },
          ],
            roles: [
              {
                role: "extraction",
                chain: [
                  { provider: "primary", capability: "llm" },
                  { provider: "fallback", capability: "llm" },
                ],
              },
            ],
        });

        assert.equal(saved.providers.length, 2);
        assert.equal(saved.roles[0]?.chain.length, 2, "fallback 链应完整保留");
        assert.equal(saved.roles[0]?.chain[1]?.provider, "fallback", "顺序应保持");
        assert.ok(!saved.unconfiguredRoles.includes("extraction"));
        assert.ok(saved.unconfiguredRoles.includes("ranking"));
        assert.deepEqual(
          [...saved.outboundWhitelist].sort(),
          ["http://127.0.0.1:9101/v1", "http://127.0.0.1:9102/v1"],
          "出站白名单应等于已配置 provider 的地址集合",
        );
        await first.stop();

        const second = await startCore({ dataDir: dir, configPath });
        try {
          const reloaded = await new CoreConnection({
            endpoint: second.ready.endpoint,
          }).readConfig();
          assert.equal(reloaded.providers.length, 2, "配置应跨重启保留");
          assert.equal(reloaded.roles[0]?.chain[1]?.provider, "fallback");
        } finally {
          await second.stop();
        }
      } finally {
        await stopAndRemove([], [dir]);
      }
    });
  });

  it(`非法 provider 配置在写入时被拒绝并给出原因（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const connection = new CoreConnection({ endpoint: core.ready.endpoint });

        const badCapability = await fetch(`${core.ready.endpoint}/v1/config`, {
          method: "PUT",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            providers: [
              {
                name: "p",
                endpoint: "http://127.0.0.1:9101/v1",
                capabilities: ["telepathy"],
                credential: { kind: "env", reference: "PATH" },
              },
            ],
            roles: [],
          }),
        });
        assert.equal(badCapability.status, 400, "未知能力应在写入时被拒绝");

        await assert.rejects(
          connection.writeConfig({
            providers: [
              {
                name: "p",
                endpoint: "http://127.0.0.1:9101/v1",
                capabilities: ["llm"],
                credential: { kind: "env", reference: "PATH" },
              },
            ],
            roles: [{ role: "ranking", chain: [{ provider: "缺失", capability: "llm" }] }],
          }),
          (error: unknown) => {
            assert.ok(error instanceof CoreProtocolError, "应抛 CoreProtocolError");
            assert.equal(error.code, "invalid_config");
            assert.equal(error.status, 400);
            assert.match(error.message, /缺失/, "报错应点名悬空引用的 provider");
            return true;
          },
        );

        await assert.rejects(
          connection.writeConfig({
            providers: [
              {
                name: "p",
                endpoint: "http://127.0.0.1:9101/v1",
                capabilities: ["llm"],
                credential: { kind: "env", reference: "OMW_这个环境变量不存在" },
              },
            ],
            roles: [],
          }),
          (error: unknown) => {
            assert.ok(error instanceof CoreProtocolError, "应抛 CoreProtocolError");
            assert.equal(error.code, "invalid_config");
            assert.match(error.message, /OMW_这个环境变量不存在/, "报错应点名缺失的凭据引用");
            return true;
          },
        );

        // 被拒绝的写入不应留下半截配置。
        const snapshot = await connection.readConfig();
        assert.equal(snapshot.providers.length, 0, "非法配置不应落盘");
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });

  it(`凭据取不到的 provider 在启动后降级报告，而不是拒绝启动（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      // 先写一份引用了「此刻取不到」的凭据的配置，再重启 Core。
      const first = await startCore({ dataDir: dir, configPath });
      try {
        await new CoreConnection({ endpoint: first.ready.endpoint }).writeConfig({
          providers: [
            {
              name: "will-lose-credential",
              endpoint: "http://127.0.0.1:9103/v1",
              capabilities: ["llm"],
              credential: { kind: "env", reference: "PATH" },
            },
          ],
          roles: [
            {
              role: "extraction",
              chain: [{ provider: "will-lose-credential", capability: "llm" }],
            },
          ],
        });
      } finally {
        await first.stop();
      }

      // 手工把配置改成指向一个不存在的环境变量，模拟换机器后凭据丢失。
      const raw = JSON.parse(await readFile(configPath, "utf8")) as {
        providers: Array<{ credential: { kind: string; reference: string } }>;
      };
      const provider = raw.providers[0];
      assert.ok(provider !== undefined);
      provider.credential.reference = "OMW_启动后取不到的凭据";
      await writeFile(configPath, JSON.stringify(raw));

      const second = await startCore({ dataDir: dir, configPath });
      try {
        const snapshot = await new CoreConnection({
          endpoint: second.ready.endpoint,
        }).readConfig();
        assert.deepEqual(
          snapshot.degradedProviders,
          ["will-lose-credential"],
          "凭据取不到的 provider 应显式报告降级，而不是让 Core 起不来",
        );
        assert.ok(
          snapshot.outboundWhitelist.includes("http://127.0.0.1:9103/v1"),
          "降级的 provider 仍留在出站白名单里",
        );
      } finally {
        await second.stop();
      }
      await stopAndRemove([], [dir]);
    });
  });

  it(`未知接口返回带可用接口列表的错误信封（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const dir = await tempDataDir();
      const configPath = await tempConfigPath();
      const core = await startCore({ dataDir: dir, configPath });
      try {
        const missing = await fetch(`${core.ready.endpoint}/v1/health`);
        assert.equal(missing.status, 404, "health 不在契约里，应返回 404");
        const body = (await missing.json()) as { error?: { code?: string; message?: string } };
        assert.equal(body.error?.code, "not_found");
        assert.match(body.error?.message ?? "", /\/v1\/handshake/, "报错应列出可用接口");

        const wrongMethod = await fetch(`${core.ready.endpoint}/v1/handshake`);
        assert.equal(wrongMethod.status, 405);
        const wrongBody = (await wrongMethod.json()) as { error?: { code?: string } };
        assert.equal(wrongBody.error?.code, "method_not_allowed");
      } finally {
        await stopAndRemove([core], [dir]);
      }
    });
  });
});
