import { CoreProtocolError, CoreUnreachableError } from "./errors.ts";
import { ADAPTER_VERSION } from "./version.ts";
import {
  PROTOCOL_MAJOR,
  PROTOCOL_VERSION,
  type CreateObservationRequest,
  type EstimateTokensResponse,
  type HandshakeRequest,
  type HandshakeResponse,
  type Observation,
  type ObservationList,
  type ProviderConfig,
  type ProviderConfigSnapshot,
} from "./generated/protocol.generated.ts";
import {
  parseEstimateTokensResponse,
  parseHandshakeResponse,
  parseObservation,
  parseObservationList,
  parseProviderConfigSnapshot,
  readErrorBody,
} from "./validate.ts";

/** 默认连接地址；实际使用时优先读 `OMW_CORE_ENDPOINT`。 */
export const DEFAULT_ENDPOINT = "http://127.0.0.1:7419";

/** 从环境变量解析 Core 地址。 */
export function resolveEndpoint(env: NodeJS.ProcessEnv = process.env): string {
  const configured = env.OMW_CORE_ENDPOINT?.trim();
  return normalizeEndpoint(configured && configured.length > 0 ? configured : DEFAULT_ENDPOINT);
}

export interface CoreConnectionOptions {
  endpoint?: string;
  protocolVersion?: string;
  adapterVersion?: string;
  timeoutMs?: number;
}

/** 查询 Observation 的条件；都不传时读最近的默认条数。 */
export interface ObservationQuery {
  sessionId?: string;
  limit?: number;
}

/**
 * Adapter 面向 Core 的连接层。
 *
 * 所有网络失败翻译成 [`CoreUnreachableError`]，所有 Core 拒绝翻译成 [`CoreProtocolError`]，
 * 调用方不需要自己解析错误信封或校验响应形状。
 */
export class CoreConnection {
  readonly endpoint: string;
  readonly protocolVersion: string;
  readonly adapterVersion: string;
  readonly timeoutMs: number;

  constructor(options: CoreConnectionOptions = {}) {
    this.endpoint = normalizeEndpoint(options.endpoint ?? resolveEndpoint());
    this.protocolVersion = options.protocolVersion ?? PROTOCOL_VERSION;
    this.adapterVersion = options.adapterVersion ?? ADAPTER_VERSION;
    this.timeoutMs = options.timeoutMs ?? 5_000;
  }

  /** 完成一次带协议版本号的握手；版本不匹配时抛 [`CoreProtocolError`]。 */
  async handshake(): Promise<HandshakeResponse> {
    // 请求体按生成的 HandshakeRequest 构造：Core 给请求加必填字段时，这里编译失败。
    const request: HandshakeRequest = {
      protocolVersion: this.protocolVersion,
      adapterVersion: this.adapterVersion,
    };
    const payload = await this.request("POST", "/v1/handshake", request);
    const handshake = parseHandshakeResponse(payload);
    this.assertCompatibleVersion(handshake.protocolVersion);
    return handshake;
  }

  /** 写入一条 Observation；返回落盘后的条目。 */
  async recordObservation(input: CreateObservationRequest): Promise<Observation> {
    const payload = await this.request("POST", "/v1/observations", input);
    return parseObservation(payload);
  }

  /** 读取 Observation；不传 `sessionId` 时读最近的若干条。 */
  async observations(query: ObservationQuery = {}): Promise<ObservationList> {
    const params = new URLSearchParams();
    if (query.sessionId !== undefined) {
      params.set("sessionId", query.sessionId);
    }
    if (query.limit !== undefined) {
      params.set("limit", String(query.limit));
    }
    const suffix = params.size > 0 ? `?${params.toString()}` : "";
    const payload = await this.request("GET", `/v1/observations${suffix}`);
    return parseObservationList(payload);
  }

  /** 读取 Core 自持的 provider 配置快照。 */
  async readConfig(): Promise<ProviderConfigSnapshot> {
    const payload = await this.request("GET", "/v1/config");
    return parseProviderConfigSnapshot(payload);
  }

  /** 写入一份 provider 配置；不合法的配置由 Core 拒绝并说明原因。 */
  async writeConfig(config: ProviderConfig): Promise<ProviderConfigSnapshot> {
    const payload = await this.request("PUT", "/v1/config", config);
    return parseProviderConfigSnapshot(payload);
  }

  /**
   * 让 Core 估算一段文本的 token 数。
   *
   * 换算口径只在 Core 实现一次：Adapter 不写第二个除法，也绝不用 `length / 4`
   * 之类的方式自算。
   */
  async estimateTokens(text: string): Promise<EstimateTokensResponse> {
    const payload = await this.request("POST", "/v1/tokens/estimate", { text });
    return parseEstimateTokensResponse(payload);
  }

  /**
   * 自己的校验不能省：Core 的拒绝只是它那一侧的判断，对端也可能更宽松或行为异常。
   * 对端版本若与本 Adapter 主版本号不同，任何读到的数据都不可信。
   */
  private assertCompatibleVersion(reported: string): void {
    const reportedMajor = reported.trim().split(".")[0];
    if (reportedMajor !== String(PROTOCOL_MAJOR)) {
      throw new CoreProtocolError(
        `handshake 时对端报告的协议版本是 ${reported}，与本 Adapter 的 ${PROTOCOL_VERSION} 主版本号不同，拒绝使用。`,
        "protocol_version_mismatch",
        undefined,
        { reported, supported: this.protocolVersion },
      );
    }
  }

  private async request(method: string, path: string, body?: unknown): Promise<unknown> {
    const headers: Record<string, string> = {};
    if (body !== undefined) {
      headers["content-type"] = "application/json";
    }

    let response: Response;
    try {
      const init: RequestInit = {
        method,
        headers,
        signal: AbortSignal.timeout(this.timeoutMs),
      };
      if (body !== undefined) {
        init.body = JSON.stringify(body);
      }
      response = await fetch(`${this.endpoint}${path}`, init);
    } catch (cause) {
      throw new CoreUnreachableError(this.endpoint, cause);
    }

    const payload = await readJson(response);
    if (!response.ok) {
      throw toProtocolError(response.status, payload);
    }
    return payload;
  }
}

function normalizeEndpoint(endpoint: string): string {
  return endpoint.replace(/\/+$/, "");
}

async function readJson(response: Response): Promise<unknown> {
  const raw = await response.text();
  if (raw.trim().length === 0) {
    return null;
  }
  try {
    return JSON.parse(raw) as unknown;
  } catch {
    return null;
  }
}

function toProtocolError(status: number, payload: unknown): CoreProtocolError {
  const body = readErrorBody(payload);
  if (body !== undefined) {
    return new CoreProtocolError(body.error.message, body.error.code, status, body.error.details);
  }
  return new CoreProtocolError(
    `Core 返回了 HTTP ${status}，但没有可读的错误信息。`,
    "malformed_error",
    status,
  );
}
