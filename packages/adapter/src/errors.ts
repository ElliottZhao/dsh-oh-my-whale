/** Core 没起来（或地址写错）时的错误：提示用户先启动 Core。 */
export class CoreUnreachableError extends Error {
  readonly endpoint: string;

  constructor(endpoint: string, cause?: unknown) {
    super(
      `无法连接 Core（${endpoint}）。请先启动 Core：omw-core；` +
        `或用 --endpoint / OMW_CORE_ENDPOINT 指向正在运行的 Core。`,
      { cause },
    );
    this.name = "CoreUnreachableError";
    this.endpoint = endpoint;
  }
}

/**
 * Core 明确拒绝、或对端响应不符合协议时的错误。
 *
 * 与 HTTP 响应无关的本地校验（协议版本不符、响应畸形）没有对应的状态码，
 * 此时 `status` 为 `undefined`，不要伪造一个 200。
 */
export class CoreProtocolError extends Error {
  readonly code: string;
  readonly status: number | undefined;
  readonly details: unknown;

  constructor(message: string, code: string, status?: number, details?: unknown) {
    super(message);
    this.name = "CoreProtocolError";
    this.code = code;
    this.status = status;
    this.details = details;
  }
}
