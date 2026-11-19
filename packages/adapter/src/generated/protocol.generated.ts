/**
 * 本文件由 `pnpm run gen:protocol` 生成，请勿手改。
 * 类型的唯一来源是 `crates/core/src/protocol.rs`：Rust 类型 → JSON Schema → 这里。
 */

export const PROTOCOL_VERSION = "1.0.0";
export const PROTOCOL_MAJOR = 1;

// 运行时校验也读这份 schema，因此它随类型一起生成，不单独维护。
export const PROTOCOL_SCHEMA: Record<string, unknown> = {
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "Capability": {
      "description": "Provider 能提供的模型能力类别，按能力而非业务划分。",
      "enum": [
        "llm",
        "embedding",
        "crossEncoder",
        "decision"
      ],
      "type": "string"
    },
    "CoreIdentity": {
      "description": "两端共享的版本与进程信息。握手响应与 ready 行都平铺这几个字段。",
      "properties": {
        "coreVersion": {
          "type": "string"
        },
        "protocolVersion": {
          "type": "string"
        },
        "schemaVersion": {
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "startedAt": {
          "description": "本次进程启动的时间；重启会变。",
          "type": "string"
        }
      },
      "required": [
        "protocolVersion",
        "coreVersion",
        "schemaVersion",
        "startedAt"
      ],
      "type": "object"
    },
    "CreateObservationRequest": {
      "additionalProperties": false,
      "description": "一条待写入的 Observation；未给出事件时间时，Core 用摄入时刻。",
      "properties": {
        "content": {
          "type": "string"
        },
        "occurredAt": {
          "default": null,
          "description": "事件发生的时刻（RFC3339）。省略时 Core 用摄入时刻，两者不混为一谈。",
          "type": [
            "string",
            "null"
          ]
        },
        "sessionId": {
          "description": "会话标识；同一会话重复摄入由此归并（去重在 #8 完整实现）。",
          "type": "string"
        },
        "source": {
          "description": "来源，例如 `host.message`、`tool.call`。",
          "type": "string"
        },
        "speaker": {
          "default": null,
          "description": "说话者或产生者；工具事件可省略。",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "sessionId",
        "source",
        "content"
      ],
      "type": "object"
    },
    "CredentialKind": {
      "description": "凭据引用的种类：只存引用，不存密钥值。",
      "oneOf": [
        {
          "const": "env",
          "description": "环境变量的名字。",
          "type": "string"
        },
        {
          "const": "file",
          "description": "只属于 Core 的密钥文件路径。",
          "type": "string"
        }
      ]
    },
    "CredentialRef": {
      "additionalProperties": false,
      "properties": {
        "kind": {
          "$ref": "#/definitions/CredentialKind"
        },
        "reference": {
          "description": "`env` 时是环境变量名，`file` 时是文件路径；都不含密钥值。",
          "type": "string"
        }
      },
      "required": [
        "kind",
        "reference"
      ],
      "type": "object"
    },
    "ErrorBody": {
      "properties": {
        "error": {
          "$ref": "#/definitions/ErrorPayload"
        }
      },
      "required": [
        "error"
      ],
      "type": "object"
    },
    "ErrorPayload": {
      "properties": {
        "code": {
          "type": "string"
        },
        "details": true,
        "message": {
          "type": "string"
        }
      },
      "required": [
        "code",
        "message"
      ],
      "type": "object"
    },
    "HandshakeRequest": {
      "additionalProperties": false,
      "properties": {
        "adapterVersion": {
          "default": null,
          "type": [
            "string",
            "null"
          ]
        },
        "protocolVersion": {
          "type": "string"
        }
      },
      "required": [
        "protocolVersion"
      ],
      "type": "object"
    },
    "HandshakeResponse": {
      "description": "握手成功时返回的 Core 身份。",
      "properties": {
        "coreVersion": {
          "type": "string"
        },
        "protocolVersion": {
          "type": "string"
        },
        "schemaVersion": {
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "startedAt": {
          "description": "本次进程启动的时间；重启会变。",
          "type": "string"
        }
      },
      "required": [
        "protocolVersion",
        "coreVersion",
        "schemaVersion",
        "startedAt"
      ],
      "type": "object"
    },
    "Observation": {
      "description": "已落盘的 Observation。`occurredAt` 是事件时刻，由请求给出或用摄入时刻补。",
      "properties": {
        "content": {
          "type": "string"
        },
        "id": {
          "format": "int64",
          "type": "integer"
        },
        "occurredAt": {
          "type": "string"
        },
        "sessionId": {
          "type": "string"
        },
        "source": {
          "type": "string"
        },
        "speaker": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "id",
        "sessionId",
        "source",
        "content",
        "occurredAt"
      ],
      "type": "object"
    },
    "ObservationList": {
      "properties": {
        "observations": {
          "items": {
            "$ref": "#/definitions/Observation"
          },
          "type": "array"
        }
      },
      "required": [
        "observations"
      ],
      "type": "object"
    },
    "ProviderConfig": {
      "additionalProperties": false,
      "description": "Core 自持的 provider 配置；出站白名单等于这里的 endpoint 集合。",
      "properties": {
        "providers": {
          "default": [],
          "items": {
            "$ref": "#/definitions/ProviderEntry"
          },
          "type": "array"
        },
        "roles": {
          "default": [],
          "items": {
            "$ref": "#/definitions/RoleChain"
          },
          "type": "array"
        }
      },
      "type": "object"
    },
    "ProviderConfigSnapshot": {
      "description": "读取或写入配置后的视图：补上未配置的角色、凭据取不到的 provider 与派生的出站白名单。",
      "properties": {
        "degradedProviders": {
          "description": "凭据引用此刻取不到的 provider；它们仍在白名单里，但用到时按 ADR-0003 降级。",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "outboundWhitelist": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "providers": {
          "items": {
            "$ref": "#/definitions/ProviderEntry"
          },
          "type": "array"
        },
        "roles": {
          "items": {
            "$ref": "#/definitions/RoleChain"
          },
          "type": "array"
        },
        "unconfiguredRoles": {
          "description": "没有任何角色绑定的能力角色；显式列出来，不隐式挑一个 provider。",
          "items": {
            "$ref": "#/definitions/Role"
          },
          "type": "array"
        }
      },
      "required": [
        "providers",
        "roles",
        "unconfiguredRoles",
        "degradedProviders",
        "outboundWhitelist"
      ],
      "type": "object"
    },
    "ProviderEntry": {
      "additionalProperties": false,
      "properties": {
        "capabilities": {
          "items": {
            "$ref": "#/definitions/Capability"
          },
          "type": "array"
        },
        "credential": {
          "$ref": "#/definitions/CredentialRef"
        },
        "endpoint": {
          "type": "string"
        },
        "name": {
          "type": "string"
        }
      },
      "required": [
        "name",
        "endpoint",
        "capabilities",
        "credential"
      ],
      "type": "object"
    },
    "ReadyLine": {
      "description": "Core 就绪后写到 stdout 的那一行。Adapter 与测试脚本都从它取端口。",
      "properties": {
        "coreVersion": {
          "type": "string"
        },
        "dataFile": {
          "type": "string"
        },
        "endpoint": {
          "type": "string"
        },
        "event": {
          "type": "string"
        },
        "pid": {
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "protocolVersion": {
          "type": "string"
        },
        "schemaVersion": {
          "format": "uint32",
          "minimum": 0,
          "type": "integer"
        },
        "startedAt": {
          "description": "本次进程启动的时间；重启会变。",
          "type": "string"
        }
      },
      "required": [
        "event",
        "endpoint",
        "dataFile",
        "pid",
        "protocolVersion",
        "coreVersion",
        "schemaVersion",
        "startedAt"
      ],
      "type": "object"
    },
    "Role": {
      "description": "Core 内部对某类工作的具名需求。",
      "enum": [
        "extraction",
        "queryRewrite",
        "ranking",
        "abstention"
      ],
      "type": "string"
    },
    "RoleBinding": {
      "additionalProperties": false,
      "description": "能力角色的一项绑定：某 provider 的某条能力。",
      "properties": {
        "capability": {
          "$ref": "#/definitions/Capability"
        },
        "provider": {
          "type": "string"
        }
      },
      "required": [
        "provider",
        "capability"
      ],
      "type": "object"
    },
    "RoleChain": {
      "additionalProperties": false,
      "description": "一个角色的有序 fallback 链。",
      "properties": {
        "chain": {
          "items": {
            "$ref": "#/definitions/RoleBinding"
          },
          "type": "array"
        },
        "role": {
          "$ref": "#/definitions/Role"
        }
      },
      "required": [
        "role",
        "chain"
      ],
      "type": "object"
    }
  },
  "description": "导出 JSON Schema 时把全部 wire 类型收进来的根对象。\n\n它只服务代码生成，不出现在任何请求或响应里。",
  "properties": {
    "capability": {
      "$ref": "#/definitions/Capability"
    },
    "coreIdentity": {
      "$ref": "#/definitions/CoreIdentity"
    },
    "createObservationRequest": {
      "$ref": "#/definitions/CreateObservationRequest"
    },
    "credentialKind": {
      "$ref": "#/definitions/CredentialKind"
    },
    "credentialRef": {
      "$ref": "#/definitions/CredentialRef"
    },
    "errorBody": {
      "$ref": "#/definitions/ErrorBody"
    },
    "errorPayload": {
      "$ref": "#/definitions/ErrorPayload"
    },
    "handshakeRequest": {
      "$ref": "#/definitions/HandshakeRequest"
    },
    "handshakeResponse": {
      "$ref": "#/definitions/HandshakeResponse"
    },
    "observation": {
      "$ref": "#/definitions/Observation"
    },
    "observationList": {
      "$ref": "#/definitions/ObservationList"
    },
    "providerConfig": {
      "$ref": "#/definitions/ProviderConfig"
    },
    "providerConfigSnapshot": {
      "$ref": "#/definitions/ProviderConfigSnapshot"
    },
    "providerEntry": {
      "$ref": "#/definitions/ProviderEntry"
    },
    "readyLine": {
      "$ref": "#/definitions/ReadyLine"
    },
    "role": {
      "$ref": "#/definitions/Role"
    },
    "roleBinding": {
      "$ref": "#/definitions/RoleBinding"
    },
    "roleChain": {
      "$ref": "#/definitions/RoleChain"
    }
  },
  "required": [
    "coreIdentity",
    "handshakeRequest",
    "handshakeResponse",
    "readyLine",
    "errorBody",
    "errorPayload",
    "createObservationRequest",
    "observation",
    "observationList",
    "capability",
    "role",
    "credentialKind",
    "credentialRef",
    "providerEntry",
    "roleBinding",
    "roleChain",
    "providerConfig",
    "providerConfigSnapshot"
  ],
  "title": "WireContract",
  "type": "object"
};

/**
 * Provider 能提供的模型能力类别，按能力而非业务划分。
 */
export type Capability = "llm" | "embedding" | "crossEncoder" | "decision";
/**
 * 凭据引用的种类：只存引用，不存密钥值。
 */
export type CredentialKind = "env" | "file";
/**
 * Core 内部对某类工作的具名需求。
 */
export type Role = "extraction" | "queryRewrite" | "ranking" | "abstention";

/**
 * 导出 JSON Schema 时把全部 wire 类型收进来的根对象。
 *
 * 它只服务代码生成，不出现在任何请求或响应里。
 */
export interface WireContract {
  capability: Capability;
  coreIdentity: CoreIdentity;
  createObservationRequest: CreateObservationRequest;
  credentialKind: CredentialKind;
  credentialRef: CredentialRef;
  errorBody: ErrorBody;
  errorPayload: ErrorPayload;
  handshakeRequest: HandshakeRequest;
  handshakeResponse: HandshakeResponse;
  observation: Observation;
  observationList: ObservationList;
  providerConfig: ProviderConfig;
  providerConfigSnapshot: ProviderConfigSnapshot;
  providerEntry: ProviderEntry;
  readyLine: ReadyLine;
  role: Role;
  roleBinding: RoleBinding;
  roleChain: RoleChain;
}
/**
 * 两端共享的版本与进程信息。握手响应与 ready 行都平铺这几个字段。
 */
export interface CoreIdentity {
  coreVersion: string;
  protocolVersion: string;
  schemaVersion: number;
  /**
   * 本次进程启动的时间；重启会变。
   */
  startedAt: string;
}
/**
 * 一条待写入的 Observation；未给出事件时间时，Core 用摄入时刻。
 */
export interface CreateObservationRequest {
  content: string;
  /**
   * 事件发生的时刻（RFC3339）。省略时 Core 用摄入时刻，两者不混为一谈。
   */
  occurredAt?: string | null;
  /**
   * 会话标识；同一会话重复摄入由此归并（去重在 #8 完整实现）。
   */
  sessionId: string;
  /**
   * 来源，例如 `host.message`、`tool.call`。
   */
  source: string;
  /**
   * 说话者或产生者；工具事件可省略。
   */
  speaker?: string | null;
}
export interface CredentialRef {
  kind: CredentialKind;
  /**
   * `env` 时是环境变量名，`file` 时是文件路径；都不含密钥值。
   */
  reference: string;
}
export interface ErrorBody {
  error: ErrorPayload;
}
export interface ErrorPayload {
  code: string;
  details?: unknown;
  message: string;
}
export interface HandshakeRequest {
  adapterVersion?: string | null;
  protocolVersion: string;
}
/**
 * 握手成功时返回的 Core 身份。
 */
export interface HandshakeResponse {
  coreVersion: string;
  protocolVersion: string;
  schemaVersion: number;
  /**
   * 本次进程启动的时间；重启会变。
   */
  startedAt: string;
}
/**
 * 已落盘的 Observation。`occurredAt` 是事件时刻，由请求给出或用摄入时刻补。
 */
export interface Observation {
  content: string;
  id: number;
  occurredAt: string;
  sessionId: string;
  source: string;
  speaker?: string | null;
}
export interface ObservationList {
  observations: Observation[];
}
/**
 * Core 自持的 provider 配置；出站白名单等于这里的 endpoint 集合。
 */
export interface ProviderConfig {
  providers?: ProviderEntry[];
  roles?: RoleChain[];
}
export interface ProviderEntry {
  capabilities: Capability[];
  credential: CredentialRef;
  endpoint: string;
  name: string;
}
/**
 * 一个角色的有序 fallback 链。
 */
export interface RoleChain {
  chain: RoleBinding[];
  role: Role;
}
/**
 * 能力角色的一项绑定：某 provider 的某条能力。
 */
export interface RoleBinding {
  capability: Capability;
  provider: string;
}
/**
 * 读取或写入配置后的视图：补上未配置的角色、凭据取不到的 provider 与派生的出站白名单。
 */
export interface ProviderConfigSnapshot {
  /**
   * 凭据引用此刻取不到的 provider；它们仍在白名单里，但用到时按 ADR-0003 降级。
   */
  degradedProviders: string[];
  outboundWhitelist: string[];
  providers: ProviderEntry[];
  roles: RoleChain[];
  /**
   * 没有任何角色绑定的能力角色；显式列出来，不隐式挑一个 provider。
   */
  unconfiguredRoles: Role[];
}
/**
 * Core 就绪后写到 stdout 的那一行。Adapter 与测试脚本都从它取端口。
 */
export interface ReadyLine {
  coreVersion: string;
  dataFile: string;
  endpoint: string;
  event: string;
  pid: number;
  protocolVersion: string;
  schemaVersion: number;
  /**
   * 本次进程启动的时间；重启会变。
   */
  startedAt: string;
}
