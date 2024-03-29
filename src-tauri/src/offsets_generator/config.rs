use serde::{Deserialize, Serialize};

/// Represents an operation that can be performed on a memory address.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Operation {
    /// Represents an `add` operation.
    ///
    /// `value` is the value to add.
    Add { value: usize },

    /// Represents a `dereference` operation.
    ///
    /// `times` is the number of times to dereference the address. If `None`, the number of times will be `1`.
    /// `size` is the size of the resulting value. If `None`, the size will be `8`.
    Deref {
        times: Option<usize>,
        size: Option<usize>,
    },

    /// Represents an operation to resolve the absolute address of a relative call.
    ///
    /// `offset` is the offset of the displacement value. If `None`, the offset will be `0x1`.
    /// `length` is the length of the instruction. If `None`, the length will be `0x5`.
    Jmp {
        offset: Option<usize>,
        length: Option<usize>,
    },

    /// Represents an operation to resolve the absolute address of a RIP-relative address.
    ///
    /// `offset` is the offset of the displacement value. If `None`, the offset will be `0x3`.
    /// `length` is the length of the instruction. If `None`, the length will be `0x7`.
    Rip {
        offset: Option<usize>,
        length: Option<usize>,
    },

    /// Represents a `slice` operation.
    ///
    /// `start` is the start index of the slice.
    /// `end` is the end index of the slice.
    Slice { start: usize, end: usize },

    /// Represents a `subtract` operation.
    ///
    /// `value` is the value to subtract.
    Sub { value: usize },
}

/// Represents a signature specified in the `config.json` file.
#[derive(Debug, Deserialize, Serialize)]
pub struct Signature {
    /// The name of the signature.
    pub name: String,

    /// The name of the module.
    pub module: String,

    /// The pattern of the signature.
    pub pattern: String,

    /// The list of operations to perform on the target address.
    pub operations: Vec<Operation>,
}

/// Represents the `config.json` file.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// The list of signatures specified in the `config.json` file.
    pub signatures: Vec<Signature>,
}

// static json string of config

pub static RAW_CONFIG: &str = r#"{
    "signatures": [
      {
        "name": "dwBuildNumber",
        "module": "engine2.dll",
        "pattern": "8B 05 ? ? ? ? C3 CC CC CC CC CC CC CC CC CC 48 8B 0D ? ? ? ? 48 8D 05",
        "operations": [
          {
            "type": "rip",
            "offset": 2,
            "length": 6
          }
        ]
      },
      {
        "name": "dwEntityList",
        "module": "client.dll",
        "pattern": "48 8B 0D ? ? ? ? 48 89 7C 24 ? 8B FA C1 EB",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwForceAttack",
        "module": "client.dll",
        "pattern": "48 8D 0D ? ? ? ? E9 C4 42 B4 FF",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "sub",
            "value": 104
          }
        ]
      },
      {
        "name": "dwForceAttack2",
        "module": "client.dll",
        "pattern": "48 8D 0D ? ? ? ? E9 C4 42 B4 FF",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "add",
            "value": 40
          }
        ]
      },
      {
        "name": "dwForceBackward",
        "module": "client.dll",
        "pattern": "48 8B 05 ? ? ? ? 4C 8D 1D",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "add",
            "value": 48
          }
        ]
      },
      {
        "name": "dwForceCrouch",
        "module": "client.dll",
        "pattern": "48 8B 05 ? ? ? ? 4C 8D 3D ? ? ? ? 48 89 45",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "add",
            "value": 48
          }
        ]
      },
      {
        "name": "dwForceForward",
        "module": "client.dll",
        "pattern": "48 8B 05 ? ? ? ? 4C 8D 0D ? ? ? ? 48 89 45",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "add",
            "value": 48
          }
        ]
      },
      {
        "name": "dwForceJump",
        "module": "client.dll",
        "pattern": "48 8B 05 ? ? ? ? 48 8D 1D ? ? ? ? 48 89 45",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "add",
            "value": 48
          }
        ]
      },
      {
        "name": "dwForceLeft",
        "module": "client.dll",
        "pattern": "48 8B 05 ? ? ? ? 48 8D 0D ? ? ? ? 44 8B 15",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "add",
            "value": 48
          }
        ]
      },
      {
        "name": "dwForceRight",
        "module": "client.dll",
        "pattern": "48 8B 05 ? ? ? ? 48 8D 15 ? ? ? ? 48 89 45",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "add",
            "value": 48
          }
        ]
      },
      {
        "name": "dwGameEntitySystem",
        "module": "client.dll",
        "pattern": "48 8B 1D ? ? ? ? 48 89 1D",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwGameEntitySystem_getHighestEntityIndex",
        "module": "client.dll",
        "pattern": "8B 81 ? ? ? ? 89 02 48 8B C2 C3 CC CC CC CC 48 89 5C 24 ? 48 89 6C 24",
        "operations": [
          {
            "type": "slice",
            "start": 2,
            "end": 4
          }
        ]
      },
      {
        "name": "dwGameRules",
        "module": "client.dll",
        "pattern": "48 89 0D ? ? ? ? 8B 0D",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwGlobalVars",
        "module": "client.dll",
        "pattern": "48 89 0D ? ? ? ? 48 89 41",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwGlowManager",
        "module": "client.dll",
        "pattern": "48 8B 05 ? ? ? ? C3 CC CC CC CC CC CC CC CC 48 89 5C 24 ? 48 89 6C 24",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwInputSystem",
        "module": "inputsystem.dll",
        "pattern": "48 89 05 ? ? ? ? 48 8D 05",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwInterfaceLinkList",
        "module": "client.dll",
        "pattern": "4C 8B 0D ? ? ? ? 4C 8B D2 4C 8B D9",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwLocalPlayerController",
        "module": "client.dll",
        "pattern": "48 8B 05 ? ? ? ? 48 85 C0 74 4F",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwLocalPlayerPawn",
        "module": "client.dll",
        "pattern": "48 8D 05 ? ? ? ? C3 CC CC CC CC CC CC CC CC 48 83 EC ? 8B 0D",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "add",
            "value": 312
          }
        ]
      },
      {
        "name": "dwNetworkGameClient",
        "module": "engine2.dll",
        "pattern": "48 89 3D ? ? ? ? 48 8D 15",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwNetworkGameClient_getLocalPlayer",
        "module": "engine2.dll",
        "pattern": "48 83 C0 ? 48 8D 04 40 8B 0C C1",
        "operations": [
          {
            "type": "slice",
            "start": 3,
            "end": 4
          },
          {
            "type": "add",
            "value": 230
          }
        ]
      },
      {
        "name": "dwNetworkGameClient_maxClients",
        "module": "engine2.dll",
        "pattern": "8B 81 ? ? ? ? C3 CC CC CC CC CC CC CC CC CC 48 8D 81",
        "operations": [
          {
            "type": "slice",
            "start": 2,
            "end": 4
          }
        ]
      },
      {
        "name": "dwNetworkGameClient_signOnState",
        "module": "engine2.dll",
        "pattern": "44 8B 81 ? ? ? ? 48 8D 0D",
        "operations": [
          {
            "type": "slice",
            "start": 3,
            "end": 5
          }
        ]
      },
      {
        "name": "dwPlantedC4",
        "module": "client.dll",
        "pattern": "48 8B 15 ? ? ? ? FF C0 48 8D 4C 24",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwPrediction",
        "module": "client.dll",
        "pattern": "48 8D 05 ? ? ? ? C3 CC CC CC CC CC CC CC CC 48 83 EC ? 8B 0D",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwSensitivity",
        "module": "client.dll",
        "pattern": "48 8B 05 ? ? ? ? 48 8B 40 ? F3 0F 10 00 F3 0F 59 86",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwSensitivity_sensitivity",
        "module": "client.dll",
        "pattern": "FF 50 ? 4C 8B C6 48 8D 55 ? 48 8B CF E8 ? ? ? ? 84 C0 0F 85 ? ? ? ? 4C 8D 45 ? 8B D3 48 8B CF E8 ? ? ? ? E9 ? ? ? ? F3 0F 10 06",
        "operations": [
          {
            "type": "slice",
            "start": 2,
            "end": 3
          }
        ]
      },
      {
        "name": "dwViewAngles",
        "module": "client.dll",
        "pattern": "48 8B 0D ? ? ? ? E9 ? ? ? ? CC CC CC CC 40 55",
        "operations": [
          {
            "type": "rip"
          },
          {
            "type": "deref"
          },
          {
            "type": "add",
            "value": 24896
          }
        ]
      },
      {
        "name": "dwViewMatrix",
        "module": "client.dll",
        "pattern": "48 8D 0D ? ? ? ? 48 C1 E0 06",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwViewRender",
        "module": "client.dll",
        "pattern": "48 89 05 ? ? ? ? 48 8B C8 48 85 C0",
        "operations": [
          {
            "type": "rip"
          }
        ]
      },
      {
        "name": "dwWindowHeight",
        "module": "engine2.dll",
        "pattern": "8B 05 ? ? ? ? 89 03",
        "operations": [
          {
            "type": "rip",
            "offset": 2,
            "length": 6
          }
        ]
      },
      {
        "name": "dwWindowWidth",
        "module": "engine2.dll",
        "pattern": "8B 05 ? ? ? ? 89 07",
        "operations": [
          {
            "type": "rip",
            "offset": 2,
            "length": 6
          }
        ]
      }
    ]
  }"#;
