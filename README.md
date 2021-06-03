# Godbox
Secure sandboxing system for untrusted code execution.

It uses [isolate](https://github.com/ioi/isolate) which uses specific functionnalities of the Linux kernel, thus godbox not able to run properly outside of Linux.

# Installation
### Docker Compose
```yml
version: "3"

services:
  godbox:
    image: quantumsheep/godbox:2
    privileged: true
    ports:
      - 8080:8080
```

### Docker
```sh
docker run -it -d --privileged -p 8080:8080 quantumsheep/godbox:2
```

# Usage
## POST /run
### Properties
| Name             | Type                     | Description                                                     |
|------------------|--------------------------|-----------------------------------------------------------------|
| phases*          | `Phase[]`                | Execution phases (check examples bellow)                        |
| files*           | `string`                 | Base64-encoded zip file containing the files used in the phases |
| environment      | `Record<string, string>` | Environment variables used in all phases                        |
| sandbox_settings | `SandboxSettings`        | Override default sandbox limitation settings                    |

```ts
interface SandboxSettings {
  run_time_limit?: number = 5;
  extra_time_limit?: number = 0;
  wall_time_limit?: number = 10;
  stack_size_limit?: number = 128000;
  process_count_limit?: number = 120;
  memory_limit?: number = 512000;
  storage_limit?: number = 10240;
}

interface Phase {
  name?: string;
  
  // Multi-line bash script 
  script: string;

  // Environment variables
  environment?: Record<string, string>;

  // Override default sandbox limitation settings
  sandbox_settings?: SandboxSettings;

  // Enable profiling (WIP)
  profiling?: boolean = false;
}
```

### Example
**The files should be passed as a base64 zip archive.**

The folowing demonstration uses the folowing file architecture:
```
.
└── src
    └── main.c
```

Encoded using command `zip -q -r - * | base64` (could have been a library, it doesn't matter while it keeps beeing `files -> zip -> base64`).

```json
{
  "phases": [
    {
      "name": "Compilation",
      "script": "/usr/local/gcc-11.1.0/bin/gcc src/main.c -o out",
      "sandbox_settings": {
        "run_time_limit": 20,
        "wall_time_limit": 40
      },
    },
    {
      "name": "Execution",
      "script": "./out"
    }
  ],
  "environment": {
    "ENABLE_AWESOME_SHEEP": "true"
  },
  "files": "UEsDBAoAAAAAAJe1pVIAAAAAAAAAAAAAAAAEABwAc3JjL1VUCQADvgOTYNQDk2B1eAsAAQT1AQAABBQAAABQSwMEFAAIAAgABbalUgAAAAAAAAAATQAAAAoAHABzcmMvbWFpbi5jVVQJAAOKBJNgjASTYHV4CwABBPUBAAAEFAAAAFPOzEvOKU1JVbApLknJzNfLsOPiyswrUchNzMzT0OSq5lIAgoLSkmINJY/UnJx8HYXw/KKcFEUlTWsusFxRaklpUZ6CgTVXLRcAUEsHCMUkHr9KAAAATQAAAFBLAQIeAwoAAAAAAJe1pVIAAAAAAAAAAAAAAAAEABgAAAAAAAAAEADtQQAAAABzcmMvVVQFAAO+A5NgdXgLAAEE9QEAAAQUAAAAUEsBAh4DFAAIAAgABbalUsUkHr9KAAAATQAAAAoAGAAAAAAAAQAAAKSBPgAAAHNyYy9tYWluLmNVVAUAA4oEk2B1eAsAAQT1AQAABBQAAABQSwUGAAAAAAIAAgCaAAAA3AAAAAAA"
}
```

#### Output
```json
{
  "phases": [
    {
      "name": "Compilation",
      "status": 0,
      "stdout": "",
      "stderr": "OK (0.041 sec real, 0.048 sec wall)\n"
    },
    {
      "name": "Execution",
      "status": 0,
      "stdout": "Hello, World!\n",
      "stderr": "OK (0.001 sec real, 0.005 sec wall)\n"
    }
  ]
}
```
