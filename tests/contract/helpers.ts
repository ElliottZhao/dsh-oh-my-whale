import { spawn, type ChildProcess } from "node:child_process";
import { createSocket } from "node:dgram";
import { mkdtemp, readFile } from "node:fs/promises";
import { connect } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import type { ReadyLine } from "../../packages/adapter/src/generated/protocol.generated.ts";

/** 被测的 Core 二进制；CI 或脚本可用 OMW_CORE_BIN 覆盖。 */
export const CORE_BIN =
  process.env.OMW_CORE_BIN ??
  resolve("target", "debug", process.platform === "win32" ? "omw-core.exe" : "omw-core");

/** 同一场景重复跑几次都通过才算通过（Pass^k，见 docs/spec/v1.md 的 Testing Decisions）。 */
export const PASS_K = Number(process.env.OMW_PASS_K ?? 3);

export type { ReadyLine };

export interface RunningCore {
  ready: ReadyLine;
  stop: () => Promise<void>;
}

export interface ProcessResult {
  code: number | null;
  stdout: string;
  stderr: string;
}

export async function tempDataDir(): Promise<string> {
  return mkdtemp(join(tmpdir(), "omw-contract-"));
}

/** Core 自持的 provider 配置文件路径；与数据目录分开，保证数据目录里只有一个数据文件。 */
export async function tempConfigPath(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "omw-config-"));
  return join(dir, "config.json");
}

/** 把同一个场景按 Pass^k 跑 `PASS_K` 遍；失败信息带上第几遍。 */
export async function passK(body: (run: number) => Promise<void>): Promise<void> {
  for (let run = 1; run <= PASS_K; run += 1) {
    try {
      await body(run);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`第 ${run}/${PASS_K} 遍未通过：${message}`, { cause: error });
    }
  }
}

/** 启动真实 Core，等它打印 ready 行。 */
export async function startCore(options: {
  dataDir: string;
  configPath: string;
  port?: number;
  timeoutMs?: number;
}): Promise<RunningCore> {
  const timeoutMs = options.timeoutMs ?? 20_000;
  const child = spawn(
    CORE_BIN,
    [
      "--data-dir",
      options.dataDir,
      "--config",
      options.configPath,
      "--port",
      String(options.port ?? 0),
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );

  let stdout = "";
  let stderr = "";
  child.stdout?.on("data", (chunk: Buffer) => {
    stdout += chunk.toString();
  });
  child.stderr?.on("data", (chunk: Buffer) => {
    stderr += chunk.toString();
  });

  const ready = await new Promise<ReadyLine>((resolveReady, reject) => {
    const timer = setTimeout(() => {
      cleanup();
      reject(
        new Error(
          `Core 未在 ${timeoutMs}ms 内就绪。\nstdout:\n${stdout}\nstderr:\n${stderr}`,
        ),
      );
    }, timeoutMs);

    const onStdout = (): void => {
      const line = stdout
        .split(/\r?\n/)
        .find((candidate) => candidate.includes('"event":"core.ready"'));
      if (line !== undefined) {
        cleanup();
        resolveReady(JSON.parse(line) as ReadyLine);
      }
    };

    const onExit = (code: number | null, signal: NodeJS.Signals | null): void => {
      cleanup();
      reject(
        new Error(
          `Core 在就绪前退出（code=${String(code)}, signal=${String(signal)}）。\nstdout:\n${stdout}\nstderr:\n${stderr}`,
        ),
      );
    };

    const cleanup = (): void => {
      clearTimeout(timer);
      child.stdout?.off("data", onStdout);
      child.off("exit", onExit);
    };

    child.stdout?.on("data", onStdout);
    child.on("exit", onExit);
  });

  return {
    ready,
    stop: async (): Promise<void> => {
      await stopChild(child);
    },
  };
}

/** 跑一个 Core 进程直到它自己退出，用于断言启动失败与可读错误。 */
export async function runCoreOnce(
  args: readonly string[],
  timeoutMs = 20_000,
): Promise<ProcessResult> {
  const child = spawn(CORE_BIN, args, { stdio: ["ignore", "pipe", "pipe"] });
  return collectUntilExit(child, timeoutMs);
}

/** 读数据文件的字节，用来断言失败的启动没有改动它。 */
export async function dataFileBytes(dataDir: string): Promise<Buffer> {
  return readFile(join(dataDir, "core.db"));
}

/** 本机的非 loopback IPv4 地址；没有则返回 undefined（例如只有回环网卡的机器）。 */
export function findNonLoopbackIpv4(): Promise<string | undefined> {
  return new Promise((resolveAddress) => {
    const socket = createSocket("udp4");
    const finish = (address: string | undefined): void => {
      socket.close();
      resolveAddress(address);
    };
    socket.once("error", () => finish(undefined));
    socket.connect(53, "8.8.8.8", () => {
      const address = socket.address().address;
      finish(
        address !== undefined && !address.startsWith("127.") && address !== "0.0.0.0"
          ? address
          : undefined,
      );
    });
  });
}

export function tryTcpConnect(host: string, port: number, timeoutMs: number): Promise<boolean> {
  return new Promise((resolveConnected) => {
    const socket = connect({ host, port });
    const finish = (connected: boolean): void => {
      socket.destroy();
      resolveConnected(connected);
    };
    socket.setTimeout(timeoutMs);
    socket.once("connect", () => finish(true));
    socket.once("error", () => finish(false));
    socket.once("timeout", () => finish(false));
  });
}

async function stopChild(child: ChildProcess): Promise<void> {
  if (child.exitCode !== null || child.signalCode !== null) {
    return;
  }
  const exited = new Promise<void>((resolveExit) => child.once("exit", () => resolveExit()));
  child.kill();
  await exited;
}

/** 一个只会返回固定 JSON 的桩 HTTP 服务，用于单独测 Adapter 的行为。 */
export async function startStubServer(
  respond: (req: { method: string; url: string | undefined }) => { status: number; body: string },
): Promise<{ endpoint: string; close: () => Promise<void> }> {
  const { createServer } = await import("node:http");
  const server = createServer((req, res) => {
    const { status, body } = respond({
      method: req.method ?? "",
      url: req.url,
    });
    res.statusCode = status;
    res.setHeader("content-type", "application/json");
    res.end(body);
  });
  await new Promise<void>((resolveListen) =>
    server.listen(0, "127.0.0.1", () => resolveListen()),
  );
  const address = server.address();
  if (address === null || typeof address === "string") {
    throw new Error("桩服务没有取到监听地址");
  }
  const endpoint = `http://127.0.0.1:${address.port}`;
  return {
    endpoint,
    close: async (): Promise<void> => {
      await new Promise<void>((resolveClose, rejectClose) =>
        server.close((error) => (error ? rejectClose(error) : resolveClose())),
      );
    },
  };
}

function collectUntilExit(child: ChildProcess, timeoutMs: number): Promise<ProcessResult> {
  let stdout = "";
  let stderr = "";
  child.stdout?.on("data", (chunk: Buffer) => {
    stdout += chunk.toString();
  });
  child.stderr?.on("data", (chunk: Buffer) => {
    stderr += chunk.toString();
  });

  return new Promise<ProcessResult>((resolveResult, reject) => {
    const timer = setTimeout(() => {
      child.kill();
      reject(new Error(`进程未在 ${timeoutMs}ms 内退出。\nstdout:\n${stdout}\nstderr:\n${stderr}`));
    }, timeoutMs);

    child.once("exit", (code) => {
      clearTimeout(timer);
      resolveResult({ code, stdout, stderr });
    });
    child.once("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
  });
}
