export {
  CoreConnection,
  DEFAULT_ENDPOINT,
  resolveEndpoint,
  type CoreConnectionOptions,
  type ObservationQuery,
} from "./connection.ts";
export { CoreProtocolError, CoreUnreachableError } from "./errors.ts";
export { ADAPTER_VERSION } from "./version.ts";
export { PROTOCOL_MAJOR, PROTOCOL_VERSION } from "./generated/protocol.generated.ts";
export type {
  Capability,
  CoreIdentity,
  CreateObservationRequest,
  CredentialKind,
  CredentialRef,
  ErrorBody,
  ErrorPayload,
  HandshakeRequest,
  HandshakeResponse,
  Observation,
  ObservationList,
  ProviderConfig,
  ProviderConfigSnapshot,
  ProviderEntry,
  ReadyLine,
  Role,
  RoleBinding,
  RoleChain,
} from "./generated/protocol.generated.ts";
