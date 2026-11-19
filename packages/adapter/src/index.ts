export {
  CoreConnection,
  DEFAULT_ENDPOINT,
  resolveEndpoint,
  type CoreConnectionOptions,
  type ObservationQuery,
} from "./connection.ts";
export { CoreProtocolError, CoreUnreachableError } from "./errors.ts";
export { ADAPTER_VERSION } from "./version.ts";
export {
  DEGRADATION_CODES,
  DEGRADATION_MESSAGES,
  INJECTION_SKIP_CODES,
  PROTOCOL_MAJOR,
  PROTOCOL_SCHEMA,
  PROTOCOL_VERSION,
} from "./generated/protocol.generated.ts";
export type {
  Capability,
  CoreIdentity,
  CreateObservationRequest,
  CredentialKind,
  CredentialRef,
  DegradationCode,
  ErrorBody,
  ErrorPayload,
  HandshakeRequest,
  HandshakeResponse,
  InjectedItem,
  InjectionExplanation,
  InjectionSkipCode,
  LedgerKind,
  Observation,
  ObservationList,
  ProviderConfig,
  ProviderConfigSnapshot,
  ProviderEntry,
  ReadyLine,
  Role,
  RoleBinding,
  RoleChain,
  SkippedRecall,
} from "./generated/protocol.generated.ts";
