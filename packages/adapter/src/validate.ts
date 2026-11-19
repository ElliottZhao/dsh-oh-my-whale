/**
 * 对端响应的运行时校验。
 *
 * 校验规则全部来自生成的 JSON Schema（`protocol.generated.ts`），不手写字段守卫；
 * schema 变了，这里的校验自动跟着变。
 */

import { Ajv, type AnySchema, type ValidateFunction } from "ajv";

import { CoreProtocolError } from "./errors.ts";
import {
  PROTOCOL_SCHEMA,
  type ErrorBody,
  type HandshakeResponse,
  type Observation,
  type ObservationList,
  type ProviderConfigSnapshot,
} from "./generated/protocol.generated.ts";

const ajv = new Ajv({
  allErrors: true,
  strict: false,
  allowUnionTypes: true,
  validateFormats: false,
});
ajv.addSchema(PROTOCOL_SCHEMA as AnySchema, "protocol");

function validator<T>(definition: string): ValidateFunction<T> {
  const validate = ajv.getSchema<T>(`protocol#/definitions/${definition}`);
  if (validate === undefined) {
    throw new Error(`协议 schema 缺少定义 ${definition}；先跑 pnpm run gen:protocol。`);
  }
  return validate;
}

const validators = {
  handshakeResponse: validator<HandshakeResponse>("HandshakeResponse"),
  observation: validator<Observation>("Observation"),
  observationList: validator<ObservationList>("ObservationList"),
  providerConfigSnapshot: validator<ProviderConfigSnapshot>("ProviderConfigSnapshot"),
  errorBody: validator<ErrorBody>("ErrorBody"),
};

/** 校验一段对端响应；不合法时抛带 schema 细节的 [`CoreProtocolError`]。 */
export function parseHandshakeResponse(payload: unknown): HandshakeResponse {
  return assertShape(validators.handshakeResponse, payload, "handshake");
}

export function parseObservation(payload: unknown): Observation {
  return assertShape(validators.observation, payload, "Observation 写入");
}

export function parseObservationList(payload: unknown): ObservationList {
  return assertShape(validators.observationList, payload, "Observation 查询");
}

export function parseProviderConfigSnapshot(payload: unknown): ProviderConfigSnapshot {
  return assertShape(validators.providerConfigSnapshot, payload, "provider 配置");
}

/** 读取错误信封；不是信封时返回 `undefined`，由调用方给兜底文案。 */
export function readErrorBody(payload: unknown): ErrorBody | undefined {
  return validators.errorBody(payload) ? payload : undefined;
}

function assertShape<T>(validate: ValidateFunction<T>, payload: unknown, what: string): T {
  if (validate(payload)) {
    return payload;
  }
  throw new CoreProtocolError(
    `对端 ${what} 响应不符合协议 schema（${describe(validate.errors)}）。`,
    "malformed_response",
    undefined,
  );
}

function describe(errors: ValidateFunction["errors"]): string {
  if (errors === null || errors === undefined || errors.length === 0) {
    return "没有给出细节";
  }
  return errors
    .slice(0, 3)
    .map((error) => {
      const path = error.instancePath === "" ? "/" : error.instancePath;
      return `${path} ${error.message ?? "不合法"}`;
    })
    .join("；");
}
